//! Collapsible adjustment section with an enable toggle and reset, in the style of
//! the Photos "Adjust" pane.

use crate::icons::*;
use crate::theme::*;
use eframe::egui::{self, Sense, Ui};

pub struct SectionResponse {
    /// The enable toggle was clicked.
    pub toggled: bool,
    pub reset: bool,
}

/// Renders a section header and, when expanded, its body.
/// `enabled` drives the toggle; `modified` shows the Reset link.
#[allow(clippy::too_many_arguments)] // header state + body closure
pub fn section(
    ui: &mut Ui,
    id: &str,
    icon: &str,
    title: &str,
    enabled: bool,
    modified: bool,
    default_open: bool,
    body: impl FnOnce(&mut Ui),
) -> SectionResponse {
    let p = pal(ui);
    let id = ui.make_persistent_id(id);
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let mut out = SectionResponse {
        toggled: false,
        reset: false,
    };

    let row = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        // Enable toggle: filled accent circle with a check when on.
        let (r, toggle) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::click());
        let c = r.center();
        if enabled {
            ui.painter().circle_filled(c, 8.0, p.accent);
            ui.painter().text(
                c,
                egui::Align2::CENTER_CENTER,
                ICON_CHECK,
                egui::FontId::proportional(11.0),
                p.on_accent,
            );
        } else {
            ui.painter()
                .circle_stroke(c, 7.5, egui::Stroke::new(1.2_f32, p.text_tertiary));
        }
        if toggle
            .on_hover_text(if enabled { "Turn off this adjustment" } else { "Turn on this adjustment" })
            .clicked()
        {
            out.toggled = true;
        }

        let title_color = if enabled { p.text } else { p.text_tertiary };
        let header = ui.add(
            egui::Label::new(
                egui::RichText::new(format!("{icon}  {title}"))
                    .size(13.5)
                    .strong()
                    .color(title_color),
            )
            .sense(Sense::click()),
        );

        let mut chevron_clicked = false;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let chevron = if state.is_open() { ICON_CHEVRON_DOWN } else { ICON_CHEVRON_RIGHT };
            chevron_clicked = ui
                .add(
                    egui::Label::new(egui::RichText::new(chevron).size(13.0).color(p.text_tertiary))
                        .sense(Sense::click()),
                )
                .clicked();
            if modified
                && ui
                    .add(
                        egui::Label::new(egui::RichText::new("Reset").size(11.5).color(p.accent))
                            .sense(Sense::click()),
                    )
                    .on_hover_text("Restore this section's defaults")
                    .clicked()
            {
                out.reset = true;
            }
        });
        header.clicked() || chevron_clicked
    });
    if row.inner {
        state.toggle(ui);
    }

    state.show_body_unindented(ui, |ui| {
        ui.add_space(SPACE_4);
        ui.add_enabled_ui(enabled, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 24,
                    right: 2,
                    top: 0,
                    bottom: 4,
                })
                .show(ui, body);
        });
    });
    ui.add_space(SPACE_4);
    crate::widgets::controls::separator(ui);
    ui.add_space(SPACE_4);
    out
}
