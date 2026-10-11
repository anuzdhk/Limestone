// Release builds are GUI-only: no console window on Windows.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod config;
mod curve_widget;
mod export_queue;
mod icons;
mod panels;
mod recovery;
mod render_worker;
mod theme;
mod widgets;

use adjust_core::{get_factory_presets, AdjustParams, HistogramData, Preset, RgbImageF32};
use config::{AppConfig, AppLanguage};
use eframe::egui;
use export_queue::{BackgroundExporter, ExportJob, ExportResult};
use icons::*;
use image::RgbImage;
use panels::adjustments::render_adjustments_accordion;
use panels::canvas::render_central_canvas;
use panels::inspector::{render_inspector_panel, InspectorTab};
use panels::presets::render_presets_sidebar;
use panels::statusbar::render_bottom_statusbar;
use panels::toolbar::{render_top_toolbar, ViewMode};
use recovery::AutoSaveManager;
use render_worker::{PixelRect, RenderResult, RenderWorker, TileKey, PREVIEW_MAX_DIM};
use std::path::PathBuf;
use std::sync::Arc;
use theme::*;
use widgets::toast::ToastManager;

fn load_app_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon.png");
    if let Ok(image) = image::load_from_memory(png_bytes) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1440.0, 920.0])
        .with_min_inner_size([1100.0, 700.0])
        .with_title("Limestone Pro — Studio Image Enhancement");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Limestone Pro",
        native_options,
        Box::new(|cc| {
            // Setup font with Tabler Icons
            setup_tabler_fonts(&cc.egui_ctx);
            let config = AppConfig::load();
            apply_theme_configured(&cc.egui_ctx, config.theme_mode, config.accent_theme, config.ui_scale);
            Ok(Box::new(AdjustApp::new(cc, config)))
        }),
    )
}

struct AdjustApp {
    config: AppConfig,
    image_path: Option<PathBuf>,
    /// Image exactly as loaded (no layers applied).
    source_f32: Option<Arc<RgbImageF32>>,
    /// Full-resolution image with all applied layers baked in; the current params render on top.
    base_f32: Option<Arc<RgbImageF32>>,
    /// Downsampled proxy of `base_f32` used for interactive previews.
    preview_base: Option<Arc<RgbImageF32>>,
    /// Bumped whenever `base_f32` changes so stale tiles/bakes are discarded.
    base_version: u64,
    preview_generation: u64,
    baking: bool,

    orig_texture: Option<egui::TextureHandle>,
    proc_texture: Option<egui::TextureHandle>,
    nav_texture: Option<egui::TextureHandle>,

    params: AdjustParams,
    presets: Vec<Preset>,
    selected_preset_name: String,
    selected_collection: String,
    preset_search: String,

    view_mode: ViewMode,
    inspector_tab: InspectorTab,
    curve_channel: crate::curve_widget::CurveChannel,
    split_ratio: f32, // 0.0 .. 1.0
    zoom: f32,
    pan: egui::Vec2,
    canvas_size: egui::Vec2,

    histogram: Option<HistogramData>,
    is_dirty: bool,
    is_rendering: bool,
    last_process_time: f32,

    // Undo / Redo & effect stacking
    undo_stack: Vec<AdjustParams>,
    redo_stack: Vec<AdjustParams>,
    /// Params of each applied ("baked") layer, oldest first. `base_f32` == source + these layers.
    layer_snapshots: Vec<AdjustParams>,

    // Full-resolution tile shown when zoomed in past the preview's resolution
    tile: Option<(TileKey, egui::TextureHandle)>,
    pending_tile: Option<TileKey>,
    canvas_img_rect: Option<egui::Rect>,
    canvas_rect: Option<egui::Rect>,
    fit_pending: bool,

    // UI state & services
    show_sidebars: bool,
    show_shortcuts_modal: bool,
    show_settings_modal: bool,
    toasts: ToastManager,
    exporter: BackgroundExporter,
    worker: RenderWorker,
    auto_saver: AutoSaveManager,
}

impl AdjustApp {
    fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        let presets = get_factory_presets();
        let default_preset = presets
            .iter()
            .find(|p| p.name == "Mild Contrast Pop")
            .cloned()
            .unwrap_or_else(|| presets[0].clone());

        let mut app = Self {
            config,
            image_path: None,
            source_f32: None,
            base_f32: None,
            preview_base: None,
            base_version: 0,
            preview_generation: 0,
            baking: false,
            orig_texture: None,
            proc_texture: None,
            nav_texture: None,
            params: default_preset.params.clone(),
            presets,
            selected_preset_name: default_preset.name,
            selected_collection: "All Collections".to_string(),
            preset_search: String::new(),
            view_mode: ViewMode::Split,
            inspector_tab: InspectorTab::Navigate,
            curve_channel: crate::curve_widget::CurveChannel::Rgb,
            split_ratio: 0.5,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            canvas_size: egui::vec2(900.0, 600.0),
            histogram: None,
            is_dirty: false,
            is_rendering: false,
            last_process_time: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            layer_snapshots: Vec::new(),
            tile: None,
            pending_tile: None,
            canvas_img_rect: None,
            canvas_rect: None,
            fit_pending: true,
            show_sidebars: true,
            show_shortcuts_modal: false,
            show_settings_modal: false,
            toasts: ToastManager::new(),
            exporter: BackgroundExporter::new(),
            worker: RenderWorker::new(cc.egui_ctx.clone()),
            auto_saver: AutoSaveManager::new(),
        };

        // A recovery file only survives an unclean exit (it is deleted in `on_exit`).
        if let Some(recovery) = AutoSaveManager::try_recover() {
            if let Some(path) = recovery.image_path.clone().filter(|p| p.exists()) {
                if let Ok(img) = RgbImageF32::open(&path) {
                    app.set_image(img, Some(path));
                    app.params = recovery.params;
                    if !recovery.layers.is_empty() {
                        app.layer_snapshots = recovery.layers;
                        app.rebake_layers();
                    }
                    app.toasts.info("Recovered your previous session after an unexpected exit");
                    return app;
                }
            }
        }

        app.load_default_sample();
        app
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.params.clone());
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.params.clone());
            self.params = prev;
            self.is_dirty = true;
            self.toasts.info("Undone adjustment");
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.params.clone());
            self.params = next;
            self.is_dirty = true;
            self.toasts.info("Redone adjustment");
        }
    }

    fn load_default_sample(&mut self) {
        let width = 960;
        let height = 640;
        let mut img = RgbImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / width as f32;
                let fy = y as f32 / height as f32;

                let sky_r = (30.0 + fx * 50.0) as u8;
                let sky_g = (80.0 + (1.0 - fy) * 110.0) as u8;
                let sky_b = (195.0 + (1.0 - fy) * 60.0) as u8;

                let ground_r = ((fy * 170.0) as u8).saturating_add((fx * 45.0) as u8);
                let ground_g = (((1.0 - fx) * 130.0) as u8).saturating_add(35);
                let ground_b: u8 = 45;

                let (r, g, b) = if fy < 0.52 {
                    (sky_r, sky_g, sky_b)
                } else {
                    let checker = if (x / 8 + y / 8) % 2 == 0 { 28 } else { 0 };
                    (
                        ground_r.saturating_add(checker),
                        ground_g.saturating_add(checker),
                        ground_b.saturating_add(checker),
                    )
                };

                img.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }

        self.set_image(RgbImageF32::from_rgb8(&img), None);
    }

    /// Replaces the working image. `path` is `None` for images without a file (sample, clipboard).
    fn set_image(&mut self, img: RgbImageF32, path: Option<PathBuf>) {
        let source = Arc::new(img);
        let label = match &path {
            Some(p) => p.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            None => format!("{}×{} image", source.width, source.height),
        };

        self.preview_base = Some(Arc::new(source.downsample(PREVIEW_MAX_DIM)));
        self.base_f32 = Some(source.clone());
        self.source_f32 = Some(source);
        self.base_version += 1;
        self.baking = false;
        self.layer_snapshots.clear();
        self.image_path = path.clone();
        self.orig_texture = None;
        self.proc_texture = None;
        self.nav_texture = None;
        self.tile = None;
        self.pending_tile = None;
        self.fit_pending = true;
        self.is_dirty = true;

        if let Some(path) = path {
            self.config.add_recent(path);
        }
        self.toasts.success(format!("Opened {}", label));
    }

    fn open_image_path(&mut self, path: PathBuf) {
        match RgbImageF32::open(&path) {
            Ok(img) => self.set_image(img, Some(path)),
            Err(e) => self.toasts.error(format!(
                "Failed to open {}: {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                e
            )),
        }
    }

    /// Pastes a bitmap, a copied file (e.g. from Explorer/Finder), or a file path from the clipboard.
    fn paste_image_from_clipboard(&mut self) {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                self.toasts.error(format!("Clipboard unavailable: {}", e));
                return;
            }
        };

        if let Ok(data) = clipboard.get_image() {
            let (w, h) = (data.width as u32, data.height as u32);
            if let Some(rgba) = image::RgbaImage::from_raw(w, h, data.bytes.into_owned()) {
                let mut img = RgbImageF32::from_rgba8(&rgba);
                // Clipboard bitmaps often carry a meaningless fully-opaque alpha channel.
                if img.alpha.as_ref().is_some_and(|a| a.iter().all(|&v| v >= 1.0)) {
                    img.alpha = None;
                }
                self.set_image(img, None);
                return;
            }
        }

        let path = clipboard
            .get()
            .file_list()
            .ok()
            .and_then(|files| files.into_iter().next())
            .or_else(|| {
                clipboard
                    .get_text()
                    .ok()
                    .map(|t| PathBuf::from(t.trim().trim_matches('"')))
            })
            .filter(|p| p.is_file());

        match path {
            Some(path) => self.open_image_path(path),
            None => self.toasts.warn("Clipboard does not contain an image"),
        }
    }

    fn open_image_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp"],
            )
            .pick_file()
        {
            self.open_image_path(path);
        }
    }

    fn queue_export_dialog(&mut self) {
        if self.baking {
            self.toasts.warn("Please wait until the applied layer finishes rendering");
            return;
        }
        if let Some(ref base) = self.base_f32 {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("TIFF Image", &["tif", "tiff"])
                .save_file()
            {
                self.exporter.queue_export(ExportJob {
                    save_path: path.clone(),
                    base: base.clone(),
                    params: self.params.clone(),
                });
                self.toasts.info(format!(
                    "Queued background export to {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        } else {
            self.toasts.warn("No image available to export");
        }
    }

    /// Queues a preview render of the current params on the background worker.
    fn trigger_reprocess(&mut self) {
        // While a layer is baking the preview base is stale; render once the bake lands.
        if self.baking {
            return;
        }
        if let Some(ref preview) = self.preview_base {
            self.preview_generation += 1;
            self.worker
                .request_preview(self.preview_generation, preview.clone(), self.params.clone());
            self.is_rendering = true;
        }
        self.is_dirty = false;

        self.auto_saver
            .auto_save(self.image_path.as_ref(), &self.layer_snapshots, &self.params);
    }

    /// Applies finished background renders to the UI state.
    fn poll_worker(&mut self, ctx: &egui::Context) {
        for result in self.worker.poll() {
            match result {
                RenderResult::Preview {
                    generation,
                    color_image,
                    histogram,
                    elapsed_ms,
                } => {
                    if generation != self.preview_generation {
                        continue; // superseded by a newer request
                    }
                    match self.proc_texture {
                        Some(ref mut tex) => tex.set(color_image, egui::TextureOptions::LINEAR),
                        None => {
                            self.proc_texture = Some(ctx.load_texture(
                                "processed_view",
                                color_image,
                                egui::TextureOptions::LINEAR,
                            ))
                        }
                    }
                    self.histogram = Some(*histogram);
                    self.last_process_time = elapsed_ms;
                    self.is_rendering = false;
                }
                RenderResult::Tile { key, color_image } => {
                    if self.pending_tile.as_ref() == Some(&*key) {
                        let tex = ctx.load_texture(
                            "high_res_view_tile",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.tile = Some((*key, tex));
                        self.pending_tile = None;
                    }
                }
                RenderResult::Baked {
                    base_version,
                    base,
                    preview,
                } => {
                    if base_version != self.base_version {
                        continue; // a newer bake (or a new image) replaced this one
                    }
                    self.base_f32 = Some(base);
                    self.preview_base = Some(preview);
                    self.baking = false;
                    self.orig_texture = None;
                    self.tile = None;
                    self.pending_tile = None;
                    self.is_dirty = true;
                }
            }
        }

        // (Re)create the "before" textures from the current base proxy.
        if self.orig_texture.is_none() {
            if let Some(ref preview) = self.preview_base {
                let color_image = render_worker::to_color_image(preview);
                self.orig_texture = Some(ctx.load_texture(
                    "original_view",
                    color_image.clone(),
                    egui::TextureOptions::LINEAR,
                ));
                self.nav_texture = Some(ctx.load_texture(
                    "nav_view",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
    }

    /// Re-renders the base image from the source with the current layer stack.
    fn rebake_layers(&mut self) {
        if let Some(ref source) = self.source_f32 {
            self.base_version += 1;
            self.baking = true;
            self.tile = None;
            self.pending_tile = None;
            self.worker
                .request_bake(self.base_version, source.clone(), self.layer_snapshots.clone());
        }
    }

    /// "Apply": bakes the current params into the full-resolution base image as a new layer.
    fn apply_layer(&mut self) {
        if self.baking {
            self.toasts.warn("Still applying the previous layer");
            return;
        }
        let Some(base) = self.base_f32.clone() else {
            return;
        };
        let params = std::mem::take(&mut self.params);
        self.layer_snapshots.push(params.clone());
        self.base_version += 1;
        self.baking = true;
        self.tile = None;
        self.pending_tile = None;
        self.worker.request_bake(self.base_version, base, vec![params]);

        self.selected_preset_name = "Neutral".to_string();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.is_dirty = true;
        self.toasts.success("Applying effect layer…");
    }

    /// "Undo Layer": removes the last applied layer and puts its settings back on the sliders.
    fn undo_layer(&mut self) {
        if self.baking {
            self.toasts.warn("Still applying the previous layer");
            return;
        }
        if let Some(last) = self.layer_snapshots.pop() {
            self.params = last;
            self.selected_preset_name = "Custom".to_string();
            self.undo_stack.clear();
            self.redo_stack.clear();
            self.rebake_layers();
            self.is_dirty = true;
            self.toasts.info("Removed last effect layer");
        }
    }

    /// "Cancel": drops every applied layer and resets the sliders.
    fn cancel_all_layers(&mut self) {
        self.params = AdjustParams::default();
        self.selected_preset_name = "Neutral".to_string();
        if !self.layer_snapshots.is_empty() {
            self.layer_snapshots.clear();
            self.rebake_layers();
        }
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.is_dirty = true;
        self.toasts.warn("Canceled all adjustments");
    }

    /// Size of the full-resolution image in pixels.
    fn image_px_size(&self) -> Option<egui::Vec2> {
        self.base_f32
            .as_ref()
            .map(|img| egui::vec2(img.width as f32, img.height as f32))
    }

    /// Zoom (physical pixels per image pixel) that fits the image inside the canvas.
    fn fit_zoom(&self, ctx: &egui::Context) -> f32 {
        let Some(img) = self.image_px_size() else {
            return 1.0;
        };
        let ppp = ctx.pixels_per_point();
        let canvas = self.canvas_size;
        let zoom = (canvas.x / img.x).min(canvas.y / img.y) * 0.95 * ppp;
        if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        }
    }

    /// Requests a full-resolution tile for the visible area when the preview proxy
    /// would be magnified (i.e. zoomed in past the preview's own resolution).
    fn update_tile_request(&mut self, ctx: &egui::Context) {
        let wants_tile = self.config.high_quality_tiled_preview
            && self.view_mode == ViewMode::Single
            && !self.baking;
        let (Some(base), Some(preview), Some(img_rect), Some(canvas_rect)) = (
            self.base_f32.clone(),
            self.preview_base.as_ref(),
            self.canvas_img_rect,
            self.canvas_rect,
        ) else {
            return;
        };
        let preview_zoom = preview.width as f32 / base.width as f32;
        if !wants_tile || self.zoom <= preview_zoom * 1.05 || base.width <= preview.width {
            self.pending_tile = None;
            return;
        }

        let visible = canvas_rect.intersect(img_rect);
        if visible.width() < 4.0 || visible.height() < 4.0 {
            return;
        }
        let to_px = |screen: f32, min: f32, len: f32, px: usize| {
            ((screen - min) / len * px as f32).clamp(0.0, px as f32)
        };
        // Snap outwards to a 128px grid so small pans reuse the same tile.
        const GRID: usize = 128;
        let x0 = to_px(visible.min.x, img_rect.min.x, img_rect.width(), base.width) as usize / GRID * GRID;
        let y0 = to_px(visible.min.y, img_rect.min.y, img_rect.height(), base.height) as usize / GRID * GRID;
        let x1 = (to_px(visible.max.x, img_rect.min.x, img_rect.width(), base.width).ceil() as usize)
            .div_ceil(GRID)
            .saturating_mul(GRID)
            .min(base.width);
        let y1 = (to_px(visible.max.y, img_rect.min.y, img_rect.height(), base.height).ceil() as usize)
            .div_ceil(GRID)
            .saturating_mul(GRID)
            .min(base.height);
        if x1 <= x0 || y1 <= y0 || (x1 - x0) * (y1 - y0) > 8_000_000 {
            return;
        }

        let key = TileKey {
            base_version: self.base_version,
            params: self.params.clone(),
            rect: PixelRect {
                x: x0,
                y: y0,
                w: x1 - x0,
                h: y1 - y0,
            },
        };
        let already_have = self.tile.as_ref().is_some_and(|(k, _)| *k == key);
        if !already_have && self.pending_tile.as_ref() != Some(&key) {
            self.worker.request_tile(key.clone(), base);
            self.pending_tile = Some(key);
            ctx.request_repaint();
        }
    }

    /// The current tile and its normalized rect, if it matches what's on screen.
    fn visible_tile(&self) -> Option<(egui::TextureHandle, egui::Rect)> {
        let (key, tex) = self.tile.as_ref()?;
        if key.base_version != self.base_version || key.params != self.params {
            return None;
        }
        let base = self.base_f32.as_ref()?;
        let (w, h) = (base.width as f32, base.height as f32);
        let r = key.rect;
        let norm = egui::Rect::from_min_max(
            egui::pos2(r.x as f32 / w, r.y as f32 / h),
            egui::pos2((r.x + r.w) as f32 / w, (r.y + r.h) as f32 / h),
        );
        Some((tex.clone(), norm))
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Don't steal keys (digits, F, Ctrl+V...) while the user is typing in a text field.
        if ctx.wants_keyboard_input() {
            return;
        }
        let fit_zoom = self.fit_zoom(ctx);
        let mut paste = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Tab) {
                self.show_sidebars = !self.show_sidebars;
            }

            if i.key_pressed(egui::Key::Questionmark)
                || (i.modifiers.shift && i.key_pressed(egui::Key::Slash))
            {
                self.show_shortcuts_modal = !self.show_shortcuts_modal;
            }

            if i.key_pressed(egui::Key::Num1) {
                self.view_mode = ViewMode::Single;
            }
            if i.key_pressed(egui::Key::Num2) {
                self.view_mode = ViewMode::Split;
            }
            if i.key_pressed(egui::Key::Num3) {
                self.view_mode = ViewMode::SideBySide;
            }

            if i.key_pressed(egui::Key::F) {
                self.zoom = fit_zoom;
                self.pan = egui::Vec2::ZERO;
            }
            if i.key_pressed(egui::Key::Num0) {
                self.zoom = 1.0;
            }

            if i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z) {
                self.redo();
            } else if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                self.undo();
            }

            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                self.open_image_dialog();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::E) {
                self.queue_export_dialog();
            }
            // egui-winit consumes the Ctrl+V *press* (and emits nothing for image clipboards),
            // so react to the key release instead and read the clipboard ourselves.
            paste = i.events.iter().any(|e| {
                matches!(e, egui::Event::Key { key: egui::Key::V, pressed: false, modifiers, .. }
                    if modifiers.command)
            });
        });
        if paste {
            self.paste_image_from_clipboard();
        }
    }

    fn render_settings_modal(&mut self, ctx: &egui::Context) {
        if !self.show_settings_modal {
            return;
        }

        let mut changed_theme = false;
        let mut close_modal = false;

        egui::Window::new(format!("{} Preferences & Settings", ICON_SETTINGS))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(460.0, 380.0))
            .show(ctx, |ui| {
                ui.add_space(SPACE_8);

                // Theme Mode (Dark / Light)
                ui.label(egui::RichText::new("Theme Appearance").strong().size(13.0));
                ui.horizontal(|ui| {
                    if ui.selectable_value(&mut self.config.theme_mode, ThemeMode::DarkStudio, "Dark Studio").clicked() {
                        changed_theme = true;
                    }
                    if ui.selectable_value(&mut self.config.theme_mode, ThemeMode::LightStudio, "Light Studio").clicked() {
                        changed_theme = true;
                    }
                });

                ui.add_space(SPACE_12);

                // Accent Color Selector
                ui.label(egui::RichText::new("Accent Color").strong().size(13.0));
                ui.horizontal(|ui| {
                    let accents = [
                        AccentTheme::ElectricCyan,
                        AccentTheme::VibrantAmber,
                        AccentTheme::EmeraldGreen,
                        AccentTheme::RoyalPurple,
                        AccentTheme::CrimsonRed,
                    ];
                    for acc in accents {
                        let is_active = self.config.accent_theme == acc;
                        let btn = ui.add_sized(
                            [68.0, BTN_HEIGHT_SM],
                            egui::SelectableLabel::new(is_active, acc.name()),
                        );
                        if btn.clicked() {
                            self.config.accent_theme = acc;
                            changed_theme = true;
                        }
                    }
                });

                ui.add_space(SPACE_12);

                // UI Scaling Slider
                ui.label(egui::RichText::new("Interface Scale").strong().size(13.0));
                ui.horizontal(|ui| {
                    if ui.add(egui::Slider::new(&mut self.config.ui_scale, 0.80..=1.50).text("Scale").fixed_decimals(2)).changed() {
                        changed_theme = true;
                    }
                });

                ui.add_space(SPACE_12);

                // Language Selector
                ui.label(egui::RichText::new("Language").strong().size(13.0));
                egui::ComboBox::from_id_salt("lang_combo")
                    .selected_text(self.config.language.name())
                    .show_ui(ui, |ui| {
                        let languages = [
                            AppLanguage::English,
                            AppLanguage::Spanish,
                            AppLanguage::French,
                            AppLanguage::German,
                            AppLanguage::Japanese,
                        ];
                        for lang in languages {
                            if ui.selectable_value(&mut self.config.language, lang, lang.name()).clicked() {
                                self.config.save();
                            }
                        }
                    });

                ui.add_space(SPACE_12);

                // Hardware Acceleration & High-Resolution Preview Toggles
                ui.label(egui::RichText::new("Performance & Engine").strong().size(13.0));
                if ui.checkbox(&mut self.config.gpu_acceleration, format!("{} Multi-Core / GPU Compute Acceleration", ICON_CPU)).on_hover_text("Enables SIMD & multithreaded compute pipelines for sub-15ms filters").changed() {
                    self.config.save();
                }

                if ui.checkbox(&mut self.config.high_quality_tiled_preview, "Tiled 100% Full-Resolution Zoom Preview").on_hover_text("Dynamically renders full-resolution viewport tiles when zoomed in").changed() {
                    self.config.save();
                }

                ui.add_space(SPACE_16);
                ui.vertical_centered(|ui| {
                    let btn_close = egui::Button::new("Close & Save")
                        .min_size(egui::vec2(120.0, BTN_HEIGHT_MD));
                    if ui.add(btn_close).clicked() {
                        close_modal = true;
                    }
                });
            });

        if changed_theme {
            self.config.save();
            apply_theme_configured(ctx, self.config.theme_mode, self.config.accent_theme, self.config.ui_scale);
        }

        if close_modal {
            self.show_settings_modal = false;
        }
    }

    fn render_shortcuts_modal(&mut self, ctx: &egui::Context) {
        if !self.show_shortcuts_modal {
            return;
        }

        egui::Window::new(format!("{} Keyboard Shortcuts", ICON_KEYBOARD))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(440.0, 380.0))
            .show(ctx, |ui| {
                ui.add_space(SPACE_8);
                ui.label(
                    egui::RichText::new("Professional Studio Keybindings")
                        .size(13.0)
                        .color(TEXT_SECONDARY),
                );
                ui.add_space(SPACE_12);

                let shortcuts = [
                    ("Space (Hold)", "Quick toggle compare original"),
                    ("Ctrl+O / Cmd+O", "Open image from filesystem"),
                    ("Ctrl+E / Cmd+E", "Queue background export"),
                    ("Ctrl+V", "Paste image from clipboard"),
                    ("Ctrl+Z / Cmd+Z", "Undo last parameter change"),
                    ("Ctrl+Shift+Z", "Redo last undone parameter"),
                    ("1 / 2 / 3", "Switch view: Single | Split | Side-by-side"),
                    ("F", "Fit image to canvas center"),
                    ("0", "Reset zoom to 100% actual size"),
                    ("Tab", "Toggle sidebars (Focus Mode)"),
                    ("?", "Toggle this shortcuts cheat sheet"),
                    ("Shift + Slider Drag", "Precision fine adjustment"),
                    ("Double-Click Slider", "Reset slider to default value"),
                ];

                for (key, desc) in shortcuts {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [160.0, 18.0],
                            egui::Label::new(
                                egui::RichText::new(key)
                                    .font(egui::FontId::monospace(11.5))
                                    .strong()
                                    .color(self.config.accent_theme.color()),
                            ),
                        );
                        ui.label(egui::RichText::new(desc).size(12.0).color(TEXT_PRIMARY));
                    });
                    ui.add_space(SPACE_4);
                }

                ui.add_space(SPACE_16);
                ui.vertical_centered(|ui| {
                    let btn_close = egui::Button::new("Close")
                        .min_size(egui::vec2(80.0, BTN_HEIGHT_MD));
                    if ui.add(btn_close).clicked() {
                        self.show_shortcuts_modal = false;
                    }
                });
            });
    }
}

impl eframe::App for AdjustApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // A clean exit means there is nothing to recover next launch.
        self.auto_saver.clear_recovery();
        self.config.save();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background export results
        for res in self.exporter.poll_results() {
            match res {
                ExportResult::Success { save_path, elapsed_ms, width, height } => {
                    self.toasts.success(format!(
                        "Exported {} ({}×{} px) in {} ms",
                        save_path.file_name().unwrap_or_default().to_string_lossy(),
                        width,
                        height,
                        elapsed_ms
                    ));
                }
                ExportResult::Failed { save_path, error } => {
                    self.toasts.error(format!(
                        "Failed to export {}: {}",
                        save_path.file_name().unwrap_or_default().to_string_lossy(),
                        error
                    ));
                }
            }
        }

        // Collect finished background renders
        self.poll_worker(ctx);

        if self.fit_pending && self.base_f32.is_some() {
            self.zoom = self.fit_zoom(ctx);
            self.pan = egui::Vec2::ZERO;
            self.fit_pending = false;
        }

        // Handle Global Shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Queue a preview render if parameters changed
        if self.is_dirty {
            self.trigger_reprocess();
        }

        // ==========================================
        // 1. TOP HEADER TOOLBAR (48px)
        // ==========================================
        egui::TopBottomPanel::top("top_header_panel")
            .exact_height(HEIGHT_TOOLBAR)
            .show(ctx, |ui| {
                let mut effect_opacity = ((1.0 - self.params.finishing.transparency) * 100.0).clamp(0.0, 100.0);
                let old_opacity = effect_opacity;

                let tb_resp = render_top_toolbar(
                    ui,
                    &mut self.view_mode,
                    &mut self.zoom,
                    &mut effect_opacity,
                    !self.undo_stack.is_empty(),
                    !self.redo_stack.is_empty(),
                    self.last_process_time,
                    self.exporter.active_jobs(),
                );

                if (effect_opacity - old_opacity).abs() > 0.001 {
                    self.params.finishing.transparency = 1.0 - (effect_opacity / 100.0);
                    self.is_dirty = true;
                }

                if tb_resp.open_clicked {
                    self.open_image_dialog();
                }
                if tb_resp.export_clicked {
                    self.queue_export_dialog();
                }
                if tb_resp.paste_clipboard_clicked {
                    self.paste_image_from_clipboard();
                }
                if tb_resp.undo_clicked {
                    self.undo();
                }
                if tb_resp.redo_clicked {
                    self.redo();
                }
                if tb_resp.reset_all_clicked {
                    self.push_undo();
                    self.params = AdjustParams::default();
                    self.selected_preset_name = "Neutral".to_string();
                    self.is_dirty = true;
                    self.toasts.info("Reset all parameters to default");
                }
                if tb_resp.fit_clicked {
                    self.zoom = self.fit_zoom(ui.ctx());
                    self.pan = egui::Vec2::ZERO;
                }
                if tb_resp.zoom_100_clicked {
                    self.zoom = 1.0;
                }
                if tb_resp.settings_clicked {
                    self.show_settings_modal = true;
                }
                if tb_resp.shortcuts_clicked {
                    self.show_shortcuts_modal = true;
                }
            });

        // ==========================================
        // 2. BOTTOM STATUS BAR (24px)
        // ==========================================
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .exact_height(HEIGHT_STATUSBAR)
            .show(ctx, |ui| {
                let dimensions = self
                    .base_f32
                    .as_ref()
                    .map(|img| (img.width, img.height));
                render_bottom_statusbar(
                    ui,
                    dimensions,
                    self.zoom,
                    self.is_rendering || self.baking || self.exporter.active_jobs() > 0,
                    self.last_process_time,
                    self.base_f32.is_some(),
                );
            });

        // ==========================================
        // 3. LEFT SIDEBAR: PRESET BROWSER (Collapsible)
        // ==========================================
        if self.show_sidebars {
            egui::SidePanel::left("left_preset_panel")
                .resizable(true)
                .default_width(WIDTH_SIDEBAR_LEFT)
                .width_range(220.0..=360.0)
                .show(ctx, |ui| {
                    let p_resp = render_presets_sidebar(
                        ui,
                        &self.presets,
                        &self.selected_preset_name,
                        &mut self.selected_collection,
                        &mut self.preset_search,
                        &mut self.config.favorite_presets,
                        self.nav_texture.as_ref(),
                        &self.config.recent_files,
                    );

                    if let Some(recent_path) = p_resp.open_recent_path {
                        self.open_image_path(recent_path);
                    }

                    if let Some(preset) = p_resp.selected_preset {
                        self.push_undo();
                        self.params = preset.params.clone();
                        self.selected_preset_name = preset.name.clone();
                        self.is_dirty = true;
                        self.toasts.info(format!("Applied preset: {}", preset.name));
                    }

                    if p_resp.save_custom_clicked {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON Preset", &["json"])
                            .save_file()
                        {
                            let custom = Preset {
                                name: "Custom Preset".to_string(),
                                category: "Custom".to_string(),
                                description: "User custom configured preset".to_string(),
                                params: self.params.clone(),
                            };
                            match custom
                                .to_json()
                                .map_err(|e| e.to_string())
                                .and_then(|json| std::fs::write(&path, json).map_err(|e| e.to_string()))
                            {
                                Ok(()) => self.toasts.success("Custom preset saved"),
                                Err(e) => self.toasts.error(format!("Failed to save preset: {}", e)),
                            }
                        }
                    }

                    if p_resp.load_custom_clicked {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON Preset", &["json"])
                            .pick_file()
                        {
                            match std::fs::read_to_string(&path) {
                                Ok(content) => match Preset::from_json(&content) {
                                    Ok(custom) => {
                                        self.push_undo();
                                        // Clamp out-of-range values from hand-edited files.
                                        self.params = custom.params.sanitized();
                                        self.selected_preset_name = custom.name.clone();
                                        self.is_dirty = true;
                                        self.toasts.success(format!("Loaded preset {}", custom.name));
                                    }
                                    Err(e) => self.toasts.error(format!("Invalid preset file: {}", e)),
                                },
                                Err(e) => self.toasts.error(format!("Failed to read preset: {}", e)),
                            }
                        }
                    }
                });

            // ==========================================
            // 4. RIGHT SIDEBAR: INSPECTOR & ACCORDION (Collapsible)
            // ==========================================
            egui::SidePanel::right("right_adjustment_panel")
                .resizable(true)
                .default_width(WIDTH_SIDEBAR_RIGHT)
                .width_range(300.0..=480.0)
                .show(ctx, |ui| {
                    // Top Zone: Inspector tabs
                    let display_size = self
                        .image_px_size()
                        .map(|px| px * self.zoom / ui.ctx().pixels_per_point());
                    render_inspector_panel(
                        ui,
                        &mut self.inspector_tab,
                        self.nav_texture.as_ref(),
                        self.proc_texture.as_ref(),
                        self.histogram.as_ref(),
                        &mut self.pan,
                        display_size,
                        self.canvas_size,
                    );

                    ui.add_space(SPACE_4);
                    ui.separator();
                    ui.add_space(SPACE_4);

                    // Bottom Zone: Accordion adjustments
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let adj_resp = render_adjustments_accordion(
                                ui,
                                &mut self.params,
                                &mut self.curve_channel,
                                self.histogram.as_ref(),
                                &mut self.is_dirty,
                                self.layer_snapshots.len(),
                            );

                            if adj_resp.reset_all_clicked {
                                self.push_undo();
                                self.params = AdjustParams::default();
                                self.selected_preset_name = "Neutral".to_string();
                                self.is_dirty = true;
                                self.toasts.info("Reset all adjustments");
                            }

                            if adj_resp.cancel_clicked {
                                self.cancel_all_layers();
                            }

                            if adj_resp.apply_clicked {
                                self.apply_layer();
                            }

                            if adj_resp.undo_layer_clicked {
                                self.undo_layer();
                            }
                        });
                });
        }

        // ==========================================
        // 5. CENTER CANVAS VIEWPORT
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            let image_px = self.image_px_size().unwrap_or(egui::Vec2::ZERO);
            let tile = self.visible_tile();
            let canvas_resp = render_central_canvas(
                ui,
                self.view_mode,
                &mut self.split_ratio,
                &mut self.zoom,
                &mut self.pan,
                image_px,
                self.proc_texture.as_ref(),
                self.orig_texture.as_ref(),
                tile.as_ref().map(|(tex, rect)| (tex, *rect)),
                self.base_f32.is_some(),
            );

            self.canvas_size = canvas_resp.canvas_size;
            self.canvas_rect = Some(canvas_resp.canvas_rect);
            self.canvas_img_rect = canvas_resp.img_rect;

            if let Some(path) = canvas_resp.dropped_path {
                self.open_image_path(path);
            }

            if canvas_resp.open_clicked {
                self.open_image_dialog();
            }
        });

        // Ask for a full-resolution tile of whatever is now on screen
        self.update_tile_request(ctx);

        // Overlay: Keyboard Shortcuts Cheat Sheet
        self.render_shortcuts_modal(ctx);

        // Overlay: Preferences & Theme Settings
        self.render_settings_modal(ctx);

        // Overlay: In-App Toast Notifications
        self.toasts.show(ctx);
    }
}
