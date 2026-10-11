//! Modal sheets: Settings, Keyboard Shortcuts and Save Preset.

use crate::config::AppConfig;
use crate::icons::*;
use crate::theme::*;
use crate::widgets::controls::{caption, pill_button, segmented};
use eframe::egui::{self, Sense};

/// A centred, dimmed-background modal window. Returns false when it should close.
fn sheet(ctx: &egui::Context, id: &str, title: &str, width: f32, body: impl FnOnce(&mut egui::Ui) -> bool) -> bool {
    let p = pal_ctx(ctx);
    // Dim and block the app behind the sheet. Like macOS sheets, it only closes via
    // its buttons or Esc (an outside click would also catch the click that opened it).
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new((id, "backdrop")))
        .fixed_pos(screen.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.allocate_rect(screen, Sense::click_and_drag());
            ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(if p.dark { 120 } else { 70 }));
        });

    let mut open = true;
    egui::Window::new(title)
        .id(egui::Id::new(id))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(width, 0.0))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(p.card)
                .inner_margin(egui::Margin::same(20)),
        )
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(title).size(17.0).strong().color(p.text));
            ui.add_space(SPACE_12);
            open = body(ui);
        });
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    open && !escape
}

/// Returns (keep_open, theme_changed).
pub fn settings(ctx: &egui::Context, config: &mut AppConfig) -> (bool, bool) {
    let mut theme_changed = false;
    let open = sheet(ctx, "settings_sheet", "Settings", 420.0, |ui| {
        let p = pal(ui);
        caption(ui, "APPEARANCE");
        theme_changed |= segmented(
            ui,
            &mut config.theme_mode,
            &[
                (ThemeMode::System, ThemeMode::System.name(), "Follow your system setting"),
                (ThemeMode::Light, ThemeMode::Light.name(), ""),
                (ThemeMode::Dark, ThemeMode::Dark.name(), ""),
            ],
            120.0,
        );
        ui.add_space(SPACE_12);

        caption(ui, "ACCENT COLOUR");
        ui.horizontal(|ui| {
            for accent in Accent::ALL {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                let c = rect.center();
                ui.painter().circle_filled(c, 9.0, accent.color(p.dark));
                if config.accent == accent {
                    ui.painter().circle_stroke(c, 12.0, egui::Stroke::new(2.0_f32, accent.color(p.dark)));
                    ui.painter().circle_filled(c, 3.0, egui::Color32::WHITE);
                }
                if resp.on_hover_text(accent.name()).clicked() {
                    config.accent = accent;
                    theme_changed = true;
                }
            }
        });
        ui.add_space(SPACE_12);

        caption(ui, "INTERFACE SIZE");
        let mut pct = config.ui_scale * 100.0;
        if crate::widgets::slider::slider(
            ui,
            &mut pct,
            crate::widgets::slider::SliderSpec::new("Scale", 75.0..=150.0, 100.0).decimals(0).suffix("%"),
        ) {
            config.ui_scale = (pct / 5.0).round() * 5.0 / 100.0;
            theme_changed = true;
        }
        ui.add_space(SPACE_12);

        caption(ui, "PREVIEW");
        if ui
            .checkbox(&mut config.high_quality_tiled_preview, "Render full resolution when zoomed in")
            .on_hover_text("Shows true pixels instead of the scaled preview when you zoom past it")
            .changed()
        {
            config.save();
        }
        ui.add_space(SPACE_16);

        let mut keep = true;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if pill_button(ui, "Done", true, true).clicked() {
                keep = false;
            }
        });
        keep
    });
    if theme_changed {
        config.save();
    }
    (open, theme_changed)
}

pub fn shortcuts(ctx: &egui::Context) -> bool {
    sheet(ctx, "shortcuts_sheet", "Keyboard Shortcuts", 420.0, |ui| {
        let p = pal(ui);
        let rows = [
            ("Space (hold)", "Show the original"),
            ("Ctrl + O", "Open image"),
            ("Ctrl + V", "Paste image or copied file"),
            ("Ctrl + E", "Export"),
            ("Ctrl + ,", "Settings"),
            ("Ctrl + S", "Save as preset"),
            ("Ctrl + Z", "Undo"),
            ("Ctrl + Shift + Z", "Redo"),
            ("1 / 2 / 3", "Single · Split · Side by Side"),
            ("F", "Fit to window"),
            ("0", "Actual size (100%)"),
            ("+ / −", "Zoom in / out"),
            ("Tab", "Show or hide sidebars"),
            ("Shift + drag slider", "Fine adjustment"),
            ("Double-click slider", "Reset to default"),
        ];
        egui::Grid::new("shortcut_grid").num_columns(2).spacing([24.0, 8.0]).show(ui, |ui| {
            for (key, desc) in rows {
                ui.label(egui::RichText::new(key).monospace().size(12.0).color(p.accent));
                ui.label(egui::RichText::new(desc).size(12.5).color(p.text));
                ui.end_row();
            }
        });
        ui.add_space(SPACE_16);
        let mut keep = true;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if pill_button(ui, "Done", true, true).clicked() {
                keep = false;
            }
        });
        keep
    })
}

pub enum SaveOutcome {
    Open,
    Cancel,
    Save,
}

pub fn save_preset(ctx: &egui::Context, name: &mut String, exists: bool) -> SaveOutcome {
    let mut outcome = SaveOutcome::Open;
    let keep = sheet(ctx, "save_preset_sheet", "Save Preset", 380.0, |ui| {
        let p = pal(ui);
        ui.label(egui::RichText::new("Saves the current adjustments to My Collection.").size(12.5).color(p.text_secondary));
        ui.add_space(SPACE_8);
        let edit = ui.add(egui::TextEdit::singleline(name).hint_text("Preset name").desired_width(f32::INFINITY));
        edit.request_focus();
        if exists {
            ui.label(egui::RichText::new(format!("{ICON_INFO}  A preset with this name will be replaced.")).size(11.5).color(p.warning));
        }
        ui.add_space(SPACE_16);
        let valid = !name.trim().is_empty();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            if pill_button(ui, "Save", true, valid).clicked() || (enter && valid) {
                outcome = SaveOutcome::Save;
            }
            if pill_button(ui, "Cancel", false, true).clicked() {
                outcome = SaveOutcome::Cancel;
            }
        });
        true
    });
    if !keep {
        return SaveOutcome::Cancel;
    }
    outcome
}
