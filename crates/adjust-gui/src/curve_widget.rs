use adjust_core::{generate_curve_lut, CurveParams, CurvePoint, HistogramData};
use eframe::egui;

/// Interactive Graphical Curve Editor with Spline Visualization, Drag Points, and Presets.
pub fn render_curve_editor(
    ui: &mut egui::Ui,
    params: &mut CurveParams,
    hist: Option<&HistogramData>,
    is_dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        if ui.checkbox(&mut params.enabled, "Enable Curves").changed() {
            *is_dirty = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("↺ Reset").clicked() {
                *params = CurveParams::linear();
                *is_dirty = true;
            }
        });
    });

    let width = ui.available_width().max(160.0);
    let height = 150.0;
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::click_and_drag());
    let rect = response.rect;

    // 1. Dark Canvas Background
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 19, 23));
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 48, 56)), egui::StrokeKind::Inside);

    // 2. Faint Background Histogram (if available)
    if let Some(h) = hist {
        let max_val = h.lum.iter().copied().max().unwrap_or(1).max(1) as f32;
        for i in (0..255).step_by(2) {
            let x0 = rect.min.x + (i as f32 / 255.0) * rect.width();
            let x1 = rect.min.x + ((i + 2) as f32 / 255.0) * rect.width();
            let bar_h = (h.lum[i] as f32 / max_val) * (height - 8.0);
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(x0, rect.max.y - bar_h - 2.0),
                egui::pos2(x1, rect.max.y - 2.0),
            );
            painter.rect_filled(bar_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18));
        }
    }

    // 3. Grid Lines (0.25, 0.50, 0.75)
    let grid_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25));
    for step in [0.25, 0.50, 0.75] {
        let gx = rect.min.x + step * rect.width();
        let gy = rect.max.y - step * rect.height();
        painter.line_segment([egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)], grid_stroke);
        painter.line_segment([egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)], grid_stroke);
    }

    // 4. Diagonal Reference Line
    painter.line_segment(
        [egui::pos2(rect.min.x, rect.max.y), egui::pos2(rect.max.x, rect.min.y)],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(100, 105, 120, 70)),
    );

    // 5. Render Curve Spline Line
    let lut = generate_curve_lut(&params.points);
    let curve_color = if params.enabled {
        egui::Color32::from_rgb(0, 210, 255) // Cyan
    } else {
        egui::Color32::from_rgb(120, 130, 145) // Muted
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

        painter.line_segment([pt0, pt1], egui::Stroke::new(2.0_f32, curve_color));
    }

    // 6. Interactive Control Points Handling
    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if rect.contains(mouse_pos) {
            let rel_x = ((mouse_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let rel_y = ((rect.max.y - mouse_pos.y) / rect.height()).clamp(0.0, 1.0);

            // Drag existing point
            if response.dragged_by(egui::PointerButton::Primary) {
                // Find closest point within interaction distance
                let mut closest_idx = None;
                let mut min_dist = 20.0; // pixel distance threshold

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

                    // Update point
                    if idx == 0 {
                        params.points[idx].y = rel_y; // Keep x=0.0
                    } else if idx == params.points.len() - 1 {
                        params.points[idx].y = rel_y; // Keep x=1.0
                    } else {
                        params.points[idx].x = rel_x;
                        params.points[idx].y = rel_y;
                    }

                    if (params.points[idx].x - old_x).abs() > 0.001 || (params.points[idx].y - old_y).abs() > 0.001 {
                        params.enabled = true;
                        *is_dirty = true;
                    }
                }
            } else if response.clicked_by(egui::PointerButton::Primary) {
                // Click on curve to add a new point if not clicking on existing point
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
                // Right click to delete an intermediate point
                if params.points.len() > 2 {
                    let mut del_idx = None;
                    for (idx, pt) in params.points.iter().enumerate() {
                        if idx == 0 || idx == params.points.len() - 1 {
                            continue; // Don't delete end anchors
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
            (egui::Color32::from_rgb(0, 180, 240), 4.5)
        } else {
            (egui::Color32::WHITE, 5.5)
        };

        painter.circle_filled(screen_pt, radius + 1.5, egui::Color32::from_rgba_unmultiplied(0, 210, 255, 90));
        painter.circle_filled(screen_pt, radius, fill_col);
    }

    // 8. Curve Style Presets (Quick Chips)
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.label("Presets:");
        if ui.button("Linear").clicked() {
            *params = CurveParams::linear();
            *is_dirty = true;
        }
        if ui.button("Med S-Curve").clicked() {
            *params = CurveParams::medium_contrast();
            *is_dirty = true;
        }
        if ui.button("High S-Curve").clicked() {
            *params = CurveParams::high_contrast();
            *is_dirty = true;
        }
        if ui.button("Lift Shadows").clicked() {
            *params = CurveParams::lift_shadows();
            *is_dirty = true;
        }
        if ui.button("Matte Film").clicked() {
            *params = CurveParams::matte_film();
            *is_dirty = true;
        }
    });
}
