use crate::icons::*;
use crate::theme::*;
use eframe::egui::{self, Color32, Response, Ui};

#[derive(PartialEq, Clone, Copy)]
pub enum ViewMode {
    Single,
    Split,
    SideBySide,
}

pub struct ToolbarResponse {
    pub open_clicked: bool,
    pub save_preset_clicked: bool,
    pub export_clicked: bool,
    pub undo_clicked: bool,
    pub redo_clicked: bool,
    pub reset_all_clicked: bool,
    pub fit_clicked: bool,
    pub zoom_100_clicked: bool,
    pub shortcuts_clicked: bool,
}

pub fn render_top_toolbar(
    ui: &mut Ui,
    view_mode: &mut ViewMode,
    zoom: &mut f32,
    can_undo: bool,
    can_redo: bool,
    last_process_time: f32,
) -> ToolbarResponse {
    let mut resp = ToolbarResponse {
        open_clicked: false,
        save_preset_clicked: false,
        export_clicked: false,
        undo_clicked: false,
        redo_clicked: false,
        reset_all_clicked: false,
        fit_clicked: false,
        zoom_100_clicked: false,
        shortcuts_clicked: false,
    };

    ui.horizontal_centered(|ui| {
        ui.add_space(SPACE_8);

        // App Brand / Logo
        ui.label(
            egui::RichText::new("LIMESTONE")
                .strong()
                .size(15.0)
                .color(TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new("PRO")
                .strong()
                .size(11.0)
                .color(ACCENT),
        );

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_8);

        // File I/O
        if ui
            .button(format!("{} Open", ICON_FOLDER_OPEN))
            .on_hover_text("Open image (Ctrl+O)")
            .clicked()
        {
            resp.open_clicked = true;
        }

        if ui
            .button(format!("{} Export", ICON_DEVICE_FLOPPY))
            .on_hover_text("Export full-resolution image (Ctrl+E)")
            .clicked()
        {
            resp.export_clicked = true;
        }

        ui.add_space(SPACE_8);
        ui.separator();
        ui.add_space(SPACE_8);

        // Undo / Redo
        ui.add_enabled_ui(can_undo, |ui| {
            if ui
                .button(format!("{}", ICON_ARROW_BACK_UP))
                .on_hover_text("Undo adjustment (Ctrl+Z)")
                .clicked()
            {
                resp.undo_clicked = true;
            }
        });

        ui.add_enabled_ui(can_redo, |ui| {
            if ui
                .button(format!("{}", ICON_ARROW_FORWARD_UP))
                .on_hover_text("Redo adjustment (Ctrl+Shift+Z)")
                .clicked()
            {
                resp.redo_clicked = true;
            }
        });

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // View Mode Segmented Selector (Single | Split | Side-by-side)
        ui.label(egui::RichText::new("View:").size(12.0).color(TEXT_SECONDARY));
        if ui
            .selectable_label(*view_mode == ViewMode::Single, format!("{} Single", ICON_PHOTO))
            .on_hover_text("Single full view (1)")
            .clicked()
        {
            *view_mode = ViewMode::Single;
        }

        if ui
            .selectable_label(
                *view_mode == ViewMode::Split,
                format!("{} Split Wipe", ICON_LAYOUT_COLUMNS),
            )
            .on_hover_text("Split screen comparison wipe (2)")
            .clicked()
        {
            *view_mode = ViewMode::Split;
        }

        if ui
            .selectable_label(
                *view_mode == ViewMode::SideBySide,
                format!("{} Side-by-Side", ICON_LAYOUT_COLUMNS),
            )
            .on_hover_text("Dual side-by-side view (3)")
            .clicked()
        {
            *view_mode = ViewMode::SideBySide;
        }

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // Zoom Controls
        if ui.button("Fit").on_hover_text("Fit to canvas (F)").clicked() {
            resp.fit_clicked = true;
        }
        if ui.button("100%").on_hover_text("Actual size 100% (0)").clicked() {
            resp.zoom_100_clicked = true;
        }

        let zoom_pct = (*zoom * 100.0).round() as u32;
        ui.label(
            egui::RichText::new(format!("{}%", zoom_pct))
                .font(egui::FontId::monospace(12.0))
                .color(TEXT_SECONDARY),
        );

        // Right side: Reset all, Latency, and Shortcuts modal
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(SPACE_12);

            // Shortcuts Help Button
            if ui
                .button(format!("{} Help", ICON_HELP))
                .on_hover_text("Keyboard shortcuts cheat sheet (?)")
                .clicked()
            {
                resp.shortcuts_clicked = true;
            }

            if ui
                .button(format!("{} Reset All", ICON_REFRESH))
                .on_hover_text("Reset all parameters to default")
                .clicked()
            {
                resp.reset_all_clicked = true;
            }

            if last_process_time > 0.0 {
                let badge = format!("⚡ {:.1} ms", last_process_time);
                ui.label(
                    egui::RichText::new(badge)
                        .font(egui::FontId::monospace(11.5))
                        .color(ACCENT),
                );
            }
        });
    });

    resp
}
