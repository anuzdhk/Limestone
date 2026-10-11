use crate::icons::*;
use crate::theme::*;
use eframe::egui::{self, Ui};

#[derive(PartialEq, Clone, Copy)]
pub enum ViewMode {
    Single,
    Split,
    SideBySide,
}

pub struct ToolbarResponse {
    pub open_clicked: bool,
    pub export_clicked: bool,
    pub undo_clicked: bool,
    pub redo_clicked: bool,
    pub reset_all_clicked: bool,
    pub fit_clicked: bool,
    pub zoom_100_clicked: bool,
    pub paste_clipboard_clicked: bool,
    pub settings_clicked: bool,
    pub shortcuts_clicked: bool,
}

#[allow(clippy::too_many_arguments)] // one argument per piece of widget state
pub fn render_top_toolbar(
    ui: &mut Ui,
    view_mode: &mut ViewMode,
    zoom: &mut f32,
    opacity: &mut f32,
    can_undo: bool,
    can_redo: bool,
    last_process_time: f32,
    active_export_count: usize,
) -> ToolbarResponse {
    let mut resp = ToolbarResponse {
        open_clicked: false,
        export_clicked: false,
        undo_clicked: false,
        redo_clicked: false,
        reset_all_clicked: false,
        fit_clicked: false,
        zoom_100_clicked: false,
        paste_clipboard_clicked: false,
        settings_clicked: false,
        shortcuts_clicked: false,
    };

    let text_col = theme_text_primary(ui.visuals());
    let sec_col = theme_text_secondary(ui.visuals());

    ui.horizontal_centered(|ui| {
        ui.add_space(SPACE_8);

        // App Brand / Logo
        ui.label(
            egui::RichText::new("LIMESTONE")
                .strong()
                .size(15.0)
                .color(text_col),
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

        // File I/O - Standardized 28px height buttons with tooltips
        let btn_open = egui::Button::new(
            egui::RichText::new(format!("{} Open", ICON_FOLDER_OPEN)).size(12.5),
        )
        .min_size(egui::vec2(74.0, BTN_HEIGHT_MD));

        if ui.add(btn_open).on_hover_text("Open image from disk (Ctrl+O)").clicked() {
            resp.open_clicked = true;
        }

        let export_label = if active_export_count > 0 {
            format!("{} Export ({})", ICON_DEVICE_FLOPPY, active_export_count)
        } else {
            format!("{} Export", ICON_DEVICE_FLOPPY)
        };
        let btn_export = egui::Button::new(
            egui::RichText::new(export_label).size(12.5),
        )
        .min_size(egui::vec2(80.0, BTN_HEIGHT_MD));

        if ui.add(btn_export).on_hover_text("Queue full-resolution background export (Ctrl+E)").clicked() {
            resp.export_clicked = true;
        }

        let btn_paste = egui::Button::new(
            egui::RichText::new(ICON_CLIPBOARD.to_string()).size(13.0),
        )
        .min_size(egui::vec2(32.0, BTN_HEIGHT_MD));

        if ui.add(btn_paste).on_hover_text("Paste image from clipboard (Ctrl+V)").clicked() {
            resp.paste_clipboard_clicked = true;
        }

        ui.add_space(SPACE_8);
        ui.separator();
        ui.add_space(SPACE_8);

        // Undo / Redo Buttons
        ui.add_enabled_ui(can_undo, |ui| {
            let btn_undo = egui::Button::new(
                egui::RichText::new(ICON_ARROW_BACK_UP.to_string()).size(13.0),
            )
            .min_size(egui::vec2(32.0, BTN_HEIGHT_MD));
            if ui.add(btn_undo).on_hover_text("Undo adjustment (Ctrl+Z)").clicked() {
                resp.undo_clicked = true;
            }
        });

        ui.add_enabled_ui(can_redo, |ui| {
            let btn_redo = egui::Button::new(
                egui::RichText::new(ICON_ARROW_FORWARD_UP.to_string()).size(13.0),
            )
            .min_size(egui::vec2(32.0, BTN_HEIGHT_MD));
            if ui.add(btn_redo).on_hover_text("Redo adjustment (Ctrl+Shift+Z)").clicked() {
                resp.redo_clicked = true;
            }
        });

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // View Mode Segmented Selector (Single | Split | Side-by-side)
        ui.label(egui::RichText::new("View:").size(12.0).color(sec_col));
        let btn_v1 = egui::SelectableLabel::new(*view_mode == ViewMode::Single, format!("{} Single", ICON_PHOTO));
        if ui.add_sized([78.0, BTN_HEIGHT_MD], btn_v1).on_hover_text("Single full view (1)").clicked() {
            *view_mode = ViewMode::Single;
        }

        let btn_v2 = egui::SelectableLabel::new(*view_mode == ViewMode::Split, format!("{} Split", ICON_LAYOUT_COLUMNS));
        if ui.add_sized([82.0, BTN_HEIGHT_MD], btn_v2).on_hover_text("Split comparison wipe (2)").clicked() {
            *view_mode = ViewMode::Split;
        }

        let btn_v3 = egui::SelectableLabel::new(*view_mode == ViewMode::SideBySide, format!("{} Dual", ICON_LAYOUT_COLUMNS));
        if ui.add_sized([72.0, BTN_HEIGHT_MD], btn_v3).on_hover_text("Side-by-side dual view (3)").clicked() {
            *view_mode = ViewMode::SideBySide;
        }

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // Master Opacity / Strength Slider at the top
        ui.label(
            egui::RichText::new(format!("{} Opacity:", ICON_SLIDERS))
                .size(12.0)
                .color(sec_col),
        );
        let op_slider = egui::Slider::new(opacity, 0.0..=100.0)
            .show_value(false)
            .step_by(1.0);
        let op_resp = ui.add_sized([100.0, BTN_HEIGHT_MD], op_slider);
        if op_resp.double_clicked() {
            *opacity = 100.0;
        }
        op_resp.on_hover_text("Master Effect Opacity (0% - 100%). Double-click to reset to 100%.");

        ui.label(
            egui::RichText::new(format!("{:.0}%", *opacity))
                .font(egui::FontId::monospace(11.5))
                .color(ACCENT),
        );

        ui.add_space(SPACE_12);
        ui.separator();
        ui.add_space(SPACE_12);

        // Zoom Controls
        if ui.add_sized([38.0, BTN_HEIGHT_MD], egui::Button::new("Fit")).on_hover_text("Fit to canvas (F)").clicked() {
            resp.fit_clicked = true;
        }
        if ui.add_sized([44.0, BTN_HEIGHT_MD], egui::Button::new("100%")).on_hover_text("Actual size 100% (0)").clicked() {
            resp.zoom_100_clicked = true;
        }

        let zoom_pct = (*zoom * 100.0).round() as u32;
        ui.label(
            egui::RichText::new(format!("{}%", zoom_pct))
                .font(egui::FontId::monospace(12.0))
                .color(sec_col),
        );

        // Right side: Reset all, Settings, Latency, and Shortcuts modal
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(SPACE_12);

            // Settings Preferences Dialog
            let btn_cfg = egui::Button::new(
                egui::RichText::new(format!("{} Settings", ICON_SETTINGS)).size(12.5),
            )
            .min_size(egui::vec2(86.0, BTN_HEIGHT_MD));
            if ui.add(btn_cfg).on_hover_text("Themes, language, and acceleration").clicked() {
                resp.settings_clicked = true;
            }

            // Shortcuts Help Button
            let btn_help = egui::Button::new(
                egui::RichText::new(ICON_HELP.to_string()).size(13.0),
            )
            .min_size(egui::vec2(32.0, BTN_HEIGHT_MD));
            if ui.add(btn_help).on_hover_text("Keyboard shortcuts cheat sheet (?)").clicked() {
                resp.shortcuts_clicked = true;
            }

            let btn_reset = egui::Button::new(
                egui::RichText::new(format!("{} Reset", ICON_REFRESH)).size(12.5),
            )
            .min_size(egui::vec2(72.0, BTN_HEIGHT_MD));
            if ui.add(btn_reset).on_hover_text("Reset all parameters to default").clicked() {
                resp.reset_all_clicked = true;
            }

            // Skip the badge when the window is too narrow (it's also in the status bar),
            // otherwise it would draw over the zoom controls.
            if last_process_time > 0.0 && ui.available_width() > 110.0 {
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
