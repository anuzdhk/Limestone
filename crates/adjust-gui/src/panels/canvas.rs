use crate::icons::*;
use crate::panels::toolbar::ViewMode;
use crate::theme::*;
use eframe::egui::{self, Color32, Ui};
use std::path::PathBuf;

pub struct CanvasResponse {
    pub dropped_path: Option<PathBuf>,
    pub open_clicked: bool,
}

pub fn render_central_canvas(
    ui: &mut Ui,
    view_mode: ViewMode,
    split_ratio: &mut f32,
    zoom: &mut f32,
    pan: &mut egui::Vec2,
    proc_texture: Option<&egui::TextureHandle>,
    orig_texture: Option<&egui::TextureHandle>,
    has_image: bool,
) -> CanvasResponse {
    let mut resp = CanvasResponse {
        dropped_path: None,
        open_clicked: false,
    };

    // Receive drag and dropped files
    ui.input(|i| {
        if !i.raw.dropped_files.is_empty() {
            if let Some(ref file) = i.raw.dropped_files.first() {
                if let Some(ref path) = file.path {
                    resp.dropped_path = Some(path.clone());
                }
            }
        }
    });

    let available_size = ui.available_size();
    let (response, painter) =
        ui.allocate_painter(available_size, egui::Sense::click_and_drag());

    // Canvas background
    painter.rect_filled(response.rect, 0.0, BG_CANVAS);

    // Empty state if no image loaded
    if !has_image || proc_texture.is_none() {
        let center = response.rect.center();
        let box_size = egui::vec2(380.0, 220.0);
        let box_rect = egui::Rect::from_center_size(center, box_size);

        // Dashed drop border
        painter.rect_filled(box_rect, RADIUS_CARD, SURFACE_CARD);
        painter.rect_stroke(
            box_rect,
            RADIUS_CARD,
            egui::Stroke::new(1.5, BORDER_SUBTLE),
            egui::StrokeKind::Inside,
        );

        painter.text(
            egui::pos2(center.x, center.y - 36.0),
            egui::Align2::CENTER_CENTER,
            ICON_PHOTO,
            egui::FontId::proportional(36.0),
            ACCENT,
        );

        painter.text(
            egui::pos2(center.x, center.y + 10.0),
            egui::Align2::CENTER_CENTER,
            "Drag & Drop an image here",
            egui::FontId::proportional(15.0),
            TEXT_PRIMARY,
        );

        painter.text(
            egui::pos2(center.x, center.y + 34.0),
            egui::Align2::CENTER_CENTER,
            "Supports PNG, JPEG, TIFF, WebP, BMP",
            egui::FontId::proportional(12.0),
            TEXT_MUTED,
        );

        // Open button inside empty state
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(center.x, center.y + 70.0),
            egui::vec2(120.0, 28.0),
        );
        let btn_resp = ui.interact(btn_rect, ui.id().with("empty_open_btn"), egui::Sense::click());
        let btn_fill = if btn_resp.is_pointer_button_down_on() {
            ACCENT_PRESSED
        } else if btn_resp.hovered() {
            ACCENT_HOVER
        } else {
            ACCENT
        };
        painter.rect_filled(btn_rect, RADIUS_CONTROL, btn_fill);
        painter.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Open Image",
            egui::FontId::proportional(12.5),
            Color32::BLACK,
        );

        if btn_resp.clicked() {
            resp.open_clicked = true;
        }

        return resp;
    }

    let proc_tex = proc_texture.unwrap();

    // Pan & Drag Interactions
    if response.dragged_by(egui::PointerButton::Primary) {
        if view_mode == ViewMode::Split {
            let rel_x = (response.interact_pointer_pos().unwrap().x - response.rect.min.x)
                / response.rect.width();
            *split_ratio = rel_x.clamp(0.02, 0.98);
        } else {
            *pan += response.drag_delta();
        }
    } else if response.dragged_by(egui::PointerButton::Secondary)
        || response.dragged_by(egui::PointerButton::Middle)
    {
        *pan += response.drag_delta();
    }

    // Mouse wheel zoom
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll_delta != 0.0 {
        *zoom = (*zoom + scroll_delta * 0.0025).clamp(0.05, 10.0);
    }

    // Calculate image layout and centered drop shadow
    let img_size = proc_tex.size_vec2();
    let scaled_size = img_size * *zoom;
    let center = response.rect.center() + *pan;
    let img_rect = egui::Rect::from_center_size(center, scaled_size);

    // Subtle drop shadow around photo
    let shadow_rect = img_rect.expand(6.0);
    painter.rect_filled(
        shadow_rect,
        RADIUS_CONTROL,
        Color32::from_black_alpha(80),
    );

    // Space hold-to-compare detection
    let holding_space = ui.input(|i| i.key_down(egui::Key::Space));

    match view_mode {
        ViewMode::Single => {
            let active_tex = if holding_space {
                orig_texture.unwrap_or(proc_tex)
            } else {
                proc_tex
            };

            painter.image(
                active_tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            if holding_space {
                // Show badge "ORIGINAL (HOLDING SPACE)"
                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(img_rect.min.x + 12.0, img_rect.min.y + 12.0),
                    egui::vec2(160.0, 24.0),
                );
                painter.rect_filled(badge_rect, RADIUS_CONTROL, Color32::from_black_alpha(190));
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ORIGINAL (HOLDING SPACE)",
                    egui::FontId::monospace(10.5),
                    ACCENT,
                );
            }
        }
        ViewMode::Split => {
            if let Some(orig_tex) = orig_texture {
                let split_x = img_rect.min.x + img_rect.width() * *split_ratio;

                // Left partition: Original
                let orig_uv = egui::Rect::from_min_max(
                    egui::pos2(0.0, 0.0),
                    egui::pos2(*split_ratio, 1.0),
                );
                let orig_view =
                    egui::Rect::from_min_max(img_rect.min, egui::pos2(split_x, img_rect.max.y));
                painter.image(orig_tex.id(), orig_view, orig_uv, Color32::WHITE);

                // Right partition: Processed
                let proc_uv = egui::Rect::from_min_max(
                    egui::pos2(*split_ratio, 0.0),
                    egui::pos2(1.0, 1.0),
                );
                let proc_view =
                    egui::Rect::from_min_max(egui::pos2(split_x, img_rect.min.y), img_rect.max);
                painter.image(proc_tex.id(), proc_view, proc_uv, Color32::WHITE);

                // Split divider line
                painter.line_segment(
                    [
                        egui::pos2(split_x, img_rect.min.y),
                        egui::pos2(split_x, img_rect.max.y),
                    ],
                    egui::Stroke::new(1.5, Color32::WHITE),
                );

                // Elegant grip handle pill
                let pill_center = egui::pos2(split_x, img_rect.center().y);
                let pill_rect = egui::Rect::from_center_size(pill_center, egui::vec2(24.0, 40.0));
                painter.rect_filled(pill_rect, 12.0, SURFACE_PANEL);
                painter.rect_stroke(
                    pill_rect,
                    12.0,
                    egui::Stroke::new(1.5, ACCENT),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    pill_center,
                    egui::Align2::CENTER_CENTER,
                    ICON_GRIP,
                    egui::FontId::proportional(16.0),
                    TEXT_PRIMARY,
                );

                // Tags "Original" and "Adjusted"
                painter.text(
                    egui::pos2(split_x - 12.0, img_rect.min.y + 16.0),
                    egui::Align2::RIGHT_TOP,
                    "ORIGINAL",
                    egui::FontId::monospace(10.0),
                    TEXT_SECONDARY,
                );
                painter.text(
                    egui::pos2(split_x + 12.0, img_rect.min.y + 16.0),
                    egui::Align2::LEFT_TOP,
                    "ADJUSTED",
                    egui::FontId::monospace(10.0),
                    ACCENT,
                );
            }
        }
        ViewMode::SideBySide => {
            if let Some(orig_tex) = orig_texture {
                let half_w = scaled_size.x * 0.495;
                let left_rect = egui::Rect::from_min_size(
                    egui::pos2(center.x - scaled_size.x * 0.5, img_rect.min.y),
                    egui::vec2(half_w, scaled_size.y),
                );
                let right_rect = egui::Rect::from_min_size(
                    egui::pos2(center.x + scaled_size.x * 0.005, img_rect.min.y),
                    egui::vec2(half_w, scaled_size.y),
                );

                painter.image(
                    orig_tex.id(),
                    left_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.image(
                    proc_tex.id(),
                    right_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Labels
                painter.text(
                    egui::pos2(left_rect.min.x + 8.0, left_rect.min.y + 8.0),
                    egui::Align2::LEFT_TOP,
                    "ORIGINAL",
                    egui::FontId::monospace(10.5),
                    TEXT_SECONDARY,
                );
                painter.text(
                    egui::pos2(right_rect.min.x + 8.0, right_rect.min.y + 8.0),
                    egui::Align2::LEFT_TOP,
                    "ADJUSTED",
                    egui::FontId::monospace(10.5),
                    ACCENT,
                );
            }
        }
    }

    // Canvas footer hint chip
    let hint_text = "Space: Hold to compare  |  Scroll: Zoom  |  Drag: Pan canvas";
    painter.text(
        egui::pos2(response.rect.min.x + 14.0, response.rect.max.y - 14.0),
        egui::Align2::LEFT_BOTTOM,
        hint_text,
        egui::FontId::proportional(11.0),
        Color32::from_rgba_unmultiplied(154, 163, 178, 140),
    );

    resp
}
