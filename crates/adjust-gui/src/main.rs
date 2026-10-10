mod curve_widget;
mod icons;
mod panels;
mod theme;
mod widgets;

use adjust_core::{
    compute_histogram, get_factory_presets, process_image, AdjustParams, HistogramData, Preset,
    RgbImageF32,
};
use eframe::egui;
use icons::*;
use image::RgbImage;
use panels::adjustments::render_adjustments_accordion;
use panels::canvas::render_central_canvas;
use panels::inspector::{render_inspector_panel, InspectorTab};
use panels::presets::render_presets_sidebar;
use panels::statusbar::render_bottom_statusbar;
use panels::toolbar::{render_top_toolbar, ViewMode};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;
use theme::*;
use widgets::toast::ToastManager;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([1100.0, 700.0])
            .with_title("Limestone Pro — Studio Image Enhancement"),
        ..Default::default()
    };

    eframe::run_native(
        "Limestone Pro",
        native_options,
        Box::new(|cc| {
            // Setup font with Tabler Icons
            setup_tabler_fonts(&cc.egui_ctx);
            // Apply Modern Dark Theme system
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(AdjustApp::new(cc)))
        }),
    )
}

struct AdjustApp {
    image_path: Option<PathBuf>,
    original_f32: Option<RgbImageF32>,
    preview_orig_f32: Option<RgbImageF32>,
    preview_proc_f32: Option<RgbImageF32>,

    orig_texture: Option<egui::TextureHandle>,
    proc_texture: Option<egui::TextureHandle>,
    nav_texture: Option<egui::TextureHandle>,

    params: AdjustParams,
    presets: Vec<Preset>,
    selected_preset_name: String,
    selected_collection: String,
    preset_search: String,
    favorite_presets: HashSet<String>,

    view_mode: ViewMode,
    inspector_tab: InspectorTab,
    curve_channel: crate::curve_widget::CurveChannel,
    split_ratio: f32, // 0.0 .. 1.0
    zoom: f32,
    pan: egui::Vec2,

    histogram: Option<HistogramData>,
    is_dirty: bool,
    is_rendering: bool,
    last_process_time: f32,

    // Undo / Redo & effect stacking
    undo_stack: Vec<AdjustParams>,
    redo_stack: Vec<AdjustParams>,
    layer_snapshots: Vec<AdjustParams>,

    // UI state
    show_sidebars: bool,
    show_shortcuts_modal: bool,
    toasts: ToastManager,
}

impl AdjustApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let presets = get_factory_presets();
        let default_preset = presets
            .iter()
            .find(|p| p.name == "Mild Contrast Pop")
            .cloned()
            .unwrap_or_else(|| presets[0].clone());

        let mut app = Self {
            image_path: None,
            original_f32: None,
            preview_orig_f32: None,
            preview_proc_f32: None,
            orig_texture: None,
            proc_texture: None,
            nav_texture: None,
            params: default_preset.params.clone(),
            presets,
            selected_preset_name: default_preset.name,
            selected_collection: "All Collections".to_string(),
            preset_search: String::new(),
            favorite_presets: HashSet::new(),
            view_mode: ViewMode::Split,
            inspector_tab: InspectorTab::Navigate,
            curve_channel: crate::curve_widget::CurveChannel::Rgb,
            split_ratio: 0.5,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            histogram: None,
            is_dirty: false,
            is_rendering: false,
            last_process_time: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            layer_snapshots: Vec::new(),
            show_sidebars: true,
            show_shortcuts_modal: false,
            toasts: ToastManager::new(),
        };

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

                // Rich landscape simulation with sky gradient, hills, shadows, and textures
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

        self.set_image(img, PathBuf::from("sample_landscape.png"));
    }

    fn set_image(&mut self, img: RgbImage, path: PathBuf) {
        let f32_full = RgbImageF32::from_rgb8(&img);
        let preview = f32_full.downsample(960);

        self.image_path = Some(path.clone());
        self.original_f32 = Some(f32_full);
        self.preview_orig_f32 = Some(preview);
        self.orig_texture = None;
        self.proc_texture = None;
        self.nav_texture = None;
        self.is_dirty = true;
        self.toasts.success(format!(
            "Opened {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    fn open_image_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp"],
            )
            .pick_file()
        {
            if let Ok(img) = image::open(&path) {
                self.set_image(img.to_rgb8(), path);
            } else {
                self.toasts.error("Failed to decode image file");
            }
        }
    }

    fn export_image_dialog(&mut self) {
        if let Some(ref full_orig) = self.original_f32 {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("TIFF Image", &["tif", "tiff"])
                .save_file()
            {
                let start = Instant::now();
                let full_proc = process_image(full_orig, &self.params);
                let rgb8 = full_proc.to_rgb8();
                if rgb8.save(&path).is_ok() {
                    let elapsed = start.elapsed().as_millis();
                    self.toasts.success(format!(
                        "Exported successfully in {} ms to {}",
                        elapsed,
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                } else {
                    self.toasts.error("Failed to save exported file");
                }
            }
        } else {
            self.toasts.warn("No image available to export");
        }
    }

    fn trigger_reprocess(&mut self, ctx: &egui::Context) {
        self.is_rendering = true;
        if let Some(ref orig) = self.preview_orig_f32 {
            let start = Instant::now();
            let proc = process_image(orig, &self.params);
            self.last_process_time = start.elapsed().as_secs_f32() * 1000.0;

            self.histogram = Some(compute_histogram(&proc));

            let rgb8 = proc.to_rgb8();
            let color_image = egui::ColorImage::from_rgb(
                [rgb8.width() as usize, rgb8.height() as usize],
                rgb8.as_raw(),
            );

            self.proc_texture = Some(ctx.load_texture(
                "processed_view",
                color_image,
                egui::TextureOptions::LINEAR,
            ));
            self.preview_proc_f32 = Some(proc);
        }

        if self.orig_texture.is_none() {
            if let Some(ref orig) = self.preview_orig_f32 {
                let rgb8 = orig.to_rgb8();
                let color_image = egui::ColorImage::from_rgb(
                    [rgb8.width() as usize, rgb8.height() as usize],
                    rgb8.as_raw(),
                );
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

        self.is_dirty = false;
        self.is_rendering = false;
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Tab: Toggle panels (focus mode)
            if i.key_pressed(egui::Key::Tab) {
                self.show_sidebars = !self.show_sidebars;
            }

            // ?: Toggle shortcuts overlay
            if i.key_pressed(egui::Key::Questionmark)
                || (i.modifiers.shift && i.key_pressed(egui::Key::Slash))
            {
                self.show_shortcuts_modal = !self.show_shortcuts_modal;
            }

            // View modes: 1, 2, 3
            if i.key_pressed(egui::Key::Num1) {
                self.view_mode = ViewMode::Single;
            }
            if i.key_pressed(egui::Key::Num2) {
                self.view_mode = ViewMode::Split;
            }
            if i.key_pressed(egui::Key::Num3) {
                self.view_mode = ViewMode::SideBySide;
            }

            // Zoom: F (Fit), 0 (100%)
            if i.key_pressed(egui::Key::F) {
                self.zoom = 1.0;
                self.pan = egui::Vec2::ZERO;
            }
            if i.key_pressed(egui::Key::Num0) {
                self.zoom = 1.0;
            }

            // Undo / Redo
            if i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z) {
                // Redo
                self.redo();
            } else if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                // Undo
                self.undo();
            }

            // Open & Export
            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                self.open_image_dialog();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::E) {
                self.export_image_dialog();
            }
        });
    }

    fn render_shortcuts_modal(&mut self, ctx: &egui::Context) {
        if !self.show_shortcuts_modal {
            return;
        }

        egui::Window::new(format!("{} Keyboard Shortcuts", ICON_KEYBOARD))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(440.0, 360.0))
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
                    ("Ctrl+E / Cmd+E", "Export full-resolution image"),
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
                                    .color(ACCENT),
                            ),
                        );
                        ui.label(egui::RichText::new(desc).size(12.0).color(TEXT_PRIMARY));
                    });
                    ui.add_space(SPACE_4);
                }

                ui.add_space(SPACE_16);
                ui.vertical_centered(|ui| {
                    if ui.button("Close").clicked() {
                        self.show_shortcuts_modal = false;
                    }
                });
            });
    }
}

impl eframe::App for AdjustApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Global Shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Reprocess DSP if dirty
        if self.is_dirty {
            self.trigger_reprocess(ctx);
        }

        // ==========================================
        // 1. TOP HEADER TOOLBAR (48px)
        // ==========================================
        egui::TopBottomPanel::top("top_header_panel")
            .exact_height(HEIGHT_TOOLBAR)
            .show(ctx, |ui| {
                let tb_resp = render_top_toolbar(
                    ui,
                    &mut self.view_mode,
                    &mut self.zoom,
                    !self.undo_stack.is_empty(),
                    !self.redo_stack.is_empty(),
                    self.last_process_time,
                );

                if tb_resp.open_clicked {
                    self.open_image_dialog();
                }
                if tb_resp.export_clicked {
                    self.export_image_dialog();
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
                    self.zoom = 1.0;
                    self.pan = egui::Vec2::ZERO;
                }
                if tb_resp.zoom_100_clicked {
                    self.zoom = 1.0;
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
                    .original_f32
                    .as_ref()
                    .map(|img| (img.width, img.height));
                render_bottom_statusbar(
                    ui,
                    dimensions,
                    self.zoom,
                    self.is_rendering,
                    self.last_process_time,
                    self.original_f32.is_some(),
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
                        &mut self.favorite_presets,
                        self.nav_texture.as_ref(),
                    );

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
                            if let Ok(json) = custom.to_json() {
                                if std::fs::write(&path, json).is_ok() {
                                    self.toasts.success("Custom preset saved");
                                }
                            }
                        }
                    }

                    if p_resp.load_custom_clicked {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON Preset", &["json"])
                            .pick_file()
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(custom) = Preset::from_json(&content) {
                                    self.push_undo();
                                    self.params = custom.params;
                                    self.selected_preset_name = custom.name.clone();
                                    self.is_dirty = true;
                                    self.toasts.success(format!("Loaded preset {}", custom.name));
                                } else {
                                    self.toasts.error("Invalid preset file format");
                                }
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
                    render_inspector_panel(
                        ui,
                        &mut self.inspector_tab,
                        self.nav_texture.as_ref(),
                        self.proc_texture.as_ref(),
                        self.histogram.as_ref(),
                        &mut self.pan,
                        self.zoom,
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
                                self.params = AdjustParams::default();
                                self.selected_preset_name = "Neutral".to_string();
                                self.layer_snapshots.clear();
                                self.is_dirty = true;
                                self.toasts.warn("Canceled all adjustments");
                            }

                            if adj_resp.apply_clicked {
                                // Effect stacking workflow (#10 feature)
                                if let Some(ref proc) = self.preview_proc_f32 {
                                    self.layer_snapshots.push(self.params.clone());
                                    self.preview_orig_f32 = Some(proc.clone());
                                    if let Some(ref full_orig) = self.original_f32 {
                                        let full_proc = process_image(full_orig, &self.params);
                                        self.original_f32 = Some(full_proc);
                                    }
                                    self.params = AdjustParams::default();
                                    self.selected_preset_name = "Neutral".to_string();
                                    self.orig_texture = None;
                                    self.is_dirty = true;
                                    self.toasts.success("Baked effect layer into base!");
                                }
                            }

                            if adj_resp.undo_layer_clicked {
                                if !self.layer_snapshots.is_empty() {
                                    self.layer_snapshots.pop();
                                    self.is_dirty = true;
                                    self.toasts.info("Popped effect layer");
                                }
                            }
                        });
                });
        }

        // ==========================================
        // 5. CENTER CANVAS VIEWPORT
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            let canvas_resp = render_central_canvas(
                ui,
                self.view_mode,
                &mut self.split_ratio,
                &mut self.zoom,
                &mut self.pan,
                self.proc_texture.as_ref(),
                self.orig_texture.as_ref(),
                self.original_f32.is_some(),
            );

            if let Some(path) = canvas_resp.dropped_path {
                if let Ok(img) = image::open(&path) {
                    self.set_image(img.to_rgb8(), path);
                } else {
                    self.toasts.error("Failed to load dropped image");
                }
            }

            if canvas_resp.open_clicked {
                self.open_image_dialog();
            }
        });

        // Overlay: Keyboard Shortcuts Cheat Sheet
        self.render_shortcuts_modal(ctx);

        // Overlay: In-App Toast Notifications
        self.toasts.show(ctx);
    }
}
