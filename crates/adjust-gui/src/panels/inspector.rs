use crate::icons::*;
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
) {
    ui.add_space(SPACE_4);

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
                        .color(if is_active { ACCENT } else { TEXT_SECONDARY }),
                ),
            );
            if btn.clicked() {
                *current_tab = tab;
            }
        }
    });

    ui.add_space(SPACE_6);

    // Inspector Content Area (Fixed Height Card)
    let card_frame = egui::Frame::new()
        .fill(SURFACE_CARD)
        .stroke(egui::Stroke::new(1.0_f32, BORDER_SUBTLE))
        .corner_radius(RADIUS_CONTROL)
        .inner_margin(egui::Margin::same(6));

    card_frame.show(ui, |ui| {
        let available_w = ui.available_width().max(120.0);
        let height = 110.0;

        match *current_tab {
            InspectorTab::Navigate => {
                if let Some(tex) = nav_texture {
                    let (resp, painter) = ui.allocate_painter(egui::vec2(available_w, height), egui::Sense::click_and_drag());
                    
                    // Center and fit texture inside the card
                    painter.image(
                        tex.id(),
                        resp.rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    // Viewport crop box indicating canvas view area
                    let box_w = (resp.rect.width() / zoom.max(1.0)).clamp(20.0, resp.rect.width());
                    let box_h = (resp.rect.height() / zoom.max(1.0)).clamp(16.0, resp.rect.height());

                    let center_offset = *pan * 0.05;
                    let box_center = resp.rect.center() - center_offset;
                    let crop_rect = egui::Rect::from_center_size(box_center, egui::vec2(box_w, box_h));
                    let clamped_crop = crop_rect.intersect(resp.rect);

                    painter.rect_stroke(
                        clamped_crop,
                        0.0,
                        egui::Stroke::new(1.5_f32, ACCENT),
                        egui::StrokeKind::Inside,
                    );

                    if resp.dragged() {
                        *pan -= resp.drag_delta() * 15.0;
                    }
                } else {
                    render_empty_placeholder(ui, available_w, height, "No preview available");
                }
            }
            InspectorTab::Loupe => {
                if let Some(tex) = proc_texture {
                    let (resp, painter) = ui.allocate_painter(egui::vec2(available_w, height), egui::Sense::hover());
                    // 400% center magnification
                    let loupe_uv = egui::Rect::from_min_max(egui::pos2(0.375, 0.375), egui::pos2(0.625, 0.625));
                    painter.image(tex.id(), resp.rect, loupe_uv, Color32::WHITE);
                    
                    // Crosshair overlay
                    let center = resp.rect.center();
                    painter.line_segment(
                        [egui::pos2(center.x - 8.0, center.y), egui::pos2(center.x + 8.0, center.y)],
                        egui::Stroke::new(1.0_f32, ACCENT),
                    );
                    painter.line_segment(
                        [egui::pos2(center.x, center.y - 8.0), egui::pos2(center.x, center.y + 8.0)],
                        egui::Stroke::new(1.0_f32, ACCENT),
                    );

                    painter.text(
                        egui::pos2(resp.rect.min.x + 4.0, resp.rect.min.y + 4.0),
                        egui::Align2::LEFT_TOP,
                        "400% Zoom",
                        egui::FontId::proportional(10.0),
                        TEXT_SECONDARY,
                    );
                } else {
                    render_empty_placeholder(ui, available_w, height, "No image loaded");
                }
            }
            InspectorTab::Histogram => {
                if let Some(hist) = histogram {
                    render_histogram_graph(ui, hist, available_w, height);
                } else {
                    render_empty_placeholder(ui, available_w, height, "Computing histogram...");
                }
            }
        }
    });

    ui.add_space(SPACE_4);
}

fn render_empty_placeholder(ui: &mut Ui, width: f32, height: f32, message: &str) {
    let (resp, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    painter.rect_filled(resp.rect, RADIUS_CONTROL, SURFACE_PANEL);
    painter.text(
        resp.rect.center(),
        egui::Align2::CENTER_CENTER,
        message,
        egui::FontId::proportional(11.0),
        TEXT_MUTED,
    );
}

fn render_histogram_graph(ui: &mut Ui, hist: &HistogramData, width: f32, height: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = response.rect;

    // Background fill
    painter.rect_filled(rect, RADIUS_CONTROL, SURFACE_PANEL);

    // Max values for normalization
    let max_val = hist.lum.iter().copied().max().unwrap_or(1).max(1) as f32;

    // Draw grid lines at 25%, 50%, 75%
    for factor in [0.25, 0.50, 0.75] {
        let x = rect.min.x + factor * rect.width();
        painter.line_segment(
            [egui::pos2(x, rect.min.y + 2.0), egui::pos2(x, rect.max.y - 2.0)],
            egui::Stroke::new(1.0_f32, Color32::from_rgb(32, 35, 42)),
        );
    }

    // Draw Luminance spline/bars
    for i in 0..255 {
        let x0 = rect.min.x + (i as f32 / 255.0) * rect.width();
        let x1 = rect.min.x + ((i + 1) as f32 / 255.0) * rect.width();

        let lum_h = (hist.lum[i] as f32 / max_val) * (height - 6.0);
        let y0 = rect.max.y - 3.0;
        let y1 = y0 - lum_h;

        painter.line_segment(
            [egui::pos2(x0, y0), egui::pos2(x1, y1)],
            egui::Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(232, 234, 240, 150)),
        );
    }

    // Histogram labels
    painter.text(
        egui::pos2(rect.min.x + 4.0, rect.max.y - 4.0),
        egui::Align2::LEFT_BOTTOM,
        "0",
        egui::FontId::monospace(9.0),
        TEXT_MUTED,
    );
    painter.text(
        egui::pos2(rect.max.x - 4.0, rect.max.y - 4.0),
        egui::Align2::RIGHT_BOTTOM,
        "255",
        egui::FontId::monospace(9.0),
        TEXT_MUTED,
    );
}
