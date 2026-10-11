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
mod thumbnails;
mod user_presets;
mod widgets;

use adjust_core::{get_collections, get_factory_presets, AdjustParams, HistogramData, Preset, PresetCollection, RgbImageF32};
use config::AppConfig;
use curve_widget::CurveChannel;
use eframe::egui;
use export_queue::{BackgroundExporter, ExportJob, ExportResult};
use panels::canvas::{canvas, CanvasState};
use panels::dialogs::{self, SaveOutcome};
use panels::inspector::{inspector, InfoTab, InspectorAction, InspectorState};
use panels::sidebar::{presets_for, sidebar, Library, SidebarAction, SidebarState};
use panels::toolbar::{toolbar, ToolbarAction, ToolbarState, ViewMode};
use recovery::AutoSaveManager;
use render_worker::{PixelRect, RenderResult, RenderWorker, TileKey, PREVIEW_MAX_DIM};
use std::path::PathBuf;
use std::sync::Arc;
use theme::*;
use thumbnails::ThumbnailCache;
use widgets::toast::ToastManager;

const NO_PRESET: &str = "No preset";

fn load_app_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([980.0, 620.0])
        .with_title("LimeStone");
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    eframe::run_native(
        "LimeStone",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            let config = AppConfig::load();
            apply_theme(&cc.egui_ctx, config.theme_mode, config.accent, config.ui_scale);
            Ok(Box::new(AdjustApp::new(cc, config)))
        }),
    )
}

/// Deferred UI commands (so file dialogs never run while egui holds an input lock).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Open,
    Export,
    Paste,
    SavePreset,
    Undo,
    Redo,
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
    user_presets: Vec<Preset>,
    collections: Vec<PresetCollection>,
    thumbs: ThumbnailCache,
    library: Library,
    preset_search: String,
    selected_preset_name: String,

    view_mode: ViewMode,
    info_tab: InfoTab,
    curve_channel: CurveChannel,
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
    /// True while a slider drag is in progress (one undo step per gesture).
    editing_gesture: bool,
    /// Params of each applied ("baked") layer, oldest first. `base_f32` == source + these layers.
    layer_snapshots: Vec<AdjustParams>,

    // Full-resolution tile shown when zoomed in past the preview's resolution
    tile: Option<(TileKey, egui::TextureHandle)>,
    pending_tile: Option<TileKey>,
    canvas_img_rect: Option<egui::Rect>,
    canvas_rect: Option<egui::Rect>,
    /// Keep the image fitted to the canvas (window/panel resizes) until the user zooms or pans.
    auto_fit: bool,

    // UI state & services
    applied_dark: Option<bool>,
    show_shortcuts: bool,
    show_settings: bool,
    save_dialog: Option<String>,
    commands: Vec<Command>,
    toasts: ToastManager,
    exporter: BackgroundExporter,
    worker: RenderWorker,
    auto_saver: AutoSaveManager,
}

impl AdjustApp {
    fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
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
            params: AdjustParams::default(),
            presets: get_factory_presets(),
            user_presets: user_presets::load_all(),
            collections: get_collections(),
            thumbs: ThumbnailCache::new(cc.egui_ctx.clone()),
            library: Library::Collection(adjust_core::preset::ADJUST4),
            preset_search: String::new(),
            selected_preset_name: NO_PRESET.to_string(),
            view_mode: ViewMode::Split,
            info_tab: InfoTab::Histogram,
            curve_channel: CurveChannel::Rgb,
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
            editing_gesture: false,
            layer_snapshots: Vec::new(),
            tile: None,
            pending_tile: None,
            canvas_img_rect: None,
            canvas_rect: None,
            auto_fit: true,
            applied_dark: None,
            show_shortcuts: false,
            show_settings: false,
            save_dialog: None,
            commands: Vec::new(),
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
                }
            }
        }
        app
    }

    /// Re-renders every preset thumbnail for the current base image, starting with
    /// the collection on screen.
    fn rebuild_thumbnails(&mut self) {
        let Some(preview) = self.preview_base.clone() else {
            return;
        };
        let first: Vec<String> = presets_for(
            &self.library,
            &self.collections,
            &self.presets,
            &self.user_presets,
            &self.config.favorite_presets,
        )
        .iter()
        .map(|p| p.name.clone())
        .collect();
        let mut items: Vec<(String, AdjustParams)> = Vec::new();
        for name in &first {
            if let Some(p) = self.presets.iter().chain(&self.user_presets).find(|p| &p.name == name) {
                items.push((p.name.clone(), p.params.clone()));
            }
        }
        for p in self.presets.iter().chain(&self.user_presets) {
            if !first.contains(&p.name) {
                items.push((p.name.clone(), p.params.clone()));
            }
        }
        self.thumbs.rebuild(&preview, items);
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
        self.auto_fit = true;
        self.is_dirty = true;
        self.histogram = None;
        self.rebuild_thumbnails();

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
            .add_filter("Images", &["png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp"])
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
                .add_filter("WebP Image", &["webp"])
                .set_file_name(self.suggested_export_name())
                .save_file()
            {
                self.exporter.queue_export(ExportJob {
                    save_path: path.clone(),
                    base: base.clone(),
                    params: self.params.clone(),
                });
                self.toasts.info(format!(
                    "Exporting {}…",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        } else {
            self.toasts.warn("No image available to export");
        }
    }

    /// "photo.jpg" -> "photo-edited.png" (keeps 16-bit/alpha by default).
    fn suggested_export_name(&self) -> String {
        let stem = self
            .image_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "image".into());
        format!("{stem}-edited.png")
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
                    self.rebuild_thumbnails();
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

        self.selected_preset_name = NO_PRESET.to_string();
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
            self.selected_preset_name = "Restored layer".to_string();
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
        self.selected_preset_name = NO_PRESET.to_string();
        if !self.layer_snapshots.is_empty() {
            self.layer_snapshots.clear();
            self.rebake_layers();
        }
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.is_dirty = true;
        self.toasts.info("Reset all adjustments");
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
        let mut commands = Vec::new();
        ctx.input(|i| {
            let cmd = i.modifiers.command;
            if i.key_pressed(egui::Key::Tab) {
                let show = !(self.config.show_sidebar || self.config.show_inspector);
                self.config.show_sidebar = show;
                self.config.show_inspector = show;
            }
            if i.key_pressed(egui::Key::Questionmark) || (i.modifiers.shift && i.key_pressed(egui::Key::Slash)) {
                self.show_shortcuts = !self.show_shortcuts;
            }
            if !cmd {
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
                    self.auto_fit = true;
                }
                if i.key_pressed(egui::Key::Num0) {
                    self.zoom = 1.0;
                }
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    self.zoom = (self.zoom * 1.25).min(16.0);
                }
                if i.key_pressed(egui::Key::Minus) {
                    self.zoom = (self.zoom / 1.25).max(0.01);
                }
            }
            if cmd && i.key_pressed(egui::Key::Z) {
                commands.push(if i.modifiers.shift { Command::Redo } else { Command::Undo });
            }
            if cmd && i.key_pressed(egui::Key::O) {
                commands.push(Command::Open);
            }
            if cmd && i.key_pressed(egui::Key::E) {
                commands.push(Command::Export);
            }
            if cmd && i.key_pressed(egui::Key::S) {
                commands.push(Command::SavePreset);
            }
            if cmd && i.key_pressed(egui::Key::Comma) {
                self.show_settings = true;
            }
            // egui-winit consumes the Ctrl+V *press* (and emits nothing for image clipboards),
            // so react to the key release instead and read the clipboard ourselves.
            if i.events.iter().any(|e| {
                matches!(e, egui::Event::Key { key: egui::Key::V, pressed: false, modifiers, .. } if modifiers.command)
            }) {
                commands.push(Command::Paste);
            }
        });
        self.commands.extend(commands);
    }

    fn run_commands(&mut self) {
        for command in std::mem::take(&mut self.commands) {
            match command {
                Command::Open => self.open_image_dialog(),
                Command::Export => self.queue_export_dialog(),
                Command::Paste => self.paste_image_from_clipboard(),
                Command::SavePreset => {
                    let suggested = if self.selected_preset_name == NO_PRESET {
                        String::new()
                    } else {
                        format!("{} (edited)", self.selected_preset_name)
                    };
                    self.save_dialog = Some(suggested);
                }
                Command::Undo => self.undo(),
                Command::Redo => self.redo(),
            }
        }
    }

    fn apply_preset(&mut self, preset: Preset) {
        self.push_undo();
        self.params = preset.params;
        self.selected_preset_name = preset.name;
        self.is_dirty = true;
    }

    fn handle_sidebar(&mut self, action: SidebarAction) {
        match action {
            SidebarAction::Apply(preset) => self.apply_preset(preset),
            SidebarAction::SavePreset => self.commands.push(Command::SavePreset),
            SidebarAction::ToggleFavorite(name) => {
                if !self.config.favorite_presets.remove(&name) {
                    self.config.favorite_presets.insert(name);
                }
                self.config.save();
            }
            SidebarAction::ImportPreset => {
                if let Some(path) = rfd::FileDialog::new().add_filter("LimeStone Preset", &["json"]).pick_file() {
                    match user_presets::import(&path) {
                        Ok(preset) => self.add_user_preset(preset, "Imported"),
                        Err(e) => self.toasts.error(e),
                    }
                }
            }
            SidebarAction::ExportPreset(preset) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("LimeStone Preset", &["json"])
                    .set_file_name(format!("{}.json", preset.name))
                    .save_file()
                {
                    match user_presets::export(&preset, &path) {
                        Ok(()) => self.toasts.success(format!("Exported “{}”", preset.name)),
                        Err(e) => self.toasts.error(format!("Export failed: {e}")),
                    }
                }
            }
            SidebarAction::DeletePreset(name) => match user_presets::delete(&name) {
                Ok(()) => {
                    self.user_presets.retain(|p| p.name != name);
                    self.config.favorite_presets.remove(&name);
                    self.toasts.info(format!("Deleted “{name}”"));
                }
                Err(e) => self.toasts.error(format!("Couldn't delete: {e}")),
            },
        }
    }

    fn add_user_preset(&mut self, preset: Preset, verb: &str) {
        self.user_presets.retain(|p| p.name != preset.name);
        self.user_presets.push(preset.clone());
        self.user_presets.sort_by_key(|p| p.name.to_lowercase());
        if let Some(preview) = self.preview_base.clone() {
            self.thumbs.render_one(&preview, preset.name.clone(), preset.params.clone());
        }
        self.library = Library::Mine;
        self.toasts.success(format!("{verb} “{}”", preset.name));
    }

    fn handle_toolbar(&mut self, ctx: &egui::Context, action: ToolbarAction) {
        use ToolbarAction::*;
        match action {
            ToggleSidebar => self.config.show_sidebar = !self.config.show_sidebar,
            ToggleInspector => self.config.show_inspector = !self.config.show_inspector,
            Open => self.commands.push(Command::Open),
            Paste => self.commands.push(Command::Paste),
            Undo => self.commands.push(Command::Undo),
            Redo => self.commands.push(Command::Redo),
            ZoomIn => self.zoom = (self.zoom * 1.25).min(16.0),
            ZoomOut => self.zoom = (self.zoom / 1.25).max(0.01),
            Fit => {
                self.zoom = self.fit_zoom(ctx);
                self.pan = egui::Vec2::ZERO;
                self.auto_fit = true;
            }
            ActualSize => self.zoom = 1.0,
            Export => self.commands.push(Command::Export),
            Settings => self.show_settings = true,
            Shortcuts => self.show_shortcuts = true,
        }
    }

    fn handle_inspector(&mut self, action: InspectorAction, before: &AdjustParams) {
        match action {
            InspectorAction::Changed => {
                // One undo step per slider gesture, not per frame of dragging.
                if !self.editing_gesture {
                    if self.undo_stack.len() >= 100 {
                        self.undo_stack.remove(0);
                    }
                    self.undo_stack.push(before.clone());
                    self.redo_stack.clear();
                    self.editing_gesture = true;
                }
                if self.selected_preset_name != NO_PRESET && !self.selected_preset_name.ends_with(" (edited)") {
                    self.selected_preset_name = format!("{} (edited)", self.selected_preset_name);
                }
                self.is_dirty = true;
            }
            InspectorAction::Apply => self.apply_layer(),
            InspectorAction::UndoLayer => self.undo_layer(),
            InspectorAction::ResetAll => {
                if self.params != AdjustParams::default() || !self.layer_snapshots.is_empty() {
                    self.push_undo();
                    self.cancel_all_layers();
                }
            }
        }
    }

    fn render_dialogs(&mut self, ctx: &egui::Context) {
        if self.show_settings {
            let (open, theme_changed) = dialogs::settings(ctx, &mut self.config);
            self.show_settings = open;
            if theme_changed {
                self.applied_dark = None; // re-apply next frame
            }
        }
        if self.show_shortcuts {
            self.show_shortcuts = dialogs::shortcuts(ctx);
        }
        if let Some(mut name) = self.save_dialog.take() {
            let exists = self.user_presets.iter().any(|p| p.name.eq_ignore_ascii_case(name.trim()));
            match dialogs::save_preset(ctx, &mut name, exists) {
                SaveOutcome::Open => self.save_dialog = Some(name),
                SaveOutcome::Cancel => {}
                SaveOutcome::Save => match user_presets::save(&name, &self.params) {
                    Ok(preset) => {
                        self.selected_preset_name = preset.name.clone();
                        self.add_user_preset(preset, "Saved");
                    }
                    Err(e) => {
                        self.toasts.error(format!("Couldn't save preset: {e}"));
                        self.save_dialog = Some(name);
                    }
                },
            }
        }
    }

    fn window_title(&self) -> (String, String) {
        let Some(base) = &self.base_f32 else {
            return ("LimeStone".into(), "No image open".into());
        };
        let title = self
            .image_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());
        let mut sub = format!("{} × {}", base.width, base.height);
        if base.high_bit_depth {
            sub.push_str(" · 16-bit");
        }
        sub.push_str(&format!(" · {}", self.selected_preset_name));
        (title, sub)
    }
}

impl eframe::App for AdjustApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // A clean exit means there is nothing to recover next launch.
        self.auto_saver.clear_recovery();
        self.config.save();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Follow the OS appearance and apply theme changes.
        let dark = is_dark(ctx, self.config.theme_mode);
        if self.applied_dark != Some(dark) {
            apply_theme(ctx, self.config.theme_mode, self.config.accent, self.config.ui_scale);
            self.applied_dark = Some(dark);
        }
        let p = pal_ctx(ctx);

        // Background results
        for res in self.exporter.poll_results() {
            match res {
                ExportResult::Success { save_path, elapsed_ms, width, height } => self.toasts.success(format!(
                    "Exported {} ({}×{}) in {:.1}s",
                    save_path.file_name().unwrap_or_default().to_string_lossy(),
                    width,
                    height,
                    elapsed_ms as f32 / 1000.0
                )),
                ExportResult::Failed { save_path, error } => self.toasts.error(format!(
                    "Couldn't export {}: {}",
                    save_path.file_name().unwrap_or_default().to_string_lossy(),
                    error
                )),
            }
        }
        self.poll_worker(ctx);
        self.thumbs.poll(ctx);

        let view_before = (self.zoom, self.pan, self.auto_fit);
        self.handle_keyboard_shortcuts(ctx);
        self.run_commands();
        if !ctx.input(|i| i.pointer.any_down()) {
            self.editing_gesture = false;
        }
        if self.is_dirty {
            self.trigger_reprocess();
        }

        // ---- Toolbar ---------------------------------------------------------
        let (title, subtitle) = self.window_title();
        let toolbar_actions = egui::TopBottomPanel::top("toolbar")
            .exact_height(HEIGHT_TOOLBAR)
            .frame(
                egui::Frame::new()
                    .fill(p.toolbar)
                    .stroke(egui::Stroke::new(1.0_f32, p.separator)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                toolbar(
                    ui,
                    ToolbarState {
                        view_mode: &mut self.view_mode,
                        title: &title,
                        subtitle: &subtitle,
                        zoom: self.zoom,
                        can_undo: !self.undo_stack.is_empty(),
                        can_redo: !self.redo_stack.is_empty(),
                        has_image: self.base_f32.is_some(),
                        show_sidebar: self.config.show_sidebar,
                        show_inspector: self.config.show_inspector,
                        busy: self.is_rendering || self.baking || self.exporter.active_jobs() > 0,
                    },
                )
            })
            .inner;
        for action in toolbar_actions {
            self.handle_toolbar(ctx, action);
        }

        // ---- Sidebar ---------------------------------------------------------
        if self.config.show_sidebar {
            let old_library = self.library.clone();
            let actions = egui::SidePanel::left("sidebar")
                .default_width(WIDTH_SIDEBAR)
                .width_range(250.0..=440.0)
                .frame(egui::Frame::new().fill(p.sidebar).inner_margin(egui::Margin::symmetric(10, 0)))
                .show_separator_line(true)
                .show(ctx, |ui| {
                    sidebar(
                        ui,
                        SidebarState {
                            library: &mut self.library,
                            search: &mut self.preset_search,
                            collections: &self.collections,
                            factory: &self.presets,
                            user: &self.user_presets,
                            favorites: &self.config.favorite_presets,
                            selected_preset: &self.selected_preset_name,
                            thumbs: &self.thumbs,
                        },
                    )
                })
                .inner;
            for action in actions {
                self.handle_sidebar(action);
            }
            if self.library != old_library {
                self.preset_search.clear();
            }
        }

        // ---- Inspector ---------------------------------------------------------
        if self.config.show_inspector {
            let before = self.params.clone();
            let display_size = self.image_px_size().map(|px| px * self.zoom / ctx.pixels_per_point());
            let actions = egui::SidePanel::right("inspector")
                .default_width(WIDTH_INSPECTOR)
                .width_range(280.0..=460.0)
                .frame(egui::Frame::new().fill(p.sidebar).inner_margin(egui::Margin::symmetric(12, 0)))
                .show(ctx, |ui| {
                    inspector(
                        ui,
                        InspectorState {
                            params: &mut self.params,
                            info_tab: &mut self.info_tab,
                            curve_channel: &mut self.curve_channel,
                            histogram: self.histogram.as_ref(),
                            nav_texture: self.nav_texture.as_ref(),
                            proc_texture: self.proc_texture.as_ref(),
                            pan: &mut self.pan,
                            display_size,
                            canvas_size: self.canvas_size,
                            layer_count: self.layer_snapshots.len(),
                            baking: self.baking,
                            has_image: self.base_f32.is_some(),
                            preset_name: &self.selected_preset_name,
                        },
                    )
                })
                .inner;
            for action in actions {
                self.handle_inspector(action, &before);
            }
        }

        // ---- Canvas ------------------------------------------------------------
        let image_px = self.image_px_size().unwrap_or(egui::Vec2::ZERO);
        let tile = self.visible_tile();
        let canvas_resp = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.canvas))
            .show(ctx, |ui| {
                canvas(
                    ui,
                    CanvasState {
                        view_mode: self.view_mode,
                        split_ratio: &mut self.split_ratio,
                        zoom: &mut self.zoom,
                        pan: &mut self.pan,
                        image_px,
                        proc_texture: self.proc_texture.as_ref(),
                        orig_texture: self.orig_texture.as_ref(),
                        tile: tile.as_ref().map(|(t, r)| (t, *r)),
                    },
                )
            })
            .inner;
        self.canvas_size = canvas_resp.canvas_size;
        self.canvas_rect = Some(canvas_resp.canvas_rect);
        self.canvas_img_rect = canvas_resp.img_rect;
        if let Some(path) = canvas_resp.dropped_path {
            self.open_image_path(path);
        }
        if canvas_resp.open_clicked {
            self.commands.push(Command::Open);
        }

        // Any zoom/pan change this frame that didn't come from Fit turns auto-fit off.
        if self.auto_fit == view_before.2 && (self.zoom != view_before.0 || self.pan != view_before.1) {
            self.auto_fit = false;
        }
        if self.auto_fit && self.base_f32.is_some() {
            let fit = self.fit_zoom(ctx);
            if (self.zoom - fit).abs() > 1e-4 || self.pan != egui::Vec2::ZERO {
                self.zoom = fit;
                self.pan = egui::Vec2::ZERO;
                ctx.request_repaint();
            }
        }

        self.update_tile_request(ctx);
        self.render_dialogs(ctx);
        self.toasts.show(ctx, canvas_resp.canvas_rect);
    }
}
