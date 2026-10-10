use crate::theme::*;
use adjust_core::HistogramData;
use eframe::egui::{self, Color32, Ui};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InspectorTab {
    Navigate,
    Loupe,
    Histogram,
}

pub fn render_inspector_panel(
    ui: &mut Ui,
    current_tab: &mut InspectorTab,
    nav_texture: Option<&egui::TextureHandle>,
    proc_texture: Option<&egui::TextureHandle>,
    histogram: Option<&HistogramData>,
    pan: &mut egui::Vec2,
    zoom: f32,
    canvas_size: egui::Vec2,
) {
    ui.add_space(SPACE_4);

    let card_fill = theme_surface_card(ui.visuals());
    let border_col = theme_border(ui.visuals());
    let sec_col = theme_text_secondary(ui.visuals());

    // Segmented Control for Tabs: NAVIGATE | LOUPE | HISTOGRAM
    ui.horizontal(|ui| {
        let tabs = [
            (InspectorTab::Navigate, "NAVIGATE"),
            (InspectorTab::Loupe, "LOUPE"),
            (InspectorTab::Histogram, "HISTOGRAM"),
        ];

        let tab_width = (ui.available_width() - 8.0) / 3.0;

        for (tab, label) in tabs {
            let is_active = *current_tab == tab;
            let btn = ui.add_sized(
                [tab_width, 22.0],
                egui::SelectableLabel::new(
                    is_active,
                    egui::RichText::new(label)
                        .size(11.0)
                        .strong()
                        .color(if is_active { ACCENT } else { sec_col }),
                ),
            );
            if btn.clicked() {
                *current_tab = tab;
            }
        }
    });

    ui.add_space(SPACE_6);

    // Fixed Height Container Card (140px for comfortable photo proportions)
    let card_height = 140.0;
    let card_frame = egui::Frame::new()
        .fill(card_fill)
        .stroke(egui::Stroke::new(1.0_f32, border_col))
        .corner_radius(RADIUS_CONTROL)
        .inner_margin(egui::Margin::same(6));

    card_frame.show(ui, |ui| {
        let card_w = ui.available_width().max(120.0);

        match *current_tab {
            InspectorTab::Navigate => {
                if let Some(tex) = nav_texture {
                    let img_size = tex.size_vec2();
                    let aspect = if img_size.y > 0.0 {
                        img_size.x / img_size.y
                    } else {
                        1.5
                    };

                    // Compute letterboxed/pillarboxed display rect preserving exact aspect ratio
                    let (disp_w, disp_h) = if aspect > card_w / card_height {
                        (card_w, card_w / aspect)
                    } else {
                        (card_height * aspect, card_height)
                    };

                    let (resp, painter) = ui.allocate_painter(
                        egui::vec2(card_w, card_height),
                        egui::Sense::click_and_drag(),
                    );

                    // Dark neutral letterbox backdrop
                    painter.rect_filled(resp.rect, RADIUS_CONTROL, SURFACE_PANEL);

                    let img_rect = egui::Rect::from_center_size(
                        resp.rect.center(),
                        egui::vec2(disp_w, disp_h),
                    );

                    // Render thumbnail with correct aspect ratio
                    painter.image(
                        tex.id(),
                        img_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    painter.rect_stroke(
                        img_rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, BORDER_SUBTLE),
                        egui::StrokeKind::Inside,
                    );

                    // Viewport crop box indicating canvas view area
                    let effective_canvas = if canvas_size.x > 0.0 && canvas_size.y > 0.0 {
                        canvas_size
                    } else {
                        egui::vec2(900.0, 600.0)
                    };

                    let scaled_img_size = img_size * zoom;
                    let norm_vis_w = (effective_canvas.x / scaled_img_size.x).clamp(0.05, 1.0);
                    let norm_vis_h = (effective_canvas.y / scaled_img_size.y).clamp(0.05, 1.0);

                    let box_w = (disp_w * norm_vis_w).clamp(16.0, disp_w);
                    let box_h = (disp_h * norm_vis_h).clamp(12.0, disp_h);

                    // Pan offset mapping: on canvas, img center = canvas_center + pan
                    // So canvas viewport is offset by -pan relative to image center
                    let norm_pan_x = -pan.x / scaled_img_size.x.max(1.0);
                    let norm_pan_y = -pan.y / scaled_img_size.y.max(1.0);

                    let crop_cx = img_rect.center().x + norm_pan_x * disp_w;
                    let crop_cy = img_rect.center().y + norm_pan_y * disp_h;
                    let crop_rect = egui::Rect::from_center_size(
                        egui::pos2(crop_cx, crop_cy),
                        egui::vec2(box_w, box_h),
                    );

                    // Draw styled viewport crop box
                    painter.rect_stroke(
                        crop_rect,
                        2.0,
                        egui::Stroke::new(2.0_f32, ACCENT),
                        egui::StrokeKind::Middle,
                    );
                    // Subtle translucent interior highlight
                    painter.rect_filled(
                        crop_rect,
                        2.0,
                        Color32::from_rgba_premultiplied(0, 30, 40, 25),
                    );

                    // Interactive dragging inside Navigator to pan the canvas
                    if resp.dragged() {
                        let delta = resp.drag_delta();
                        // Moving crop box by delta means panning canvas in opposite relative direction
                        let pan_scale_x = scaled_img_size.x / disp_w.max(1.0);
                        let pan_scale_y = scaled_img_size.y / disp_h.max(1.0);
                        pan.x -= delta.x * pan_scale_x;
                        pan.y -= delta.y * pan_scale_y;
                    } else if resp.clicked() {
                        // Click to center viewport at clicked location
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if img_rect.contains(pos) {
                                let offset_x = pos.x - img_rect.center().x;
                                let offset_y = pos.y - img_rect.center().y;
                                let pan_scale_x = scaled_img_size.x / disp_w.max(1.0);
                                let pan_scale_y = scaled_img_size.y / disp_h.max(1.0);
                                pan.x = -offset_x * pan_scale_x;
                                pan.y = -offset_y * pan_scale_y;
                            }
                        }
                    }
                } else {
                    render_empty_placeholder(ui, card_w, card_height, "No preview available");
                }
            }
            InspectorTab::Loupe => {
                if let Some(tex) = proc_texture {
                    let img_size = tex.size_vec2();
                    let aspect = if img_size.y > 0.0 {
                        img_size.x / img_size.y
                    } else {
                        1.5
                    };

                    let (disp_w, disp_h) = if aspect > card_w / card_height {
                        (card_w, card_w / aspect)
                    } else {
                        (card_height * aspect, card_height)
                    };

                    let (resp, painter) = ui.allocate_painter(
                        egui::vec2(card_w, card_height),
                        egui::Sense::hover(),
                    );

                    painter.rect_filled(resp.rect, RADIUS_CONTROL, SURFACE_PANEL);

                    let loupe_rect = egui::Rect::from_center_size(
                        resp.rect.center(),
                        egui::vec2(disp_w, disp_h),
                    );

                    // 400% center magnification
                    let loupe_uv = egui::Rect::from_min_max(
                        egui::pos2(0.375, 0.375),
                        egui::pos2(0.625, 0.625),
                    );
                    painter.image(tex.id(), loupe_rect, loupe_uv, Color32::WHITE);
                    painter.rect_stroke(
                        loupe_rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, BORDER_SUBTLE),
                        egui::StrokeKind::Inside,
                    );

                    // Crosshair overlay
                    let center = loupe_rect.center();
                    painter.line_segment(
                        [egui::pos2(center.x - 10.0, center.y), egui::pos2(center.x + 10.0, center.y)],
                        egui::Stroke::new(1.5_f32, ACCENT),
                    );
                    painter.line_segment(
                        [egui::pos2(center.x, center.y - 10.0), egui::pos2(center.x, center.y + 10.0)],
                        egui::Stroke::new(1.5_f32, ACCENT),
                    );

                    painter.text(
                        egui::pos2(loupe_rect.min.x + 6.0, loupe_rect.min.y + 6.0),
                        egui::Align2::LEFT_TOP,
                        "400% Target",
                        egui::FontId::monospace(10.5),
                        TEXT_SECONDARY,
                    );
                } else {
                    render_empty_placeholder(ui, card_w, card_height, "No image loaded");
                }
            }
            InspectorTab::Histogram => {
                if let Some(hist) = histogram {
                    render_histogram_graph(ui, hist, card_w, card_height);
                } else {
                    render_empty_placeholder(ui, card_w, card_height, "Computing histogram...");
                }
            }
        }
    });

    ui.add_space(SPACE_4);
}

fn render_empty_placeholder(ui: &mut Ui, width: f32, height: f32, message: &str) {
    let (resp, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let panel_bg = theme_surface_panel(ui.visuals());
    let text_mut = theme_text_muted(ui.visuals());
    painter.rect_filled(resp.rect, RADIUS_CONTROL, panel_bg);
    painter.text(
        resp.rect.center(),
        egui::Align2::CENTER_CENTER,
        message,
        egui::FontId::proportional(11.0),
        text_mut,
    );
}

fn render_histogram_graph(ui: &mut Ui, hist: &HistogramData, width: f32, height: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = response.rect;

    let panel_bg = theme_surface_panel(ui.visuals());
    let text_mut = theme_text_muted(ui.visuals());
    let border_col = theme_border(ui.visuals());

    // Background fill
    painter.rect_filled(rect, RADIUS_CONTROL, panel_bg);

    // Max values for normalization
    let max_val = hist.lum.iter().copied().max().unwrap_or(1).max(1) as f32;

    // Draw grid lines at 25%, 50%, 75%
    for factor in [0.25, 0.50, 0.75] {
        let x = rect.min.x + factor * rect.width();
        painter.line_segment(
            [egui::pos2(x, rect.min.y + 2.0), egui::pos2(x, rect.max.y - 2.0)],
            egui::Stroke::new(1.0_f32, border_col),
        );
    }

    // Draw Luminance spline/bars
    let bar_color = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(232, 234, 240, 150)
    } else {
        Color32::from_rgba_unmultiplied(40, 48, 60, 180)
    };

    for i in 0..255 {
        let x0 = rect.min.x + (i as f32 / 255.0) * rect.width();
        let x1 = rect.min.x + ((i + 1) as f32 / 255.0) * rect.width();

        let lum_h = (hist.lum[i] as f32 / max_val) * (height - 6.0);
        let y0 = rect.max.y - 3.0;
        let y1 = y0 - lum_h;

        painter.line_segment(
            [egui::pos2(x0, y0), egui::pos2(x1, y1)],
            egui::Stroke::new(1.0_f32, bar_color),
        );
    }

    // Histogram labels
    painter.text(
        egui::pos2(rect.min.x + 4.0, rect.max.y - 4.0),
        egui::Align2::LEFT_BOTTOM,
        "0",
        egui::FontId::monospace(9.0),
        text_mut,
    );
    painter.text(
        egui::pos2(rect.max.x - 4.0, rect.max.y - 4.0),
        egui::Align2::RIGHT_BOTTOM,
        "255",
        egui::FontId::monospace(9.0),
        text_mut,
    );
}
