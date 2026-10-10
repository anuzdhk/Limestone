use crate::icons::*;
use crate::theme::*;
use eframe::egui::{self, Stroke, Ui};

pub struct AccordionHeaderResponse {
    pub is_open: bool,
    pub toggled_bypass: bool,
    pub reset_clicked: bool,
}

/// Renders a section header and returns whether it's open, if bypass was toggled, and if reset was clicked.
pub fn render_section_header(
    ui: &mut Ui,
    id_str: &str,
    icon: &str,
    title: &str,
    is_modified: bool,
    enabled: bool,
    default_open: bool,
) -> (AccordionHeaderResponse, egui::collapsing_header::CollapsingState) {
    ui.add_space(SPACE_4);

    let id = ui.make_persistent_id(id_str);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let is_open = state.is_open();

    let mut resp = AccordionHeaderResponse {
        is_open,
        toggled_bypass: false,
        reset_clicked: false,
    };

    let header_response = ui.horizontal(|ui| {
        // Expand/collapse chevron
        let chevron = if is_open {
            ICON_CHEVRON_DOWN
        } else {
            ICON_CHEVRON_RIGHT
        };

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(chevron)
                        .size(11.0)
                        .color(TEXT_SECONDARY),
                )
                .frame(false),
            )
            .clicked()
        {
            state.toggle(ui);
            resp.is_open = state.is_open();
        }

        // Stage Icon & Title
        let title_color = if enabled {
            TEXT_PRIMARY
        } else {
            TEXT_DISABLED
        };

        let title_resp = ui.add(
            egui::Label::new(
                egui::RichText::new(format!("{} {}", icon, title))
                    .strong()
                    .size(13.5)
                    .color(title_color),
            )
            .sense(egui::Sense::click()),
        );

        if title_resp.clicked() {
            state.toggle(ui);
            resp.is_open = state.is_open();
        }

        // Small Modified Indicator Dot (if values differ from neutral)
        if is_modified {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.0, ACCENT);
        }

        // Right side: Reset & Bypass buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Bypass / Eye toggle
            let eye_icon = if enabled { ICON_EYE } else { ICON_EYE_OFF };
            let eye_color = if enabled { ACCENT } else { TEXT_DISABLED };

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(eye_icon).size(13.0).color(eye_color),
                    )
                    .frame(false),
                )
                .on_hover_text("Bypass / Enable this processing stage")
                .clicked()
            {
                resp.toggled_bypass = true;
            }

            // Reset button (only active when modified)
            if is_modified {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(ICON_REFRESH)
                                .size(11.0)
                                .color(TEXT_SECONDARY),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Reset this stage to defaults")
                    .clicked()
                {
                    resp.reset_clicked = true;
                }
            }
        });
    });

    ui.add_space(2.0);
    ui.painter().hline(
        header_response.response.rect.x_range(),
        header_response.response.rect.bottom() + 4.0,
        Stroke::new(1.0_f32, BORDER_SUBTLE),
    );

    (resp, state)
}
