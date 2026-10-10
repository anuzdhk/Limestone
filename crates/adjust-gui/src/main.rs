mod curve_widget;
mod icons;

use adjust_core::{
    compute_histogram, get_factory_presets, process_image, AdjustParams, HistogramData, Preset,
    RgbImageF32,
};
use curve_widget::render_curve_editor;
use eframe::egui;
use icons::*;
use image::RgbImage;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([1100.0, 680.0])
            .with_title("Topaz Adjust 4.0 — Professional DSP Image Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Topaz Adjust 4.0",
        native_options,
        Box::new(|cc| {
            // Install Tabler Icons font
            setup_tabler_fonts(&cc.egui_ctx);
            // Apply Senior Product Designer Dark Theme
            apply_modern_dark_theme(&cc.egui_ctx);
            Ok(Box::new(AdjustApp::new(cc)))
        }),
    )
}

fn apply_modern_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Palette: Deep obsidian, charcoal panels, subtle graphite borders
    visuals.override_text_color = Some(egui::Color32::from_rgb(240, 242, 245));
    visuals.panel_fill = egui::Color32::from_rgb(22, 23, 28);
    visuals.window_fill = egui::Color32::from_rgb(22, 23, 28);
    visuals.extreme_bg_color = egui::Color32::from_rgb(15, 16, 20);

    // Widgets inactive
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 34, 42);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(48, 51, 62));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(215, 220, 230));

    // Widgets hovered
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(44, 47, 58);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 210, 255));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    // Widgets active
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(14, 115, 160);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 210, 255));
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    // Selection
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 180, 240, 60);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 210, 255));

    visuals.window_corner_radius = egui::CornerRadius::same(8);

    ctx.set_visuals(visuals);
}

#[derive(PartialEq, Clone, Copy)]
enum ViewMode {
    Single,
    Split,
    SideBySide,
}

#[derive(PartialEq, Clone, Copy)]
enum InspectorTab {
    Navigate,
    Loupe,
    Histogram,
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
    favorite_presets: std::collections::HashSet<String>,

    view_mode: ViewMode,
    inspector_tab: InspectorTab,
    split_ratio: f32, // 0.0 .. 1.0
    zoom: f32,
    pan: egui::Vec2,

    histogram: Option<HistogramData>,
    is_dirty: bool,
    last_process_time: f32,
    snapshots: Vec<AdjustParams>,
}

impl AdjustApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let presets = get_factory_presets();
        let default_preset = presets.iter().find(|p| p.name == "Mild Contrast Pop").cloned().unwrap_or_else(|| presets[0].clone());

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
            favorite_presets: std::collections::HashSet::new(),
            view_mode: ViewMode::Split,
            inspector_tab: InspectorTab::Navigate,
            split_ratio: 0.5,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            histogram: None,
            is_dirty: false,
            last_process_time: 0.0,
            snapshots: Vec::new(),
        };

        app.load_default_sample();
        app
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
        // High-responsiveness interactive preview downsampled to 960px for immediate slider & curve feedback
        let preview = f32_full.downsample(960);

        self.image_path = Some(path);
        self.original_f32 = Some(f32_full);
        self.preview_orig_f32 = Some(preview);
        self.orig_texture = None;
        self.proc_texture = None;
        self.nav_texture = None;
        self.is_dirty = true;
    }

    fn trigger_reprocess(&mut self, ctx: &egui::Context) {
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

            self.proc_texture = Some(ctx.load_texture("processed_view", color_image, egui::TextureOptions::LINEAR));
            self.preview_proc_f32 = Some(proc);
        }

        if self.orig_texture.is_none() {
            if let Some(ref orig) = self.preview_orig_f32 {
                let rgb8 = orig.to_rgb8();
                let color_image = egui::ColorImage::from_rgb(
                    [rgb8.width() as usize, rgb8.height() as usize],
                    rgb8.as_raw(),
                );
                self.orig_texture = Some(ctx.load_texture("original_view", color_image.clone(), egui::TextureOptions::LINEAR));
                // Also create nav thumbnail texture
                self.nav_texture = Some(ctx.load_texture("nav_view", color_image, egui::TextureOptions::LINEAR));
            }
        }

        self.is_dirty = false;
    }
}

impl eframe::App for AdjustApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_dirty {
            self.trigger_reprocess(ctx);
        }

        // ==========================================
        // 1. TOP HEADER TOOLBAR (ANTIGRAVITY / TOPAZ STYLE)
        // ==========================================
        egui::TopBottomPanel::top("top_header")
            .exact_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

                    // Antigravity / Topaz Brand Title
                    ui.label(egui::RichText::new("TOPAZ ADJUST").strong().size(16.0).color(egui::Color32::WHITE));
                    ui.label(egui::RichText::new("4.0").strong().size(13.0).color(egui::Color32::from_rgb(0, 210, 255)));

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Action Buttons with Tabler Icons
                    if ui.button(format!("{} Open", ICON_FOLDER_OPEN)).on_hover_text("Open an image from disk").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp"])
                            .pick_file()
                        {
                            if let Ok(img) = image::open(&path) {
                                self.set_image(img.to_rgb8(), path);
                            }
                        }
                    }

                    if ui.button(format!("{} Export", ICON_DEVICE_FLOPPY)).on_hover_text("Export processed full-resolution image").clicked() {
                        if let Some(ref full_orig) = self.original_f32 {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PNG Image", &["png"])
                                .add_filter("JPEG Image", &["jpg", "jpeg"])
                                .add_filter("TIFF Image", &["tif", "tiff"])
                                .save_file()
                            {
                                let full_proc = process_image(full_orig, &self.params);
                                let rgb8 = full_proc.to_rgb8();
                                let _ = rgb8.save(&path);
                            }
                        }
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // View Modes Pill Selector
                    ui.label(egui::RichText::new("View:").color(egui::Color32::from_rgb(160, 165, 180)));
                    if ui.selectable_label(self.view_mode == ViewMode::Single, format!("{} Single", ICON_PHOTO)).clicked() {
                        self.view_mode = ViewMode::Single;
                    }
                    if ui.selectable_label(self.view_mode == ViewMode::Split, format!("{} Split Wipe", ICON_LAYOUT_COLUMNS)).clicked() {
                        self.view_mode = ViewMode::Split;
                    }
                    if ui.selectable_label(self.view_mode == ViewMode::SideBySide, format!("{} Side-by-Side", ICON_LAYOUT_COLUMNS)).clicked() {
                        self.view_mode = ViewMode::SideBySide;
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Zoom Controls
                    if ui.button("Fit").clicked() {
                        self.zoom = 1.0;
                        self.pan = egui::Vec2::ZERO;
                    }
                    if ui.button("100%").clicked() {
                        self.zoom = 1.0;
                    }
                    if ui.button("200%").clicked() {
                        self.zoom = 2.0;
                    }

                    // Right Aligned Latency & Reset
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        if self.last_process_time > 0.0 {
                            let badge = format!("⚡ {:.1} ms", self.last_process_time);
                            ui.label(egui::RichText::new(badge).size(12.0).color(egui::Color32::from_rgb(0, 210, 255)));
                        }

                        if ui.button(format!("{} Reset All", ICON_REFRESH)).on_hover_text("Revert all sliders and curves to default").clicked() {
                            self.params = AdjustParams::default();
                            self.selected_preset_name = "Neutral".to_string();
                            self.is_dirty = true;
                        }
                    });
                });
            });

        // ==========================================
        // 2. BOTTOM ACTION BAR (APPLY / OK / CANCEL)
        // ==========================================
        egui::TopBottomPanel::bottom("bottom_bar")
            .exact_height(44.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(format!("Preset: {}", self.selected_preset_name)).strong().size(12.5).color(egui::Color32::from_rgb(0, 210, 255)));

                    if !self.snapshots.is_empty() {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("({} Layer Stacked)", self.snapshots.len())).size(11.5).color(egui::Color32::from_rgb(160, 165, 180)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);

                        // OK / Export Full Image
                        if ui.button(format!("{} OK", ICON_CHECK)).on_hover_text("Export & save the full image").clicked() {
                            if let Some(ref full_orig) = self.original_f32 {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("PNG Image", &["png"])
                                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                                    .save_file()
                                {
                                    let full_proc = process_image(full_orig, &self.params);
                                    let _ = full_proc.to_rgb8().save(&path);
                                }
                            }
                        }

                        // CANCEL
                        if ui.button(format!("{} Cancel", ICON_X)).on_hover_text("Revert all changes and restore original").clicked() {
                            self.params = AdjustParams::default();
                            self.selected_preset_name = "Neutral".to_string();
                            self.snapshots.clear();
                            self.is_dirty = true;
                        }

                        // APPLY BUTTON (#10 feature: Stack multiple effects in one session!)
                        if ui.button(format!("{} Apply", ICON_APPLY)).on_hover_text("Bake current adjustment stack as a new baseline to apply additional effects on top!").clicked() {
                            if let Some(ref proc) = self.preview_proc_f32 {
                                self.snapshots.push(self.params.clone());
                                self.preview_orig_f32 = Some(proc.clone());
                                if let Some(ref full_orig) = self.original_f32 {
                                    let full_proc = process_image(full_orig, &self.params);
                                    self.original_f32 = Some(full_proc);
                                }
                                self.params = AdjustParams::default();
                                self.selected_preset_name = "Neutral".to_string();
                                self.orig_texture = None;
                                self.is_dirty = true;
                            }
                        }

                        // Snapshots Undo if stacked
                        if !self.snapshots.is_empty() {
                            if ui.button(format!("{} Undo Layer", ICON_HISTORY)).clicked() {
                                self.snapshots.pop();
                                self.is_dirty = true;
                            }
                        }
                    });
                });
            });

        // ==========================================
        // 3. LEFT SIDEBAR: COLLECTIONS & PRESETS EXPLORER
        // ==========================================
        egui::SidePanel::left("left_preset_panel")
            .resizable(true)
            .default_width(280.0)
            .width_range(240.0..=360.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Collections Header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("COLLECTIONS").strong().size(12.0).color(egui::Color32::from_rgb(160, 165, 180)));
                });
                ui.add_space(4.0);

                // Collections list
                let collection_names = [
                    "All Collections",
                    "Adjust 4 Preset List",
                    "Classic Collection",
                    "Vibrant Collection",
                    "HDR Collection",
                    "Stylized Collection",
                    "Toned Collection",
                    "Film Collection",
                    "Favorites",
                ];

                egui::ComboBox::from_id_salt("collections_dropdown")
                    .selected_text(&self.selected_collection)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for col in &collection_names {
                            ui.selectable_value(&mut self.selected_collection, col.to_string(), *col);
                        }
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // Preset Search Box
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(ICON_SEARCH).size(13.0).color(egui::Color32::from_rgb(160, 165, 180)));
                    ui.text_edit_singleline(&mut self.preset_search);
                });

                ui.add_space(6.0);
                ui.separator();

                // Presets Scroll View
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let query = self.preset_search.trim().to_lowercase();

                    let filtered_presets: Vec<Preset> = self.presets
                        .iter()
                        .filter(|p| {
                            if self.selected_collection == "Favorites" {
                                self.favorite_presets.contains(&p.name)
                            } else if self.selected_collection == "All Collections" || self.selected_collection == "Adjust 4 Preset List" {
                                true
                            } else {
                                p.category == self.selected_collection
                            }
                        })
                        .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query) || p.description.to_lowercase().contains(&query))
                        .cloned()
                        .collect();

                    for preset in filtered_presets {
                        let is_selected = self.selected_preset_name == preset.name;
                        let is_fav = self.favorite_presets.contains(&preset.name);

                        ui.horizontal(|ui| {
                            // Favorite Star Toggle
                            let star_icon = if is_fav { ICON_STAR_FILLED } else { ICON_STAR };
                            let star_color = if is_fav { egui::Color32::from_rgb(255, 200, 50) } else { egui::Color32::from_rgb(90, 95, 110) };
                            if ui.add(egui::Button::new(egui::RichText::new(star_icon).size(12.0).color(star_color)).frame(false)).clicked() {
                                if is_fav {
                                    self.favorite_presets.remove(&preset.name);
                                } else {
                                    self.favorite_presets.insert(preset.name.clone());
                                }
                            }

                            // Preset Name Label
                            let label_text = egui::RichText::new(&preset.name)
                                .size(13.0)
                                .color(if is_selected { egui::Color32::from_rgb(0, 210, 255) } else { egui::Color32::from_rgb(220, 225, 235) });

                            let resp = ui.selectable_label(is_selected, label_text);
                            if resp.clicked() {
                                self.params = preset.params.clone();
                                self.selected_preset_name = preset.name.clone();
                                self.is_dirty = true;
                            }
                            resp.on_hover_text(&preset.description);
                        });
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if ui.button("Save Custom Preset...").clicked() {
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
                                let _ = std::fs::write(&path, json);
                            }
                        }
                    }

                    if ui.button("Load Custom Preset...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON Preset", &["json"])
                            .pick_file()
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(custom) = Preset::from_json(&content) {
                                    self.params = custom.params;
                                    self.selected_preset_name = custom.name;
                                    self.is_dirty = true;
                                }
                            }
                        }
                    }
                });
            });

        // ==========================================
        // 4. RIGHT SIDEBAR: NAVIGATE / LOUPE / HISTOGRAM & ADJUSTMENTS
        // ==========================================
        egui::SidePanel::right("right_adjustment_panel")
            .resizable(true)
            .default_width(340.0)
            .width_range(300.0..=480.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);

                // Inspector Tabs: NAVIGATE | LOUPE | HISTOGRAM
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.inspector_tab, InspectorTab::Navigate, "NAVIGATE");
                    ui.selectable_value(&mut self.inspector_tab, InspectorTab::Loupe, "LOUPE");
                    ui.selectable_value(&mut self.inspector_tab, InspectorTab::Histogram, "HISTOGRAM");
                });
                ui.add_space(4.0);

                // Render Inspector Subview
                match self.inspector_tab {
                    InspectorTab::Navigate => {
                        if let Some(ref nav_tex) = self.nav_texture {
                            let nav_w = ui.available_width().max(100.0);
                            let nav_h = (nav_w * 0.65).min(130.0);
                            let (resp, painter) = ui.allocate_painter(egui::vec2(nav_w, nav_h), egui::Sense::hover());
                            painter.image(nav_tex.id(), resp.rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                            // Draw Navigator viewport crop box (cyan rectangle matching Topaz screenshot)
                            let crop_box = egui::Rect::from_min_size(
                                egui::pos2(resp.rect.min.x + resp.rect.width() * 0.25, resp.rect.min.y + 4.0),
                                egui::vec2(resp.rect.width() * 0.50, resp.rect.height() - 8.0),
                            );
                            painter.rect_stroke(crop_box, 0.0, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 210, 255)), egui::StrokeKind::Inside);
                        }
                    }
                    InspectorTab::Loupe => {
                        ui.label(egui::RichText::new("Loupe Zoom: 400% Target").size(12.0).color(egui::Color32::from_rgb(160, 165, 180)));
                        if let Some(ref proc_tex) = self.proc_texture {
                            let loupe_w = ui.available_width().max(100.0);
                            let loupe_h = (loupe_w * 0.65).min(130.0);
                            let (resp, painter) = ui.allocate_painter(egui::vec2(loupe_w, loupe_h), egui::Sense::hover());
                            // Center 25% zoomed in
                            let center_uv = egui::Rect::from_min_max(egui::pos2(0.35, 0.35), egui::pos2(0.65, 0.65));
                            painter.image(proc_tex.id(), resp.rect, center_uv, egui::Color32::WHITE);
                            painter.rect_stroke(resp.rect, 0.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 48, 56)), egui::StrokeKind::Inside);
                        }
                    }
                    InspectorTab::Histogram => {
                        if let Some(ref hist) = self.histogram {
                            render_histogram_modern(ui, hist);
                        }
                    }
                }

                ui.add_space(6.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("GLOBAL ADJUSTMENTS").strong().size(12.5).color(egui::Color32::WHITE));
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- TAB 1: BASIC EXPOSURE ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_basic_exposure"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.basic_exposure.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Basic Exposure", ICON_SUN)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.basic_exposure;
                            slider_row(ui, "Brightness", &mut p.brightness, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Contrast", &mut p.contrast, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Boost Blacks", &mut p.boost_blacks, 0.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Boost Whites", &mut p.boost_whites, 0.0..=1.0, 0.0, &mut self.is_dirty);
                        });
                    ui.separator();

                    // --- TAB 2: ADAPTIVE EXPOSURE ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_adaptive_exposure"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.adaptive_exposure.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Adaptive Exposure", ICON_EYE)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.adaptive_exposure;
                            slider_row(ui, "Adaptive Exp", &mut p.adaptive_exposure, 0.0..=3.0, 0.0, &mut self.is_dirty);
                            slider_row_u32(ui, "Regions", &mut p.regions, 1..=30, 10, &mut self.is_dirty);
                            slider_row(ui, "Contrast", &mut p.contrast, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Brightness", &mut p.brightness, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Protect High", &mut p.protect_highlights, 0.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Protect Shad", &mut p.protect_shadows, 0.0..=1.0, 0.0, &mut self.is_dirty);
                        });
                    ui.separator();

                    // --- TAB 3: DETAILS ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_details"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.details.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Details & Clarity", ICON_SPARKLES)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.details;
                            slider_row(ui, "Strength", &mut p.strength, -1.0..=3.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Detail Boost", &mut p.detail_boost, 0.0..=3.0, 1.0, &mut self.is_dirty);
                            slider_row(ui, "Threshold", &mut p.threshold, 0.0..=1.0, 0.10, &mut self.is_dirty);
                            slider_row(ui, "Radius", &mut p.radius, 0.5..=30.0, 1.0, &mut self.is_dirty);
                            slider_row(ui, "Sharpen", &mut p.sharpen, 0.0..=2.0, 0.0, &mut self.is_dirty);
                            if ui.checkbox(&mut p.process_independently, "Process Details Independently").on_hover_text("Extracts detail from original un-tone-mapped image to avoid highlight blowout and halos.").changed() {
                                self.is_dirty = true;
                            }
                        });
                    ui.separator();

                    // --- TAB 4: COLOR ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_color"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.color.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Color & Vibrance", ICON_PALETTE)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.color;
                            slider_row(ui, "Adaptive Sat", &mut p.adaptive_saturation, -1.0..=2.0, 0.0, &mut self.is_dirty);
                            slider_row_u32(ui, "Color Regions", &mut p.color_regions, 1..=30, 10, &mut self.is_dirty);
                            slider_row(ui, "Saturation", &mut p.saturation, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Sat Boost", &mut p.saturation_boost, 0.0..=2.0, 1.0, &mut self.is_dirty);
                            slider_row(ui, "Hue Shift", &mut p.hue, -180.0..=180.0, 0.0, &mut self.is_dirty);
                        });
                    ui.separator();

                    // --- TAB 5: NOISE ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_noise"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.noise.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Noise Suppression", ICON_VOLUME_OFF)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.noise;
                            slider_row(ui, "Suppression", &mut p.suppression, 0.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Amount", &mut p.amount, 0.0..=1.0, 0.50, &mut self.is_dirty);
                        });
                    ui.separator();

                    // --- TAB 6: CURVE TOOL (INTERACTIVE GRAPHICAL SPLINE) ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_curve"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.curve.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Curve Tool", ICON_VECTOR_BEZIER)).strong().size(13.5));
                        })
                        .body(|ui| {
                            render_curve_editor(ui, &mut self.params.curve, self.histogram.as_ref(), &mut self.is_dirty);
                        });
                    ui.separator();

                    // --- TAB 7: FINISHING TOUCHES ---
                    ui.add_space(4.0);
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("tab_finishing"), true)
                        .show_header(ui, |ui| {
                            if ui.checkbox(&mut self.params.finishing.enabled, "").changed() {
                                self.is_dirty = true;
                            }
                            ui.label(egui::RichText::new(format!("{} Finishing Touches", ICON_WAND)).strong().size(13.5));
                        })
                        .body(|ui| {
                            let p = &mut self.params.finishing;
                            slider_row(ui, "Transparency", &mut p.transparency, 0.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Warmth", &mut p.warmth, -1.0..=1.0, 0.0, &mut self.is_dirty);

                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Vignette:").strong().size(12.0));
                            slider_row(ui, "Amount", &mut p.vignette.amount, -1.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Size", &mut p.vignette.size, 0.1..=1.0, 0.50, &mut self.is_dirty);
                            slider_row(ui, "Transition", &mut p.vignette.transition, 0.0..=1.0, 0.50, &mut self.is_dirty);
                            slider_row(ui, "Roundness", &mut p.vignette.roundness, 0.0..=1.0, 0.0, &mut self.is_dirty);

                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Film Grain:").strong().size(12.0));
                            slider_row(ui, "Amount", &mut p.grain.amount, 0.0..=1.0, 0.0, &mut self.is_dirty);
                            slider_row(ui, "Size", &mut p.grain.size, 0.5..=3.0, 1.0, &mut self.is_dirty);

                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Border:").strong().size(12.0));
                            if ui.checkbox(&mut p.border.enabled, "Enable Border").changed() {
                                self.is_dirty = true;
                            }
                            slider_row(ui, "Thickness", &mut p.border.thickness, 0.0..=0.15, 0.0, &mut self.is_dirty);
                        });
                    ui.separator();

                    ui.add_space(20.0);
                });
            });

        // ==========================================
        // 5. CENTER CANVAS VIEWPORT
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

            // Pan & Dragging
            if response.dragged_by(egui::PointerButton::Primary) {
                if self.view_mode == ViewMode::Split {
                    let rel_x = (response.interact_pointer_pos().unwrap().x - response.rect.min.x) / response.rect.width();
                    self.split_ratio = rel_x.clamp(0.02, 0.98);
                } else {
                    self.pan += response.drag_delta();
                }
            } else if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
                self.pan += response.drag_delta();
            }

            // Mouse wheel zoom
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom = (self.zoom + scroll_delta * 0.0025).clamp(0.1, 10.0);
            }

            if let Some(ref proc_tex) = self.proc_texture {
                let img_size = proc_tex.size_vec2();
                let scaled_size = img_size * self.zoom;

                let center = response.rect.center() + self.pan;
                let img_rect = egui::Rect::from_center_size(center, scaled_size);

                // Quick Hold-to-Compare original via Spacebar
                let holding_space = ui.input(|i| i.key_down(egui::Key::Space));

                match self.view_mode {
                    ViewMode::Single => {
                        let tex = if holding_space {
                            self.orig_texture.as_ref().unwrap_or(proc_tex)
                        } else {
                            proc_tex
                        };
                        painter.image(tex.id(), img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                    }
                    ViewMode::Split => {
                        if let Some(ref orig_tex) = self.orig_texture {
                            let split_x = img_rect.min.x + img_rect.width() * self.split_ratio;

                            // Left: Original
                            let orig_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(self.split_ratio, 1.0));
                            let orig_rect = egui::Rect::from_min_max(img_rect.min, egui::pos2(split_x, img_rect.max.y));
                            painter.image(orig_tex.id(), orig_rect, orig_uv, egui::Color32::WHITE);

                            // Right: Processed
                            let proc_uv = egui::Rect::from_min_max(egui::pos2(self.split_ratio, 0.0), egui::pos2(1.0, 1.0));
                            let proc_rect = egui::Rect::from_min_max(egui::pos2(split_x, img_rect.min.y), img_rect.max);
                            painter.image(proc_tex.id(), proc_rect, proc_uv, egui::Color32::WHITE);

                            // Divider line
                            painter.line_segment(
                                [egui::pos2(split_x, img_rect.min.y), egui::pos2(split_x, img_rect.max.y)],
                                egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
                            );

                            // Draggable Handle Pill
                            let handle_center = egui::pos2(split_x, img_rect.center().y);
                            painter.circle_filled(handle_center, 9.0, egui::Color32::from_rgb(0, 210, 255));
                            painter.circle_filled(handle_center, 4.0, egui::Color32::WHITE);
                        }
                    }
                    ViewMode::SideBySide => {
                        if let Some(ref orig_tex) = self.orig_texture {
                            let half_w = scaled_size.x * 0.49;
                            let left_rect = egui::Rect::from_min_size(egui::pos2(center.x - scaled_size.x * 0.5, img_rect.min.y), egui::vec2(half_w, scaled_size.y));
                            let right_rect = egui::Rect::from_min_size(egui::pos2(center.x + scaled_size.x * 0.01, img_rect.min.y), egui::vec2(half_w, scaled_size.y));

                            painter.image(orig_tex.id(), left_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                            painter.image(proc_tex.id(), right_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                        }
                    }
                }

                // Overlay shortcut hints
                painter.text(
                    egui::pos2(response.rect.min.x + 14.0, response.rect.max.y - 20.0),
                    egui::Align2::LEFT_BOTTOM,
                    "Space: Hold to compare original | Scroll: Zoom | Drag: Pan",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgba_unmultiplied(200, 205, 220, 140),
                );
            }
        });
    }
}

// ==========================================
// UI HELPER FUNCTIONS & CONTROLS
// ==========================================

fn slider_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default_val: f32,
    is_dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.add_sized([100.0, 18.0], egui::Label::new(egui::RichText::new(label).size(12.0)));
        let slider = egui::Slider::new(value, range).show_value(true).fixed_decimals(2);
        let resp = ui.add(slider);
        if resp.changed() {
            *is_dirty = true;
        }
        if resp.double_clicked() {
            *value = default_val;
            *is_dirty = true;
        }
    });
}

fn slider_row_u32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    default_val: u32,
    is_dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.add_sized([100.0, 18.0], egui::Label::new(egui::RichText::new(label).size(12.0)));
        let slider = egui::Slider::new(value, range).show_value(true);
        let resp = ui.add(slider);
        if resp.changed() {
            *is_dirty = true;
        }
        if resp.double_clicked() {
            *value = default_val;
            *is_dirty = true;
        }
    });
}

fn render_histogram_modern(ui: &mut egui::Ui, hist: &HistogramData) {
    let height = 64.0;
    let width = ui.available_width().max(120.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(16, 17, 21));
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 42, 50)), egui::StrokeKind::Inside);

    let max_val = hist.lum.iter().copied().max().unwrap_or(1).max(1) as f32;

    for i in 0..255 {
        let x0 = rect.min.x + (i as f32 / 255.0) * rect.width();
        let x1 = rect.min.x + ((i + 1) as f32 / 255.0) * rect.width();

        let lum_h = (hist.lum[i] as f32 / max_val) * (height - 6.0);
        let y0 = rect.max.y - 3.0;
        let y1 = y0 - lum_h;

        painter.line_segment(
            [egui::pos2(x0, y0), egui::pos2(x1, y1)],
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(220, 225, 235, 170)),
        );
    }
}
