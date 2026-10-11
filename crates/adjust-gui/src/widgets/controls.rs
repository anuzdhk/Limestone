//! Small Apple-style controls: segmented control, toolbar buttons, pills, search field.

use crate::theme::*;
use eframe::egui::{self, Color32, Response, Sense, Stroke, Ui};

/// macOS segmented control. Each segment is `(value, label, tooltip)`.
/// Returns true when the selection changed.
pub fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    segments: &[(T, &str, &str)],
    segment_width: f32,
) -> bool {
    let p = pal(ui);
    let h = CONTROL_HEIGHT - 2.0;
    let total = egui::vec2(segment_width * segments.len() as f32 + 4.0, h + 4.0);
    let (rect, _) = ui.allocate_exact_size(total, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, RADIUS_CONTROL, p.control);

    let mut changed = false;
    for (i, (value, label, tip)) in segments.iter().enumerate() {
        let seg = egui::Rect::from_min_size(
            rect.min + egui::vec2(2.0 + i as f32 * segment_width, 2.0),
            egui::vec2(segment_width, h),
        );
        let resp = ui.interact(seg, ui.id().with(("seg", i, *label)), Sense::click());
        let selected = *current == *value;
        if selected {
            painter.rect_filled(seg.translate(egui::vec2(0.0, 0.5)), RADIUS_SMALL, p.shadow);
            painter.rect_filled(seg, RADIUS_SMALL, p.segment_selected);
        } else if resp.hovered() {
            painter.rect_filled(seg, RADIUS_SMALL, with_alpha(p.control_hover, 160));
        }
        painter.text(
            seg.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(12.5),
            if selected { p.text } else { p.text_secondary },
        );
        if !tip.is_empty() {
            resp.clone().on_hover_text(*tip);
        }
        if resp.clicked() && !selected {
            *current = *value;
            changed = true;
        }
    }
    changed
}

/// Borderless toolbar icon button (SF-Symbol style). Highlights on hover/active.
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str, enabled: bool, active: bool) -> Response {
    let p = pal(ui);
    let size = egui::vec2(30.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    let painter = ui.painter_at(rect);
    if active {
        painter.rect_filled(rect, RADIUS_CONTROL, p.selection);
    } else if enabled && resp.hovered() {
        painter.rect_filled(rect, RADIUS_CONTROL, p.control);
    }
    let color = if !enabled {
        p.text_tertiary
    } else if active {
        p.accent
    } else {
        p.text_secondary
    };
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(17.0), color);
    resp.on_hover_text(tooltip)
}

/// Rounded push button. `primary` uses the accent colour (default button).
pub fn pill_button(ui: &mut Ui, text: &str, primary: bool, enabled: bool) -> Response {
    let p = pal(ui);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(13.0),
        Color32::PLACEHOLDER,
    );
    let size = egui::vec2(galley.size().x + 24.0, CONTROL_HEIGHT);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    let pressed = resp.is_pointer_button_down_on();
    let (fill, fg) = if !enabled {
        (p.control, p.text_tertiary)
    } else if primary {
        let base = p.accent;
        let fill = if pressed {
            base.gamma_multiply(0.85)
        } else if resp.hovered() {
            lighten(base, 18)
        } else {
            base
        };
        (fill, p.on_accent)
    } else {
        (if resp.hovered() { p.control_hover } else { p.control }, p.text)
    };
    let painter = ui.painter_at(rect.expand(1.0));
    painter.rect_filled(rect.translate(egui::vec2(0.0, 0.5)), RADIUS_CONTROL, p.shadow);
    painter.rect_filled(rect, RADIUS_CONTROL, fill);
    painter.galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, fg);
    resp
}

fn lighten(c: Color32, by: u8) -> Color32 {
    Color32::from_rgb(c.r().saturating_add(by), c.g().saturating_add(by), c.b().saturating_add(by))
}

/// Rounded search field with a magnifier and clear button.
pub fn search_field(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    let p = pal(ui);
    let width = ui.available_width();
    egui::Frame::new()
        .fill(p.control)
        .corner_radius(RADIUS_CONTROL)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.set_width(width - 16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(egui::RichText::new(crate::icons::ICON_SEARCH).size(14.0).color(p.text_tertiary));
                let clear_w = if text.is_empty() { 0.0 } else { 20.0 };
                let resp = ui.add(
                    egui::TextEdit::singleline(text)
                        .hint_text(egui::RichText::new(hint).color(p.text_tertiary))
                        .frame(false)
                        .desired_width(ui.available_width() - clear_w),
                );
                if !text.is_empty()
                    && ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new(crate::icons::ICON_X).size(13.0).color(p.text_secondary),
                            )
                            .sense(Sense::click()),
                        )
                        .clicked()
                {
                    text.clear();
                }
                resp
            })
            .inner
        })
        .inner
}

/// Thin full-width separator line.
pub fn separator(ui: &mut Ui) {
    let p = pal(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0_f32, p.separator));
}

/// Small group caption (like Finder sidebar section titles).
pub fn caption(ui: &mut Ui, text: &str) {
    let p = pal(ui);
    ui.label(egui::RichText::new(text).size(11.0).strong().color(p.text_tertiary));
}
