use crate::icons::*;
use crate::theme::*;
use eframe::egui::{self, Ui};

pub fn render_bottom_statusbar(
    ui: &mut Ui,
    dimensions: Option<(usize, usize)>,
    zoom: f32,
    is_rendering: bool,
    last_process_time: f32,
    has_image: bool,
) {
    let sec_col = theme_text_secondary(ui.visuals());
    let mut_col = theme_text_muted(ui.visuals());

    ui.horizontal_centered(|ui| {
        ui.add_space(SPACE_8);

        // Status indicator
        if is_rendering {
            ui.spinner();
            ui.label(
                egui::RichText::new("Processing DSP pipeline...")
                    .size(11.0)
                    .color(ACCENT),
            );
        } else {
            let dot_color = if has_image { COLOR_SUCCESS } else { mut_col };
            let (rect, painter) = ui.allocate_painter(egui::vec2(6.0, 6.0), egui::Sense::hover());
            painter.circle_filled(rect.rect.center(), 3.0, dot_color);
            ui.label(
                egui::RichText::new(if has_image { "Ready" } else { "Idle" })
                    .size(11.0)
                    .color(sec_col),
            );
        }

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // Image Dimensions
        if let Some((w, h)) = dimensions {
            ui.label(
                egui::RichText::new(format!("{} × {} px", w, h))
                    .font(egui::FontId::monospace(11.0))
                    .size(11.0)
                    .color(sec_col),
            );
            ui.add_space(SPACE_8);
            ui.separator();
            ui.add_space(SPACE_8);
        }

        // Color Mode & Pipeline Resolution
        ui.label(
            egui::RichText::new("RGB f32 (Linear Rec.709)")
                .size(11.0)
                .color(mut_col),
        );

        ui.add_space(SPACE_8);
        ui.separator();
        ui.add_space(SPACE_8);

        ui.label(
            egui::RichText::new("Preview: 960px downsample (Interactive)")
                .size(11.0)
                .color(mut_col),
        );

        // Right side info: Zoom % and latency
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(SPACE_12);

            if last_process_time > 0.0 {
                ui.label(
                    egui::RichText::new(format!("Frame latency: {:.1} ms", last_process_time))
                        .font(egui::FontId::monospace(11.0))
                        .size(11.0)
                        .color(sec_col),
                );
                ui.add_space(SPACE_8);
                ui.separator();
                ui.add_space(SPACE_8);
            }

            let zoom_pct = (zoom * 100.0).round() as u32;
            ui.label(
                egui::RichText::new(format!("Zoom: {}%", zoom_pct))
                    .font(egui::FontId::monospace(11.0))
                    .size(11.0)
                    .color(sec_col),
            );
        });
    });
}
