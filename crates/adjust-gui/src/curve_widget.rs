//! Tone curve editor with master (RGB) and per-channel curves.
//!
//! Click to add a point, drag to move it, right-click to remove it.

use crate::theme::*;
use crate::widgets::controls::segmented;
use adjust_core::{generate_curve_lut, CurveParams, CurvePoint, HistogramData};
use eframe::egui::{self, Color32, Rect, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CurveChannel {
    Rgb,
    Red,
    Green,
    Blue,
}

fn channel_points(params: &mut CurveParams, ch: CurveChannel) -> &mut Vec<CurvePoint> {
    match ch {
        CurveChannel::Rgb => &mut params.points,
        CurveChannel::Red => &mut params.red,
        CurveChannel::Green => &mut params.green,
        CurveChannel::Blue => &mut params.blue,
    }
}

/// Returns true when the curve changed.
pub fn curve_editor(
    ui: &mut egui::Ui,
    params: &mut CurveParams,
    hist: Option<&HistogramData>,
    channel: &mut CurveChannel,
) -> bool {
    let p = pal(ui);
    let mut changed = false;

    ui.horizontal(|ui| {
        segmented(
            ui,
            channel,
            &[
                (CurveChannel::Rgb, "RGB", "Master curve"),
                (CurveChannel::Red, "R", "Red channel"),
                (CurveChannel::Green, "G", "Green channel"),
                (CurveChannel::Blue, "B", "Blue channel"),
            ],
            42.0,
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Label::new(egui::RichText::new("Reset").size(11.5).color(p.accent)).sense(egui::Sense::click()))
                .on_hover_text("Reset this channel to a straight line")
                .clicked()
            {
                let pts = channel_points(params, *channel);
                if *channel == CurveChannel::Rgb {
                    *pts = CurveParams::linear().points;
                } else {
                    pts.clear();
                }
                changed = true;
            }
        });
    });
    ui.add_space(SPACE_6);

    let ch = *channel;
    let line_color = match ch {
        CurveChannel::Rgb => p.text,
        CurveChannel::Red => Color32::from_rgb(255, 95, 95),
        CurveChannel::Green => Color32::from_rgb(80, 210, 110),
        CurveChannel::Blue => Color32::from_rgb(90, 160, 255),
    };

    let size = ui.available_width().min(280.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(size, size), egui::Sense::click_and_drag());
    let rect = response.rect;
    painter.rect_filled(rect, RADIUS_CONTROL, p.card);

    // Histogram backdrop
    if let Some(h) = hist {
        let bins: &[u32; 256] = match ch {
            CurveChannel::Rgb => &h.lum,
            CurveChannel::Red => &h.r,
            CurveChannel::Green => &h.g,
            CurveChannel::Blue => &h.b,
        };
        let max = bins.iter().copied().max().unwrap_or(1).max(1) as f32;
        let fill = with_alpha(line_color, if p.dark { 34 } else { 40 });
        for i in (0..256).step_by(2) {
            let x0 = rect.min.x + i as f32 / 256.0 * rect.width();
            let x1 = rect.min.x + (i + 2) as f32 / 256.0 * rect.width();
            let hgt = (bins[i] as f32 / max).sqrt() * (rect.height() - 6.0);
            painter.rect_filled(
                Rect::from_min_max(egui::pos2(x0, rect.max.y - hgt), egui::pos2(x1, rect.max.y)),
                0.0,
                fill,
            );
        }
    }

    // Grid and diagonal
    let grid = Stroke::new(1.0_f32, p.separator);
    for t in [0.25, 0.5, 0.75] {
        let x = rect.min.x + t * rect.width();
        let y = rect.max.y - t * rect.height();
        painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], grid);
        painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], grid);
    }
    painter.line_segment([rect.left_bottom(), rect.right_top()], Stroke::new(1.0_f32, p.text_tertiary));

    let to_screen = |pt: &CurvePoint| egui::pos2(rect.min.x + pt.x * rect.width(), rect.max.y - pt.y * rect.height());

    // Editing works on a copy so merely viewing an empty channel doesn't modify it.
    let stored = channel_points(params, ch);
    let mut work = if stored.len() < 2 { CurveParams::linear().points } else { stored.clone() };
    let points = &mut work;
    let mut readout = None;
    if let Some(mouse) = response.interact_pointer_pos() {
        let rel = egui::vec2(
            ((mouse.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
            ((rect.max.y - mouse.y) / rect.height()).clamp(0.0, 1.0),
        );
        let nearest = points
            .iter()
            .enumerate()
            .map(|(i, pt)| (i, (to_screen(pt) - mouse).length()))
            .min_by(|a, b| a.1.total_cmp(&b.1));

        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some((i, _)) = nearest.filter(|(_, d)| *d < 24.0) {
                let last = points.len() - 1;
                // Endpoints only move vertically; others stay between neighbours.
                if i != 0 && i != last {
                    points[i].x = rel.x.clamp(points[i - 1].x + 0.01, points[i + 1].x - 0.01);
                }
                points[i].y = rel.y;
                readout = Some((points[i].x, points[i].y));
                changed = true;
            }
        } else if response.clicked_by(egui::PointerButton::Primary) {
            if nearest.is_none_or(|(_, d)| d > 12.0) && points.len() < 12 {
                points.push(CurvePoint { x: rel.x, y: rel.y });
                points.sort_by(|a, b| a.x.total_cmp(&b.x));
                changed = true;
            }
        } else if response.clicked_by(egui::PointerButton::Secondary) {
            if let Some((i, _)) = nearest.filter(|(i, d)| *d < 12.0 && *i != 0 && *i != points.len() - 1) {
                points.remove(i);
                changed = true;
            }
        }
    }

    // Curve
    let lut = generate_curve_lut(points);
    let curve: Vec<egui::Pos2> = (0..=96)
        .map(|i| {
            let t = i as f32 / 96.0;
            let y = lut[((t * 1023.0).round() as usize).min(1023)];
            egui::pos2(rect.min.x + t * rect.width(), rect.max.y - y * rect.height())
        })
        .collect();
    painter.add(egui::Shape::line(curve, Stroke::new(2.0_f32, line_color)));
    for pt in points.iter() {
        let c = to_screen(pt);
        painter.circle_filled(c, 5.5, p.knob);
        painter.circle_stroke(c, 5.5, Stroke::new(1.5_f32, line_color));
    }
    if let Some((x, y)) = readout {
        painter.text(
            rect.left_top() + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            format!("In {:.0}   Out {:.0}", x * 255.0, y * 255.0),
            egui::FontId::monospace(11.0),
            p.text_secondary,
        );
    }
    response.on_hover_text("Click to add a point · drag to move · right-click to remove");
    if changed {
        // An identity channel curve is stored as "no curve" so it matches the defaults.
        let identity = work == CurveParams::linear().points;
        *channel_points(params, ch) = if identity && ch != CurveChannel::Rgb { Vec::new() } else { work };
    }

    ui.add_space(SPACE_6);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (label, preset) in [
            ("Linear", CurveParams::linear()),
            ("Medium", CurveParams::medium_contrast()),
            ("Strong", CurveParams::high_contrast()),
            ("Lift", CurveParams::lift_shadows()),
            ("Matte", CurveParams::matte_film()),
        ] {
            if ui.small_button(label).on_hover_text("Apply this master curve shape").clicked() {
                params.points = preset.points;
                changed = true;
            }
        }
    });

    if changed {
        params.enabled = true;
    }
    changed
}
