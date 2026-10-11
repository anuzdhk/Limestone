//! Left sidebar: a Finder-style source list of collections above a Photos-style grid
//! of live preset thumbnails.

use crate::icons::*;
use crate::theme::*;
use crate::thumbnails::ThumbnailCache;
use crate::user_presets::MY_COLLECTION;
use crate::widgets::controls::{caption, icon_button, search_field, separator};
use adjust_core::{Preset, PresetCollection};
use eframe::egui::{self, Color32, Sense, Ui};
use std::collections::HashSet;

/// What the preset grid is showing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Library {
    All,
    Favorites,
    Mine,
    Collection(&'static str),
}

impl Library {
    pub fn title(&self) -> &str {
        match self {
            Library::All => "All Presets",
            Library::Favorites => "Favorites",
            Library::Mine => MY_COLLECTION,
            Library::Collection(name) => name,
        }
    }
}

pub enum SidebarAction {
    Apply(Preset),
    SavePreset,
    ImportPreset,
    ExportPreset(Preset),
    DeletePreset(String),
    ToggleFavorite(String),
}

pub struct SidebarState<'a> {
    pub library: &'a mut Library,
    pub search: &'a mut String,
    pub collections: &'a [PresetCollection],
    pub factory: &'a [Preset],
    pub user: &'a [Preset],
    pub favorites: &'a HashSet<String>,
    pub selected_preset: &'a str,
    pub thumbs: &'a ThumbnailCache,
}

fn collection_icon(name: &str) -> &'static str {
    match name {
        adjust_core::preset::ADJUST4 => ICON_LIBRARY,
        adjust_core::preset::CLASSIC => ICON_APERTURE,
        adjust_core::preset::VIBRANT => ICON_PALETTE,
        adjust_core::preset::HDR => ICON_HDR,
        adjust_core::preset::FILM => ICON_MOVIE,
        adjust_core::preset::TONED => ICON_CIRCLE_HALF,
        adjust_core::preset::STYLIZED => ICON_BRUSH,
        _ => ICON_FILTER,
    }
}

/// Presets shown for a library selection (before search filtering).
pub fn presets_for<'a>(
    library: &Library,
    collections: &[PresetCollection],
    factory: &'a [Preset],
    user: &'a [Preset],
    favorites: &HashSet<String>,
) -> Vec<&'a Preset> {
    let find = |name: &str| factory.iter().chain(user).find(|p| p.name == name);
    match library {
        Library::All => factory.iter().collect(),
        Library::Favorites => factory.iter().chain(user).filter(|p| favorites.contains(&p.name)).collect(),
        Library::Mine => user.iter().collect(),
        Library::Collection(name) => collections
            .iter()
            .find(|c| c.name == *name)
            .map(|c| c.presets.iter().filter_map(|n| find(n)).collect())
            .unwrap_or_default(),
    }
}

pub fn sidebar(ui: &mut Ui, s: SidebarState) -> Vec<SidebarAction> {
    let p = pal(ui);
    let mut actions = Vec::new();

    // ---- Bottom bar: save / import ----------------------------------------
    egui::TopBottomPanel::bottom("sidebar_footer")
        .frame(egui::Frame::new().fill(p.sidebar).inner_margin(egui::Margin::symmetric(10, 6)))
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            separator(ui);
            ui.add_space(SPACE_4);
            ui.horizontal(|ui| {
                if icon_button(ui, ICON_SAVE, "Save current adjustments as a preset", true, false).clicked() {
                    actions.push(SidebarAction::SavePreset);
                }
                if icon_button(ui, ICON_IMPORT, "Import a preset file into My Collection", true, false).clicked() {
                    actions.push(SidebarAction::ImportPreset);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total = s.factory.len() + s.user.len();
                    ui.label(egui::RichText::new(format!("{total} presets")).size(11.0).color(p.text_tertiary));
                });
            });
        });

    egui::ScrollArea::vertical()
        .id_salt("sidebar_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(SPACE_8);

            // ---- Source list ---------------------------------------------
            caption(ui, "LIBRARY");
            let all_count = s.factory.len();
            let fav_count = s.factory.iter().chain(s.user).filter(|p| s.favorites.contains(&p.name)).count();
            source_row(ui, s.library, Library::All, ICON_GRID, "All Presets", all_count);
            source_row(ui, s.library, Library::Favorites, ICON_STAR_FILLED, "Favorites", fav_count);
            source_row(ui, s.library, Library::Mine, ICON_USER, MY_COLLECTION, s.user.len());
            ui.add_space(SPACE_8);
            caption(ui, "COLLECTIONS");
            for col in s.collections {
                source_row(
                    ui,
                    s.library,
                    Library::Collection(col.name),
                    collection_icon(col.name),
                    col.name.trim_end_matches(" Collection"),
                    col.presets.len(),
                );
            }

            ui.add_space(SPACE_12);
            search_field(ui, s.search, "Search presets");
            ui.add_space(SPACE_8);

            // ---- Grid ------------------------------------------------------
            let query = s.search.trim().to_lowercase();
            // Searching always looks through everything.
            let scope = if query.is_empty() { s.library.clone() } else { Library::All };
            let mut items = presets_for(&scope, s.collections, s.factory, s.user, s.favorites);
            if !query.is_empty() {
                items = s
                    .factory
                    .iter()
                    .chain(s.user)
                    .filter(|p| {
                        p.name.to_lowercase().contains(&query)
                            || p.category.to_lowercase().contains(&query)
                            || p.description.to_lowercase().contains(&query)
                    })
                    .collect();
            }

            ui.horizontal(|ui| {
                let title = if query.is_empty() { s.library.title().to_string() } else { "Search Results".to_string() };
                ui.label(egui::RichText::new(title).size(15.0).strong().color(p.text));
                ui.label(egui::RichText::new(format!("{}", items.len())).size(12.0).color(p.text_tertiary));
            });
            if let Library::Collection(name) = &scope {
                if let Some(col) = s.collections.iter().find(|c| c.name == *name) {
                    ui.label(egui::RichText::new(col.description).size(11.5).color(p.text_secondary));
                }
            }
            ui.add_space(SPACE_8);

            if items.is_empty() {
                ui.add_space(SPACE_16);
                ui.vertical_centered(|ui| {
                    let msg = match &scope {
                        Library::Favorites => "Right-click a preset and choose\n“Add to Favorites”.",
                        Library::Mine => "Save your adjustments with the\nsave button below.",
                        _ => "No presets match your search.",
                    };
                    ui.label(egui::RichText::new(msg).size(12.0).color(p.text_tertiary));
                });
            }

            let gap = 10.0;
            let card_w = ((ui.available_width() - gap) / 2.0).floor();
            for row in items.chunks(2) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for preset in row {
                        if let Some(a) = preset_card(ui, preset, card_w, &s) {
                            actions.push(a);
                        }
                    }
                });
                ui.add_space(4.0);
            }
            ui.add_space(SPACE_12);
        });

    actions
}

/// One source-list row: icon, title, count; rounded accent highlight when selected.
fn source_row(ui: &mut Ui, current: &mut Library, value: Library, icon: &str, title: &str, count: usize) {
    let p = pal(ui);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());
    let selected = *current == value;
    let painter = ui.painter_at(rect);
    if selected {
        painter.rect_filled(rect, RADIUS_CONTROL, p.selection);
    } else if resp.hovered() {
        painter.rect_filled(rect, RADIUS_CONTROL, with_alpha(p.control, 140));
    }
    let icon_color = if value == Library::Favorites { p.star } else { p.accent };
    painter.text(
        rect.left_center() + egui::vec2(16.0, 0.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(15.0),
        icon_color,
    );
    painter.text(
        rect.left_center() + egui::vec2(32.0, 0.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(13.0),
        p.text,
    );
    painter.text(
        rect.right_center() - egui::vec2(10.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        count.to_string(),
        egui::FontId::proportional(11.5),
        p.text_tertiary,
    );
    if resp.clicked() {
        *current = value;
    }
}

/// Thumbnail card with name; selected cards get an accent ring.
fn preset_card(ui: &mut Ui, preset: &Preset, width: f32, s: &SidebarState) -> Option<SidebarAction> {
    let p = pal(ui);
    let img_h = (width * 0.72).round();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, img_h + 22.0), Sense::click());
    let img_rect = egui::Rect::from_min_size(rect.min, egui::vec2(width, img_h));
    let selected = s.selected_preset == preset.name;
    let fav = s.favorites.contains(&preset.name);
    let painter = ui.painter_at(rect.expand(3.0));

    match s.thumbs.get(&preset.name) {
        Some(tex) => {
            // Centre-crop the thumbnail to the card's aspect ratio.
            let [tw, th] = tex.size().map(|v| v as f32);
            let card_aspect = width / img_h;
            let uv = if tw / th > card_aspect {
                let u = (card_aspect * th / tw) / 2.0;
                egui::Rect::from_min_max(egui::pos2(0.5 - u, 0.0), egui::pos2(0.5 + u, 1.0))
            } else {
                let v = (tw / card_aspect / th) / 2.0;
                egui::Rect::from_min_max(egui::pos2(0.0, 0.5 - v), egui::pos2(1.0, 0.5 + v))
            };
            egui::Image::new((tex.id(), img_rect.size()))
                .uv(uv)
                .corner_radius(RADIUS_CONTROL)
                .paint_at(ui, img_rect);
        }
        None => {
            painter.rect_filled(img_rect, RADIUS_CONTROL, p.control);
            painter.text(
                img_rect.center(),
                egui::Align2::CENTER_CENTER,
                ICON_PHOTO,
                egui::FontId::proportional(20.0),
                p.text_tertiary,
            );
        }
    }

    if selected {
        painter.rect_stroke(img_rect.expand(1.5), RADIUS_CARD, egui::Stroke::new(2.5_f32, p.accent), egui::StrokeKind::Outside);
    } else if resp.hovered() {
        painter.rect_stroke(img_rect, RADIUS_CONTROL, egui::Stroke::new(1.0_f32, p.separator), egui::StrokeKind::Inside);
    }
    if fav {
        let c = img_rect.right_top() + egui::vec2(-12.0, 12.0);
        painter.circle_filled(c, 9.0, Color32::from_black_alpha(110));
        painter.text(c, egui::Align2::CENTER_CENTER, ICON_STAR_FILLED, egui::FontId::proportional(12.0), p.star);
    }

    let mut name = egui::text::LayoutJob::simple_singleline(
        preset.name.clone(),
        egui::FontId::proportional(12.0),
        if selected { p.accent } else { p.text },
    );
    name.wrap = egui::text::TextWrapping::truncate_at_width(width);
    let galley = ui.fonts(|f| f.layout_job(name));
    let text_pos = egui::pos2(
        img_rect.center().x - galley.size().x.min(width) / 2.0,
        img_rect.bottom() + 4.0,
    );
    painter.with_clip_rect(rect).galley(text_pos, galley, p.text);

    let resp = resp.on_hover_text(format!("{}\n{}", preset.name, preset.description));
    let mut action = None;
    if resp.clicked() {
        action = Some(SidebarAction::Apply(preset.clone()));
    }
    resp.context_menu(|ui| {
        let fav_label = if fav { "Remove from Favorites" } else { "Add to Favorites" };
        if ui.button(format!("{ICON_STAR}  {fav_label}")).clicked() {
            action = Some(SidebarAction::ToggleFavorite(preset.name.clone()));
            ui.close_menu();
        }
        if ui.button(format!("{ICON_EXPORT}  Export Preset…")).clicked() {
            action = Some(SidebarAction::ExportPreset(preset.clone()));
            ui.close_menu();
        }
        if preset.category == MY_COLLECTION {
            ui.separator();
            if ui.button(format!("{ICON_TRASH}  Delete")).clicked() {
                action = Some(SidebarAction::DeletePreset(preset.name.clone()));
                ui.close_menu();
            }
        }
    });
    action
}
