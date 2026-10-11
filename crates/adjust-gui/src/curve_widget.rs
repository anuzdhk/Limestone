use crate::theme::*;
use adjust_core::{generate_curve_lut, CurveParams, CurvePoint, HistogramData};
use eframe::egui::{self, Color32, Rect, Stroke};

#[derive(Clone, Copy, PartialEq)]
pub enum CurveChannel {
    Rgb,
    Red,
    Green,
    Blue,
}

/// Interactive Graphical Curve Editor with Spline Visualization, Drag Points,
/// channel selectors (RGB/R/G/B), live coordinate tooltips, and presets.
pub fn render_curve_editor(
    ui: &mut egui::Ui,
    params: &mut CurveParams,
    hist: Option<&HistogramData>,
    active_channel: &mut CurveChannel,
    is_dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        // Channel segmented pill selector
        ui.selectable_value(active_channel, CurveChannel::Rgb, "RGB");
        ui.selectable_value(active_channel, CurveChannel::Red, "R");
        ui.selectable_value(active_channel, CurveChannel::Green, "G");
        ui.selectable_value(active_channel, CurveChannel::Blue, "B");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("↺ Reset").on_hover_text("Reset curve to linear").clicked() {
                *params = CurveParams::linear();
                *is_dirty = true;
            }
        });
    });

    ui.add_space(SPACE_4);

    // Square 220x220 curve canvas
    let width = ui.available_width().clamp(180.0, 260.0);
    let height = width;
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::click_and_drag());
    let rect = response.rect;

    // 1. Canvas Background
    let card_bg = theme_surface_card(ui.visuals());
    let border_col = theme_border(ui.visuals());
    painter.rect_filled(rect, RADIUS_CONTROL, card_bg);
    painter.rect_stroke(rect, RADIUS_CONTROL, Stroke::new(1.0_f32, border_col), egui::StrokeKind::Inside);

    // 2. Faint Background Histogram
    if let Some(h) = hist {
        let max_val = match active_channel {
            CurveChannel::Rgb => h.lum.iter().copied().max().unwrap_or(1).max(1) as f32,
            CurveChannel::Red => h.r.iter().copied().max().unwrap_or(1).max(1) as f32,
            CurveChannel::Green => h.g.iter().copied().max().unwrap_or(1).max(1) as f32,
            CurveChannel::Blue => h.b.iter().copied().max().unwrap_or(1).max(1) as f32,
        };

        let hist_tint = match active_channel {
            CurveChannel::Rgb => if ui.visuals().dark_mode {
                Color32::from_rgba_unmultiplied(220, 225, 235, 24)
            } else {
                Color32::from_rgba_unmultiplied(60, 70, 90, 28)
            },
            CurveChannel::Red => Color32::from_rgba_unmultiplied(255, 100, 100, 32),
            CurveChannel::Green => Color32::from_rgba_unmultiplied(100, 255, 120, 32),
            CurveChannel::Blue => Color32::from_rgba_unmultiplied(100, 180, 255, 32),
        };

        for i in (0..255).step_by(2) {
            let x0 = rect.min.x + (i as f32 / 255.0) * rect.width();
            let x1 = rect.min.x + ((i + 2) as f32 / 255.0) * rect.width();
            let count = match active_channel {
                CurveChannel::Rgb => h.lum[i],
                CurveChannel::Red => h.r[i],
                CurveChannel::Green => h.g[i],
                CurveChannel::Blue => h.b[i],
            } as f32;

            let bar_h = (count / max_val) * (height - 8.0);
            let bar_rect = Rect::from_min_max(
                egui::pos2(x0, rect.max.y - bar_h - 2.0),
                egui::pos2(x1, rect.max.y - 2.0),
            );
            painter.rect_filled(bar_rect, 0.0, hist_tint);
        }
    }

    // 3. Grid Lines (25% divisions)
    let grid_color = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(255, 255, 255, 18)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 22)
    };
    let grid_stroke = Stroke::new(1.0_f32, grid_color);
    for step in [0.25, 0.50, 0.75] {
        let gx = rect.min.x + step * rect.width();
        let gy = rect.max.y - step * rect.height();
        painter.line_segment([egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)], grid_stroke);
        painter.line_segment([egui::pos2(rect.min.x, gy), egui::pos2(rect.max.y, gy)], grid_stroke);
    }

    // 4. Diagonal 1:1 Reference Line
    painter.line_segment(
        [egui::pos2(rect.min.x, rect.max.y), egui::pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(154, 163, 178, 45)),
    );

    // 5. Render Curve Spline
    let lut = generate_curve_lut(&params.points);
    let curve_color = if params.enabled {
        match active_channel {
            CurveChannel::Rgb => ACCENT,
            CurveChannel::Red => Color32::from_rgb(255, 95, 95),
            CurveChannel::Green => Color32::from_rgb(80, 230, 110),
            CurveChannel::Blue => Color32::from_rgb(90, 160, 255),
        }
    } else {
        TEXT_DISABLED
    };

    let segments = 64;
    for i in 0..segments {
        let t0 = i as f32 / segments as f32;
        let t1 = (i + 1) as f32 / segments as f32;

        let lut_idx0 = (t0 * 1023.0).round() as usize;
        let lut_idx1 = (t1 * 1023.0).round() as usize;

        let y0 = lut[lut_idx0.min(1023)];
        let y1 = lut[lut_idx1.min(1023)];

        let pt0 = egui::pos2(rect.min.x + t0 * rect.width(), rect.max.y - y0 * rect.height());
        let pt1 = egui::pos2(rect.min.x + t1 * rect.width(), rect.max.y - y1 * rect.height());

        painter.line_segment([pt0, pt1], Stroke::new(2.0_f32, curve_color));
    }

    // 6. Interactive Control Points Handling
    let mut hovered_pt_coords: Option<(f32, f32)> = None;

    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if rect.contains(mouse_pos) {
            let rel_x = ((mouse_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let rel_y = ((rect.max.y - mouse_pos.y) / rect.height()).clamp(0.0, 1.0);

            // Drag existing point
            if response.dragged_by(egui::PointerButton::Primary) {
                let mut closest_idx = None;
                let mut min_dist = 22.0;

                for (idx, pt) in params.points.iter().enumerate() {
                    let screen_pt = egui::pos2(
                        rect.min.x + pt.x * rect.width(),
                        rect.max.y - pt.y * rect.height(),
                    );
                    let dist = (screen_pt - mouse_pos).length();
                    if dist < min_dist {
                        min_dist = dist;
                        closest_idx = Some(idx);
                    }
                }

                if let Some(idx) = closest_idx {
                    let old_x = params.points[idx].x;
                    let old_y = params.points[idx].y;

                    // Endpoints are pinned horizontally; only their output level moves.
                    if idx == 0 || idx == params.points.len() - 1 {
                        params.points[idx].y = rel_y;
                    } else {
                        params.points[idx].x = rel_x;
                        params.points[idx].y = rel_y;
                    }

                    hovered_pt_coords = Some((params.points[idx].x, params.points[idx].y));

                    if (params.points[idx].x - old_x).abs() > 0.001 || (params.points[idx].y - old_y).abs() > 0.001 {
                        params.enabled = true;
                        *is_dirty = true;
                    }
                }
            } else if response.clicked_by(egui::PointerButton::Primary) {
                // Click on curve to add a new point
                let on_existing = params.points.iter().any(|pt| {
                    let screen_pt = egui::pos2(
                        rect.min.x + pt.x * rect.width(),
                        rect.max.y - pt.y * rect.height(),
                    );
                    (screen_pt - mouse_pos).length() < 14.0
                });

                if !on_existing && params.points.len() < 8 {
                    params.points.push(CurvePoint { x: rel_x, y: rel_y });
                    params.points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
                    params.enabled = true;
                    *is_dirty = true;
                }
            } else if response.clicked_by(egui::PointerButton::Secondary) {
                // Right click to delete intermediate point
                if params.points.len() > 2 {
                    let mut del_idx = None;
                    for (idx, pt) in params.points.iter().enumerate() {
                        if idx == 0 || idx == params.points.len() - 1 {
                            continue;
                        }
                        let screen_pt = egui::pos2(
                            rect.min.x + pt.x * rect.width(),
                            rect.max.y - pt.y * rect.height(),
                        );
                        if (screen_pt - mouse_pos).length() < 14.0 {
                            del_idx = Some(idx);
                            break;
                        }
                    }

                    if let Some(idx) = del_idx {
                        params.points.remove(idx);
                        *is_dirty = true;
                    }
                }
            }
        }
    }

    // 7. Draw Point Markers
    for (idx, pt) in params.points.iter().enumerate() {
        let screen_pt = egui::pos2(
            rect.min.x + pt.x * rect.width(),
            rect.max.y - pt.y * rect.height(),
        );

        let (fill_col, radius) = if idx == 0 || idx == params.points.len() - 1 {
            (ACCENT, 4.5)
        } else {
            (Color32::WHITE, 5.5)
        };

        painter.circle_filled(screen_pt, radius + 2.0, ACCENT_MUTED);
        painter.circle_filled(screen_pt, radius, fill_col);
    }

    // 8. Coordinates readout when dragging
    if let Some((x, y)) = hovered_pt_coords {
        let coord_text = format!("In: {:.2}  Out: {:.2}", x, y);
        painter.text(
            egui::pos2(rect.min.x + 8.0, rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            coord_text,
            egui::FontId::monospace(11.0),
            TEXT_PRIMARY,
        );
    }

    // 9. Curve Quick Shape Presets
    ui.add_space(SPACE_4);
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Shapes:").size(11.5).color(TEXT_SECONDARY));
        if ui.button("Linear").clicked() {
            *params = CurveParams::linear();
            *is_dirty = true;
        }
        if ui.button("Med S").clicked() {
            *params = CurveParams::medium_contrast();
            *is_dirty = true;
        }
        if ui.button("High S").clicked() {
            *params = CurveParams::high_contrast();
            *is_dirty = true;
        }
        if ui.button("Lift").clicked() {
            *params = CurveParams::lift_shadows();
            *is_dirty = true;
        }
        if ui.button("Matte").clicked() {
            *params = CurveParams::matte_film();
            *is_dirty = true;
        }
    });
}
