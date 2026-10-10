use crate::theme::*;
use eframe::egui::{self, Ui};
use std::ops::RangeInclusive;

/// Renders a precision photo-editor slider with thin track, accent progress fill,
/// circular handle with hover enlargement, default tick mark, and tabular numeric display.
/// - Double-click resets to default.
/// - Shift + drag performs fine-grained adjustment.
/// - Ctrl + click enables direct numeric entry.
pub fn precision_slider_row(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    default_val: f32,
    is_dirty: &mut bool,
    tooltip: &str,
) {
    let sec_col = theme_text_secondary(ui.visuals());

    ui.horizontal(|ui| {
        // Label on left with tooltip
        let label_response = ui.add_sized(
            [96.0, 18.0],
            egui::Label::new(
                egui::RichText::new(label)
                    .size(12.0)
                    .color(sec_col),
            ),
        );
        if !tooltip.is_empty() {
            label_response.on_hover_text(tooltip);
        }

        let start = *range.start();
        let end = *range.end();
        let span = (end - start).abs().max(0.001);

        // Check if Shift is held for fine adjustment
        let shift_down = ui.input(|i| i.modifiers.shift);
        let step = if shift_down { span * 0.001 } else { span * 0.01 };

        let slider = egui::Slider::new(value, range.clone())
            .show_value(false)
            .step_by(step as f64);

        let slider_resp = ui.add(slider);

        if slider_resp.changed() {
            *is_dirty = true;
        }

        // Double click reset
        if slider_resp.double_clicked() {
            *value = default_val;
            *is_dirty = true;
        }

        // Numeric entry / readout on right (Monospace / Tabular to prevent jitter)
        let mut text_val = format!("{:.2}", *value);
        let text_edit = ui.add_sized(
            [46.0, 18.0],
            egui::TextEdit::singleline(&mut text_val)
                .font(egui::FontId::monospace(11.0))
                .horizontal_align(egui::Align::RIGHT),
        );

        if text_edit.lost_focus() {
            if let Ok(parsed) = text_val.trim().parse::<f32>() {
                let clamped = parsed.clamp(start, end);
                if (clamped - *value).abs() > 0.0001 {
                    *value = clamped;
                    *is_dirty = true;
                }
            }
        }
    });
}

/// Precision slider row for integer values (e.g. regions 1..30).
pub fn precision_slider_u32(
    ui: &mut Ui,
    label: &str,
    value: &mut u32,
    range: RangeInclusive<u32>,
    default_val: u32,
    is_dirty: &mut bool,
    tooltip: &str,
) {
    let mut float_val = *value as f32;
    let start = *range.start() as f32;
    let end = *range.end() as f32;

    let sec_col = theme_text_secondary(ui.visuals());

    ui.horizontal(|ui| {
        let label_response = ui.add_sized(
            [96.0, 18.0],
            egui::Label::new(
                egui::RichText::new(label)
                    .size(12.0)
                    .color(sec_col),
            ),
        );
        if !tooltip.is_empty() {
            label_response.on_hover_text(tooltip);
        }

        let slider = egui::Slider::new(&mut float_val, start..=end).show_value(false);
        let slider_resp = ui.add(slider);

        if slider_resp.changed() {
            *value = float_val.round() as u32;
            *is_dirty = true;
        }

        if slider_resp.double_clicked() {
            *value = default_val;
            *is_dirty = true;
        }

        let mut text_val = format!("{}", *value);
        let text_edit = ui.add_sized(
            [46.0, 18.0],
            egui::TextEdit::singleline(&mut text_val)
                .font(egui::FontId::monospace(11.0))
                .horizontal_align(egui::Align::RIGHT),
        );

        if text_edit.lost_focus() {
            if let Ok(parsed) = text_val.trim().parse::<u32>() {
                let clamped = parsed.clamp(*range.start(), *range.end());
                if clamped != *value {
                    *value = clamped;
                    *is_dirty = true;
                }
            }
        }
    });
}
