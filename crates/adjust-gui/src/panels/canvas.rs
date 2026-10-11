//! Central image canvas: zoom/pan, before/after comparison modes, full-res tile overlay,
//! drag-and-drop, and the empty-state welcome.

use crate::icons::*;
use crate::panels::toolbar::ViewMode;
use crate::theme::*;
use crate::widgets::controls::pill_button;
use eframe::egui::{self, Color32, Ui};
use std::path::PathBuf;

pub struct CanvasResponse {
    pub dropped_path: Option<PathBuf>,
    pub open_clicked: bool,
    pub canvas_size: egui::Vec2,
    pub canvas_rect: egui::Rect,
    /// Where the image was drawn on screen (None in the empty state).
    pub img_rect: Option<egui::Rect>,
}

pub struct CanvasState<'a> {
    pub view_mode: ViewMode,
    pub split_ratio: &'a mut f32,
    pub zoom: &'a mut f32,
    pub pan: &'a mut egui::Vec2,
    /// Full-resolution image size in pixels.
    pub image_px: egui::Vec2,
    pub proc_texture: Option<&'a egui::TextureHandle>,
    pub orig_texture: Option<&'a egui::TextureHandle>,
    pub tile: Option<(&'a egui::TextureHandle, egui::Rect)>,
}

const FULL_UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

pub fn canvas(ui: &mut Ui, s: CanvasState) -> CanvasResponse {
    let p = pal(ui);
    let available = ui.available_size();
    let mut resp = CanvasResponse {
        dropped_path: None,
        open_clicked: false,
        canvas_size: available,
        canvas_rect: egui::Rect::NOTHING,
        img_rect: None,
    };

    let (dropped, hovering_file) = ui.input(|i| {
        (
            i.raw.dropped_files.first().and_then(|f| f.path.clone()),
            !i.raw.hovered_files.is_empty(),
        )
    });
    resp.dropped_path = dropped;

    let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());
    let area = response.rect;
    resp.canvas_rect = area;
    painter.rect_filled(area, 0.0, p.canvas);

    let Some(proc_tex) = s.proc_texture else {
        resp.open_clicked = empty_state(ui, area, hovering_file);
        return resp;
    };

    // ---- Interaction ---------------------------------------------------------
    if response.dragged_by(egui::PointerButton::Primary) {
        if s.view_mode == ViewMode::Split {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some(img) = last_img_rect(ui) {
                    *s.split_ratio = ((pos.x - img.min.x) / img.width()).clamp(0.0, 1.0);
                }
            }
        } else {
            *s.pan += response.drag_delta();
        }
    } else if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
        *s.pan += response.drag_delta();
    }

    // Wheel zoom around the cursor.
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let old = *s.zoom;
            let new = (old * (scroll * 0.0025).exp()).clamp(0.01, 16.0);
            if let Some(cursor) = response.hover_pos() {
                // Keep the image point under the cursor fixed.
                let centre = area.center() + *s.pan;
                let k = new / old;
                *s.pan += (cursor - centre) * (1.0 - k);
            }
            *s.zoom = new;
        }
    }

    // ---- Layout ----------------------------------------------------------------
    let base_px = if s.image_px.x > 0.0 { s.image_px } else { proc_tex.size_vec2() };
    let size = base_px * *s.zoom / ui.ctx().pixels_per_point();
    let centre = area.center() + *s.pan;
    let img_rect = egui::Rect::from_center_size(centre, size);
    resp.img_rect = Some(img_rect);
    remember_img_rect(ui, img_rect);

    let holding_space = ui.input(|i| i.key_down(egui::Key::Space)) && !ui.ctx().wants_keyboard_input();
    let painter = painter.with_clip_rect(area);

    // Soft layered drop shadow; much lighter on the light canvas so it doesn't read as a frame.
    let strength = if p.dark { 1.0 } else { 0.35 };
    let shadow = |painter: &egui::Painter, r: egui::Rect| {
        for (grow, alpha) in [(14.0, 18.0), (8.0, 28.0), (3.0, 40.0)] {
            let a = (alpha * strength) as u8;
            painter.rect_filled(r.expand(grow).translate(egui::vec2(0.0, 4.0)), 8.0, Color32::from_black_alpha(a));
        }
    };

    match s.view_mode {
        ViewMode::Single => {
            shadow(&painter, img_rect);
            let tex = if holding_space { s.orig_texture.unwrap_or(proc_tex) } else { proc_tex };
            painter.image(tex.id(), img_rect, FULL_UV, Color32::WHITE);
            if !holding_space {
                if let Some((tile_tex, norm)) = s.tile {
                    let r = egui::Rect::from_min_max(
                        img_rect.lerp_inside(norm.min.to_vec2()),
                        img_rect.lerp_inside(norm.max.to_vec2()),
                    );
                    painter.image(tile_tex.id(), r, FULL_UV, Color32::WHITE);
                }
            }
            if holding_space {
                badge(&painter, &p, img_rect.left_top().max(area.left_top()) + egui::vec2(12.0, 12.0), "Original", egui::Align2::LEFT_TOP);
            }
        }
        ViewMode::Split => {
            shadow(&painter, img_rect);
            let orig = s.orig_texture.unwrap_or(proc_tex);
            let t = *s.split_ratio;
            let x = img_rect.min.x + img_rect.width() * t;
            painter.image(
                orig.id(),
                egui::Rect::from_min_max(img_rect.min, egui::pos2(x, img_rect.max.y)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(t, 1.0)),
                Color32::WHITE,
            );
            painter.image(
                proc_tex.id(),
                egui::Rect::from_min_max(egui::pos2(x, img_rect.min.y), img_rect.max),
                egui::Rect::from_min_max(egui::pos2(t, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            let top = img_rect.top().max(area.top());
            let bottom = img_rect.bottom().min(area.bottom());
            painter.line_segment([egui::pos2(x, top), egui::pos2(x, bottom)], egui::Stroke::new(2.0_f32, Color32::WHITE));
            let handle = egui::pos2(x, (top + bottom) / 2.0);
            painter.circle_filled(handle + egui::vec2(0.0, 1.5), 15.0, Color32::from_black_alpha(70));
            painter.circle_filled(handle, 14.0, Color32::WHITE);
            painter.text(handle, egui::Align2::CENTER_CENTER, "‹ ›", egui::FontId::proportional(14.0), Color32::from_gray(60));
            let label_y = top + 12.0;
            badge(&painter, &p, egui::pos2(x - 10.0, label_y), "Before", egui::Align2::RIGHT_TOP);
            badge(&painter, &p, egui::pos2(x + 10.0, label_y), "After", egui::Align2::LEFT_TOP);
        }
        ViewMode::SideBySide => {
            let gap = 12.0;
            let half = egui::vec2((size.x - gap) / 2.0, size.y);
            let left = egui::Rect::from_min_size(egui::pos2(centre.x - size.x / 2.0, img_rect.min.y), half);
            let right = egui::Rect::from_min_size(egui::pos2(left.max.x + gap, img_rect.min.y), half);
            for (r, tex, label) in [(left, s.orig_texture.unwrap_or(proc_tex), "Before"), (right, proc_tex, "After")] {
                shadow(&painter, r);
                painter.image(tex.id(), r, FULL_UV, Color32::WHITE);
                badge(&painter, &p, r.left_top().max(area.left_top()) + egui::vec2(10.0, 10.0), label, egui::Align2::LEFT_TOP);
            }
        }
    }

    resp
}

/// The split-view drag needs last frame's image rect.
fn remember_img_rect(ui: &Ui, r: egui::Rect) {
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("canvas_img_rect"), r));
}

fn last_img_rect(ui: &Ui) -> Option<egui::Rect> {
    ui.ctx().data(|d| d.get_temp(egui::Id::new("canvas_img_rect")))
}

/// Small translucent label pill over the image.
fn badge(painter: &egui::Painter, p: &Palette, pos: egui::Pos2, text: &str, align: egui::Align2) {
    let galley = painter.layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.5), Color32::WHITE);
    let size = galley.size() + egui::vec2(16.0, 8.0);
    let rect = align.anchor_size(pos, size);
    painter.rect_filled(rect, egui::CornerRadius::same(10), Color32::from_black_alpha(140));
    painter.galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
    let _ = p;
}

/// Welcome / drop target. Returns true if "Open Image…" was clicked.
fn empty_state(ui: &mut Ui, area: egui::Rect, hovering_file: bool) -> bool {
    let p = pal(ui);
    let card = egui::Rect::from_center_size(area.center(), egui::vec2(420.0, 250.0));
    let painter = ui.painter();
    let stroke_col = if hovering_file { p.accent } else { p.separator };
    painter.rect_filled(card, RADIUS_CARD, if hovering_file { p.selection } else { with_alpha(p.card, 120) });
    // Dashed border
    let pts = [card.left_top(), card.right_top(), card.right_bottom(), card.left_bottom(), card.left_top()];
    for w in pts.windows(2) {
        painter.add(egui::Shape::dashed_line(&[w[0], w[1]], egui::Stroke::new(1.5_f32, stroke_col), 8.0, 6.0));
    }
    painter.text(card.center() - egui::vec2(0.0, 62.0), egui::Align2::CENTER_CENTER, ICON_PHOTO, egui::FontId::proportional(44.0), p.accent);
    painter.text(
        card.center() - egui::vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        if hovering_file { "Drop to open" } else { "Drop a photo here" },
        egui::FontId::proportional(18.0),
        p.text,
    );
    painter.text(
        card.center() + egui::vec2(0.0, 16.0),
        egui::Align2::CENTER_CENTER,
        "or paste with Ctrl+V · PNG, JPEG, TIFF, WebP, BMP",
        egui::FontId::proportional(12.0),
        p.text_secondary,
    );
    let btn = egui::Rect::from_center_size(card.center() + egui::vec2(0.0, 62.0), egui::vec2(140.0, CONTROL_HEIGHT));
    ui.scope_builder(egui::UiBuilder::new().max_rect(btn).layout(egui::Layout::top_down(egui::Align::Center)), |ui| {
        pill_button(ui, "Open Image…", true, true).clicked()
    })
    .inner
}
