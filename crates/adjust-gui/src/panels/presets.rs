use crate::icons::*;
use crate::theme::*;
use adjust_core::Preset;
use eframe::egui::{self, Color32, Ui};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct PresetPanelResponse {
    pub selected_preset: Option<Preset>,
    pub save_custom_clicked: bool,
    pub load_custom_clicked: bool,
    pub open_recent_path: Option<PathBuf>,
}

#[allow(clippy::too_many_arguments)] // one argument per piece of widget state
pub fn render_presets_sidebar(
    ui: &mut Ui,
    presets: &[Preset],
    selected_preset_name: &str,
    selected_collection: &mut String,
    preset_search: &mut String,
    favorite_presets: &mut HashSet<String>,
    sample_preview_texture: Option<&egui::TextureHandle>,
    recent_files: &[PathBuf],
) -> PresetPanelResponse {
    let mut result = PresetPanelResponse {
        selected_preset: None,
        save_custom_clicked: false,
        load_custom_clicked: false,
        open_recent_path: None,
    };

    let text_prim = theme_text_primary(ui.visuals());
    let text_sec = theme_text_secondary(ui.visuals());
    let text_mut = theme_text_muted(ui.visuals());
    let border_col = theme_border(ui.visuals());

    ui.add_space(SPACE_8);

    // Section Header: PRESETS & COLLECTIONS
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("PRESETS")
                .strong()
                .size(13.0)
                .color(text_sec),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let fav_active = selected_collection.as_str() == "Favorites";
            let fav_btn = egui::Button::new(
                egui::RichText::new(if fav_active { ICON_STAR_FILLED } else { ICON_STAR })
                    .size(13.0)
                    .color(if fav_active { ACCENT_STAR } else { text_sec }),
            )
            .min_size(egui::vec2(26.0, BTN_HEIGHT_SM))
            .frame(false);
            if ui.add(fav_btn).on_hover_text("Filter starred favorites").clicked() {
                if fav_active {
                    *selected_collection = "All Collections".to_string();
                } else {
                    *selected_collection = "Favorites".to_string();
                }
            }
        });
    });

    ui.add_space(SPACE_6);

    // Collections Selector Dropdown
    let collections = [
        "All Collections",
        "Classic Collection",
        "Vibrant Collection",
        "HDR Collection",
        "Stylized Collection",
        "Toned Collection",
        "Film Collection",
        "Favorites",
    ];

    egui::ComboBox::from_id_salt("preset_collections_combo")
        .selected_text(selected_collection.as_str())
        .width(ui.available_width() - SPACE_8)
        .show_ui(ui, |ui| {
            for col in &collections {
                ui.selectable_value(selected_collection, col.to_string(), *col);
            }
        });

    ui.add_space(SPACE_8);

    // Search Box with Tabler search icon
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(ICON_SEARCH)
                .size(13.0)
                .color(TEXT_SECONDARY),
        );
        let text_edit = egui::TextEdit::singleline(preset_search)
            .hint_text("Search presets...")
            .desired_width(ui.available_width() - SPACE_8);
        ui.add(text_edit);
    });

    ui.add_space(SPACE_6);
    ui.separator();
    ui.add_space(SPACE_4);

    // Presets List with Live Thumbnails
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let query = preset_search.trim().to_lowercase();

            let filtered_presets: Vec<&Preset> = presets
                .iter()
                .filter(|p| {
                    if selected_collection == "Favorites" {
                        favorite_presets.contains(&p.name)
                    } else if selected_collection == "All Collections" {
                        true
                    } else {
                        p.category == *selected_collection
                    }
                })
                .filter(|p| {
                    query.is_empty()
                        || p.name.to_lowercase().contains(&query)
                        || p.description.to_lowercase().contains(&query)
                })
                .collect();

            if filtered_presets.is_empty() {
                ui.add_space(SPACE_24);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("No presets match")
                            .size(13.0)
                            .color(text_mut),
                    );
                });
            }

            let card_fill = theme_surface_card(ui.visuals());

            for preset in filtered_presets {
                let is_selected = selected_preset_name == preset.name;
                let is_fav = favorite_presets.contains(&preset.name);

                let _item_resp = egui::Frame::new()
                    .fill(if is_selected {
                        card_fill
                    } else {
                        Color32::TRANSPARENT
                    })
                    .corner_radius(RADIUS_CONTROL)
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Left accent indicator for selected preset
                            if is_selected {
                                let (rect, painter) =
                                    ui.allocate_painter(egui::vec2(3.0, 26.0), egui::Sense::hover());
                                painter.rect_filled(rect.rect, 1.5, ACCENT);
                                ui.add_space(SPACE_4);
                            } else {
                                ui.add_space(SPACE_8);
                            }

                            // Tiny preview thumbnail
                            if let Some(tex) = sample_preview_texture {
                                let (thumb_rect, painter) =
                                    ui.allocate_painter(egui::vec2(28.0, 20.0), egui::Sense::hover());
                                painter.image(
                                    tex.id(),
                                    thumb_rect.rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                                painter.rect_stroke(
                                    thumb_rect.rect,
                                    2.0,
                                    egui::Stroke::new(1.0_f32, border_col),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            // Preset name
                            let name_label = egui::RichText::new(&preset.name)
                                .size(12.5)
                                .color(if is_selected {
                                    ACCENT
                                } else {
                                    text_prim
                                });

                            let label_resp = ui.selectable_label(is_selected, name_label);
                            if label_resp.clicked() {
                                result.selected_preset = Some(preset.clone());
                            }
                            label_resp.on_hover_text(&preset.description);

                            // Right-aligned star toggle
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let star_btn = egui::Button::new(
                                    egui::RichText::new(if is_fav {
                                        ICON_STAR_FILLED
                                    } else {
                                        ICON_STAR
                                    })
                                    .size(12.0)
                                    .color(if is_fav {
                                        ACCENT_STAR
                                    } else {
                                        text_mut
                                    }),
                                )
                                .frame(false);

                                if ui.add(star_btn).on_hover_text("Toggle favorite").clicked() {
                                    if is_fav {
                                        favorite_presets.remove(&preset.name);
                                    } else {
                                        favorite_presets.insert(preset.name.clone());
                                    }
                                }
                            });
                        });
                    });
            }

            ui.add_space(SPACE_16);
            ui.separator();
            ui.add_space(SPACE_8);

            // Recent Files Section
            if !recent_files.is_empty() {
                ui.label(
                    egui::RichText::new("RECENT FILES")
                        .size(11.0)
                        .strong()
                        .color(text_sec),
                );
                ui.add_space(SPACE_4);
                for path in recent_files.iter().take(5) {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Image".to_string());
                    let btn_recent = egui::Button::new(
                        egui::RichText::new(format!("{} {}", ICON_PHOTO, file_name))
                            .size(11.5)
                            .color(text_sec),
                    )
                    .frame(false);
                    if ui.add(btn_recent).on_hover_text(path.to_string_lossy()).clicked() {
                        result.open_recent_path = Some(path.clone());
                    }
                }
                ui.add_space(SPACE_12);
                ui.separator();
                ui.add_space(SPACE_8);
            }

            // Custom Preset Management Buttons
            ui.horizontal(|ui| {
                let btn_save = egui::Button::new(
                    egui::RichText::new(format!("{} Save...", ICON_DEVICE_FLOPPY)).size(12.0),
                )
                .min_size(egui::vec2(90.0, BTN_HEIGHT_MD));
                if ui.add(btn_save).on_hover_text("Save current settings as a JSON preset").clicked() {
                    result.save_custom_clicked = true;
                }

                let btn_load = egui::Button::new(
                    egui::RichText::new(format!("{} Load...", ICON_UPLOAD)).size(12.0),
                )
                .min_size(egui::vec2(90.0, BTN_HEIGHT_MD));
                if ui.add(btn_load).on_hover_text("Load preset from JSON file").clicked() {
                    result.load_custom_clicked = true;
                }
            });

            ui.add_space(SPACE_12);
        });

    result
}
