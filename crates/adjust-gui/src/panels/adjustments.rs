use crate::curve_widget::render_curve_editor;
use crate::icons::*;
use crate::widgets::section::render_section_header;
use crate::widgets::slider::{precision_slider_row, precision_slider_u32};
use adjust_core::{AdjustParams, HistogramData};
use eframe::egui::{self, Color32, Ui};

pub struct AdjustmentResponse {
    pub reset_all_clicked: bool,
    pub apply_clicked: bool,
    pub cancel_clicked: bool,
    pub undo_layer_clicked: bool,
}

pub fn render_adjustments_accordion(
    ui: &mut Ui,
    params: &mut AdjustParams,
    active_channel: &mut crate::curve_widget::CurveChannel,
    histogram: Option<&HistogramData>,
    is_dirty: &mut bool,
    layer_count: usize,
) -> AdjustmentResponse {
    let mut resp = AdjustmentResponse {
        reset_all_clicked: false,
        apply_clicked: false,
        cancel_clicked: false,
        undo_layer_clicked: false,
    };

    let default_params = AdjustParams::default();

    // ----------------------------------------------------
    // Section 1: Basic Exposure
    // ----------------------------------------------------
    let default_be = default_params.basic_exposure.clone();
    let is_be_modified = params.basic_exposure != default_be;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_basic_exposure",
        ICON_SUN,
        "Basic Exposure",
        is_be_modified,
        params.basic_exposure.enabled,
        true,
    );
    if h_resp.toggled_bypass {
        params.basic_exposure.enabled = !params.basic_exposure.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.basic_exposure = default_be;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.basic_exposure;
                precision_slider_row(
                    ui,
                    "Brightness",
                    &mut p.brightness,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Overall image brightness adjustment",
                );
                precision_slider_row(
                    ui,
                    "Contrast",
                    &mut p.contrast,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Overall global tonal contrast",
                );
                precision_slider_row(
                    ui,
                    "Boost Blacks",
                    &mut p.boost_blacks,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Deepens shadows without clipping dark tones",
                );
                precision_slider_row(
                    ui,
                    "Boost Whites",
                    &mut p.boost_whites,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Expands highlights towards clean white point",
                );
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 2: Adaptive Exposure
    // ----------------------------------------------------
    let default_ae = default_params.adaptive_exposure.clone();
    let is_ae_modified = params.adaptive_exposure != default_ae;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_adaptive_exposure",
        ICON_EYE,
        "Adaptive Exposure",
        is_ae_modified,
        params.adaptive_exposure.enabled,
        true,
    );
    if h_resp.toggled_bypass {
        params.adaptive_exposure.enabled = !params.adaptive_exposure.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.adaptive_exposure = default_ae;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.adaptive_exposure;
                precision_slider_row(
                    ui,
                    "Adaptive Exp",
                    &mut p.adaptive_exposure,
                    0.0..=3.0,
                    0.0,
                    is_dirty,
                    "Local tone mapping strength for shadow recovery",
                );
                precision_slider_u32(
                    ui,
                    "Regions",
                    &mut p.regions,
                    1..=30,
                    10,
                    is_dirty,
                    "Number of local subdivisions for tone evaluation",
                );
                precision_slider_row(
                    ui,
                    "Contrast",
                    &mut p.contrast,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Local dynamic contrast shaping",
                );
                precision_slider_row(
                    ui,
                    "Brightness",
                    &mut p.brightness,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Adaptive midtone brightness compensation",
                );
                precision_slider_row(
                    ui,
                    "Protect High",
                    &mut p.protect_highlights,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Prevents specular highlights from blowing out",
                );
                precision_slider_row(
                    ui,
                    "Protect Shad",
                    &mut p.protect_shadows,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Prevents deep blacks from crushing",
                );
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 3: Details & Clarity
    // ----------------------------------------------------
    let default_det = default_params.details.clone();
    let is_det_modified = params.details != default_det;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_details",
        ICON_SPARKLES,
        "Details & Clarity",
        is_det_modified,
        params.details.enabled,
        true,
    );
    if h_resp.toggled_bypass {
        params.details.enabled = !params.details.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.details = default_det;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.details;
                precision_slider_row(
                    ui,
                    "Strength",
                    &mut p.strength,
                    -1.0..=3.0,
                    0.0,
                    is_dirty,
                    "Clarity and micro-contrast intensity",
                );
                precision_slider_row(
                    ui,
                    "Detail Boost",
                    &mut p.detail_boost,
                    0.0..=3.0,
                    1.0,
                    is_dirty,
                    "Fine edge and texture enhancement multiplier",
                );
                precision_slider_row(
                    ui,
                    "Threshold",
                    &mut p.threshold,
                    0.0..=1.0,
                    0.10,
                    is_dirty,
                    "Edge sensitivity threshold to prevent noise pickup",
                );
                precision_slider_row(
                    ui,
                    "Radius",
                    &mut p.radius,
                    0.5..=30.0,
                    1.0,
                    is_dirty,
                    "Kernel size for local detail extraction",
                );
                precision_slider_row(
                    ui,
                    "Sharpen",
                    &mut p.sharpen,
                    0.0..=2.0,
                    0.0,
                    is_dirty,
                    "High-frequency unsharp mask sharpening",
                );

                ui.add_space(crate::theme::SPACE_4);
                if ui
                    .checkbox(
                        &mut p.process_independently,
                        "Process Details Independently",
                    )
                    .on_hover_text(
                        "Extracts details from original un-tone-mapped image to avoid halos",
                    )
                    .changed()
                {
                    *is_dirty = true;
                }
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 4: Color & Vibrance
    // ----------------------------------------------------
    let default_col = default_params.color.clone();
    let is_col_modified = params.color != default_col;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_color",
        ICON_PALETTE,
        "Color & Vibrance",
        is_col_modified,
        params.color.enabled,
        false,
    );
    if h_resp.toggled_bypass {
        params.color.enabled = !params.color.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.color = default_col;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.color;
                precision_slider_row(
                    ui,
                    "Adaptive Sat",
                    &mut p.adaptive_saturation,
                    -1.0..=2.0,
                    0.0,
                    is_dirty,
                    "Intelligent vibrance targeting muted colors first",
                );
                precision_slider_u32(
                    ui,
                    "Color Regions",
                    &mut p.color_regions,
                    1..=30,
                    10,
                    is_dirty,
                    "Subdivisions for regional color evaluation",
                );
                precision_slider_row(
                    ui,
                    "Saturation",
                    &mut p.saturation,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Linear color saturation boost/cut",
                );
                precision_slider_row(
                    ui,
                    "Sat Boost",
                    &mut p.saturation_boost,
                    0.0..=2.0,
                    1.0,
                    is_dirty,
                    "Saturation scaling ceiling multiplier",
                );
                precision_slider_row(
                    ui,
                    "Hue Shift",
                    &mut p.hue,
                    -180.0..=180.0,
                    0.0,
                    is_dirty,
                    "Global hue angle rotation in degrees",
                );
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 5: Noise Suppression
    // ----------------------------------------------------
    let default_nse = default_params.noise.clone();
    let is_nse_modified = params.noise != default_nse;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_noise",
        ICON_VOLUME_OFF,
        "Noise Suppression",
        is_nse_modified,
        params.noise.enabled,
        false,
    );
    if h_resp.toggled_bypass {
        params.noise.enabled = !params.noise.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.noise = default_nse;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.noise;
                precision_slider_row(
                    ui,
                    "Suppression",
                    &mut p.suppression,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Bilateral denoising smoothing strength",
                );
                precision_slider_row(
                    ui,
                    "Amount",
                    &mut p.amount,
                    0.0..=1.0,
                    0.50,
                    is_dirty,
                    "Edge-preserving threshold for high-ISO grain clean",
                );
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 6: Curve Tool
    // ----------------------------------------------------
    let default_crv = default_params.curve.clone();
    let is_crv_modified = params.curve != default_crv;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_curve",
        ICON_VECTOR_BEZIER,
        "Tone Curves",
        is_crv_modified,
        params.curve.enabled,
        false,
    );
    if h_resp.toggled_bypass {
        params.curve.enabled = !params.curve.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.curve = default_crv;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                render_curve_editor(ui, &mut params.curve, histogram, active_channel, is_dirty);
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    // ----------------------------------------------------
    // Section 7: Finishing Touches
    // ----------------------------------------------------
    let default_fin = default_params.finishing.clone();
    let is_fin_modified = params.finishing != default_fin;
    let (h_resp, mut state) = render_section_header(
        ui,
        "tab_finishing",
        ICON_WAND,
        "Finishing Touches",
        is_fin_modified,
        params.finishing.enabled,
        false,
    );
    if h_resp.toggled_bypass {
        params.finishing.enabled = !params.finishing.enabled;
        *is_dirty = true;
    }
    if h_resp.reset_clicked {
        params.finishing = default_fin;
        *is_dirty = true;
    }
    state.show_body_unindented(ui, |ui| {
        ui.add_space(crate::theme::SPACE_4);
        let panel_bg = crate::theme::theme_surface_panel(ui.visuals());
        let sec_text = crate::theme::theme_text_secondary(ui.visuals());
        egui::Frame::new()
            .fill(panel_bg)
            .inner_margin(egui::Margin::symmetric(4, 4))
            .show(ui, |ui| {
                let p = &mut params.finishing;
                precision_slider_row(
                    ui,
                    "Transparency",
                    &mut p.transparency,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Blend factor with original unprocessed image",
                );
                precision_slider_row(
                    ui,
                    "Warmth",
                    &mut p.warmth,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Kelvin temperature balance (Cool blue to Warm gold)",
                );

                ui.add_space(crate::theme::SPACE_8);
                ui.label(
                    egui::RichText::new("VIGNETTE")
                        .size(11.0)
                        .strong()
                        .color(crate::theme::TEXT_SECONDARY),
                );
                precision_slider_row(
                    ui,
                    "Amount",
                    &mut p.vignette.amount,
                    -1.0..=1.0,
                    0.0,
                    is_dirty,
                    "Vignette intensity (darken corners or high key)",
                );
                precision_slider_row(
                    ui,
                    "Size",
                    &mut p.vignette.size,
                    0.1..=1.0,
                    0.50,
                    is_dirty,
                    "Radial falloff span radius",
                );
                precision_slider_row(
                    ui,
                    "Transition",
                    &mut p.vignette.transition,
                    0.0..=1.0,
                    0.50,
                    is_dirty,
                    "Feather softness of the vignette edge",
                );
                precision_slider_row(
                    ui,
                    "Roundness",
                    &mut p.vignette.roundness,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Circular vs rectangular aspect shape",
                );

                ui.add_space(crate::theme::SPACE_8);
                ui.label(
                    egui::RichText::new("FILM GRAIN")
                        .size(11.0)
                        .strong()
                        .color(crate::theme::TEXT_SECONDARY),
                );
                precision_slider_row(
                    ui,
                    "Grain Amount",
                    &mut p.grain.amount,
                    0.0..=1.0,
                    0.0,
                    is_dirty,
                    "Organic analog film grain overlay",
                );
                precision_slider_row(
                    ui,
                    "Grain Size",
                    &mut p.grain.size,
                    0.5..=3.0,
                    1.0,
                    is_dirty,
                    "Grain cluster particle scale",
                );

                ui.add_space(crate::theme::SPACE_8);
                ui.label(
                    egui::RichText::new("BORDER FRAME")
                        .size(11.0)
                        .strong()
                        .color(crate::theme::TEXT_SECONDARY),
                );
                if ui
                    .checkbox(&mut p.border.enabled, "Enable Clean Border")
                    .changed()
                {
                    *is_dirty = true;
                }
                precision_slider_row(
                    ui,
                    "Thickness",
                    &mut p.border.thickness,
                    0.0..=0.15,
                    0.0,
                    is_dirty,
                    "Border inset thickness relative to frame",
                );
            });
        ui.add_space(crate::theme::SPACE_4);
    });

    ui.add_space(crate::theme::SPACE_16);

    // ----------------------------------------------------
    // Sticky Footer Section: Layer Stack & Apply / Cancel Actions
    // ----------------------------------------------------
    let footer_bg = crate::theme::theme_surface_card(ui.visuals());
    let border_col = crate::theme::theme_border(ui.visuals());
    egui::Frame::new()
        .fill(footer_bg)
        .stroke(egui::Stroke::new(1.0_f32, border_col))
        .corner_radius(crate::theme::RADIUS_CONTROL)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            if layer_count > 0 {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} Stack: {} Layer{}",
                            ICON_HISTORY,
                            layer_count,
                            if layer_count > 1 { "s" } else { "" }
                        ))
                        .size(12.0)
                        .color(crate::theme::ACCENT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(format!("{} Pop Layer", ICON_ARROW_BACK_UP))
                            .on_hover_text("Undo last baked adjustment layer")
                            .clicked()
                        {
                            resp.undo_layer_clicked = true;
                        }
                    });
                });
                ui.add_space(crate::theme::SPACE_8);
                ui.separator();
                ui.add_space(crate::theme::SPACE_8);
            }

            ui.horizontal(|ui| {
                let btn_reset = egui::Button::new("Reset All")
                    .min_size(egui::vec2(80.0, crate::theme::BTN_HEIGHT_MD));
                if ui
                    .add(btn_reset)
                    .on_hover_text("Revert all sliders and curves to defaults")
                    .clicked()
                {
                    resp.reset_all_clicked = true;
                }

                let btn_cancel = egui::Button::new("Cancel")
                    .min_size(egui::vec2(70.0, crate::theme::BTN_HEIGHT_MD));
                if ui
                    .add(btn_cancel)
                    .on_hover_text("Clear adjustment stack and reload base")
                    .clicked()
                {
                    resp.cancel_clicked = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let apply_btn = egui::Button::new(
                        egui::RichText::new(format!("{} Apply (Bake)", ICON_APPLY))
                            .strong()
                            .color(Color32::BLACK),
                    )
                    .fill(crate::theme::ACCENT)
                    .min_size(egui::vec2(110.0, crate::theme::BTN_HEIGHT_MD));

                    if ui
                        .add(apply_btn)
                        .on_hover_text(
                            "Bake current adjustments as a new base layer to stack more effects!",
                        )
                        .clicked()
                    {
                        resp.apply_clicked = true;
                    }
                });
            });
        });

    ui.add_space(crate::theme::SPACE_12);

    resp
}
