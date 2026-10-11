//! Apple-style slider rows: label + value on top, thin track with a round knob below.
//!
//! - Double-click the track (or the value) to reset to the default.
//! - Hold Shift while dragging for 10x finer control.
//! - Bipolar ranges (e.g. -1..1) fill from the default outward, like Photos.

use crate::theme::*;
use eframe::egui::{self, Color32, Sense, Stroke, Ui};
use std::ops::RangeInclusive;

/// What the track looks like.
#[derive(Clone, Copy, PartialEq)]
pub enum Track {
    /// Accent fill from the default value to the knob.
    Fill,
    /// Rainbow hue strip (for 0..360 hue sliders).
    Hue,
}

pub struct SliderSpec<'a> {
    pub label: &'a str,
    pub range: RangeInclusive<f32>,
    pub default: f32,
    pub decimals: usize,
    pub suffix: &'a str,
    pub tooltip: &'a str,
    pub track: Track,
}

impl<'a> SliderSpec<'a> {
    pub fn new(label: &'a str, range: RangeInclusive<f32>, default: f32) -> Self {
        Self {
            label,
            range,
            default,
            decimals: 2,
            suffix: "",
            tooltip: "",
            track: Track::Fill,
        }
    }
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    pub fn suffix(mut self, s: &'a str) -> Self {
        self.suffix = s;
        self
    }
    pub fn tooltip(mut self, t: &'a str) -> Self {
        self.tooltip = t;
        self
    }
    pub fn hue(mut self) -> Self {
        self.track = Track::Hue;
        self
    }
}

/// Draws one slider row; returns true when the value changed.
pub fn slider(ui: &mut Ui, value: &mut f32, spec: SliderSpec) -> bool {
    let p = pal(ui);
    let (lo, hi) = (*spec.range.start(), *spec.range.end());
    let span = (hi - lo).max(1e-6);
    let mut changed = false;
    let modified = (*value - spec.default).abs() > 1e-4;

    // Header: label left, value right.
    let header = ui.horizontal(|ui| {
        let label = ui.label(egui::RichText::new(spec.label).size(12.5).color(p.text));
        if !spec.tooltip.is_empty() {
            label.on_hover_text(spec.tooltip);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Avoid printing "-0.00" for tiny negative values.
            let shown = if value.abs() < 0.5 * 10f32.powi(-(spec.decimals as i32)) { 0.0 } else { *value };
            let text = format!("{:.*}{}", spec.decimals, shown, spec.suffix);
            let color = if modified { p.text } else { p.text_tertiary };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(text).size(12.0).color(color).monospace(),
                )
                .sense(Sense::click()),
            )
            .on_hover_text("Double-click to reset")
        })
        .inner
    });
    if header.inner.double_clicked() {
        *value = spec.default;
        changed = true;
    }

    // Track
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 18.0), Sense::click_and_drag());
    let knob_r = 7.5;
    let track_rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width() - knob_r * 2.0, 4.0),
    );
    let to_x = |v: f32| track_rect.left() + (v - lo) / span * track_rect.width();

    if resp.dragged() || resp.drag_started() {
        let fine = ui.input(|i| i.modifiers.shift);
        if fine {
            let dx = resp.drag_delta().x;
            *value = (*value + dx / track_rect.width() * span * 0.1).clamp(lo, hi);
        } else if let Some(pos) = resp.interact_pointer_pos() {
            *value = (lo + (pos.x - track_rect.left()) / track_rect.width() * span).clamp(lo, hi);
        }
        changed = true;
    }
    if resp.double_clicked() {
        *value = spec.default;
        changed = true;
    }
    if !spec.tooltip.is_empty() {
        resp.clone().on_hover_text(spec.tooltip);
    }

    let painter = ui.painter_at(rect.expand(2.0));
    match spec.track {
        Track::Fill => {
            painter.rect_filled(track_rect, 2.0, p.track);
            let (a, b) = {
                let (x0, x1) = (to_x(spec.default.clamp(lo, hi)), to_x(*value));
                (x0.min(x1), x0.max(x1))
            };
            let fill = egui::Rect::from_x_y_ranges(a..=b, track_rect.y_range());
            painter.rect_filled(fill, 2.0, if modified { p.accent } else { p.track });
            // Default tick for bipolar sliders
            if spec.default > lo && spec.default < hi {
                let x = to_x(spec.default);
                painter.line_segment(
                    [egui::pos2(x, track_rect.top() - 3.0), egui::pos2(x, track_rect.bottom() + 3.0)],
                    Stroke::new(1.0_f32, p.text_tertiary),
                );
            }
        }
        Track::Hue => {
            let steps = 36;
            for i in 0..steps {
                let t0 = i as f32 / steps as f32;
                let t1 = (i + 1) as f32 / steps as f32;
                let seg = egui::Rect::from_x_y_ranges(
                    (track_rect.left() + t0 * track_rect.width())..=(track_rect.left() + t1 * track_rect.width()),
                    (track_rect.top() - 1.0)..=(track_rect.bottom() + 1.0),
                );
                painter.rect_filled(seg, 0.0, hue_color(lo + (t0 + t1) * 0.5 * span));
            }
        }
    }

    // Knob with soft shadow
    let knob = egui::pos2(to_x(*value), rect.center().y);
    let hovered = resp.hovered() || resp.dragged();
    painter.circle_filled(knob + egui::vec2(0.0, 1.0), knob_r + 1.0, p.shadow);
    painter.circle_filled(knob, knob_r, p.knob);
    if spec.track == Track::Hue {
        painter.circle_filled(knob, knob_r - 3.0, hue_color(*value));
    }
    if hovered {
        painter.circle_stroke(knob, knob_r, Stroke::new(1.5_f32, p.accent));
    }

    ui.add_space(2.0);
    changed
}

/// Integer slider (e.g. regions 1..30).
pub fn slider_u32(ui: &mut Ui, value: &mut u32, label: &str, range: RangeInclusive<u32>, default: u32, tooltip: &str) -> bool {
    let mut v = *value as f32;
    let spec = SliderSpec::new(label, *range.start() as f32..=*range.end() as f32, default as f32)
        .decimals(0)
        .tooltip(tooltip);
    if slider(ui, &mut v, spec) {
        let new = v.round() as u32;
        if new != *value {
            *value = new;
            return true;
        }
    }
    false
}

fn hue_color(deg: f32) -> Color32 {
    let (r, g, b) = adjust_core::color::hsv_to_rgb((deg / 360.0).rem_euclid(1.0), 0.75, 1.0);
    Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
