//! The "Adjust" pane: one collapsible section per processing stage.

use crate::curve_widget::{curve_editor, CurveChannel};
use crate::icons::*;
use crate::theme::*;
use crate::widgets::section::section;
use crate::widgets::slider::{slider, slider_u32, SliderSpec as S};
use adjust_core::{AdjustParams, HistogramData};
use eframe::egui::{self, Ui};

/// Draws every adjustment section. Returns true if any parameter changed.
pub fn adjustments(
    ui: &mut Ui,
    params: &mut AdjustParams,
    curve_channel: &mut CurveChannel,
    histogram: Option<&HistogramData>,
) -> bool {
    let d = AdjustParams::default();
    let mut changed = false;

    // Each section: (state struct, default) -> header toggle/reset + sliders.
    macro_rules! stage {
        ($field:ident, $id:expr, $icon:expr, $title:expr, $open:expr, |$ui:ident, $p:ident| $body:block) => {{
            let modified = params.$field != d.$field;
            let enabled = params.$field.enabled;
            let r = section(ui, $id, $icon, $title, enabled, modified, $open, |$ui| {
                let $p = &mut params.$field;
                if $body {
                    changed = true;
                }
            });
            if r.toggled {
                params.$field.enabled = !params.$field.enabled;
                changed = true;
            }
            if r.reset {
                params.$field = d.$field.clone();
                changed = true;
            }
        }};
    }

    stage!(basic_exposure, "sec_light", ICON_SUN, "Light", true, |ui, p| {
        slider(ui, &mut p.brightness, S::new("Brightness", -1.0..=1.0, 0.0).tooltip("Overall brightness"))
            | slider(ui, &mut p.contrast, S::new("Contrast", -1.0..=1.0, 0.0).tooltip("Global contrast around mid-grey"))
            | slider(ui, &mut p.boost_blacks, S::new("Blacks", 0.0..=1.0, 0.0).tooltip("Deepen the shadows"))
            | slider(ui, &mut p.boost_whites, S::new("Whites", 0.0..=1.0, 0.0).tooltip("Brighten the highlights"))
    });

    stage!(adaptive_exposure, "sec_adaptive", ICON_CONTRAST, "Adaptive Exposure", true, |ui, p| {
        slider(ui, &mut p.adaptive_exposure, S::new("Amount", 0.0..=3.0, 0.0).tooltip("Local tone mapping strength (HDR look)"))
            | slider_u32(ui, &mut p.regions, "Regions", 1..=30, 10, "Fewer regions = broader, smoother tone zones")
            | slider(ui, &mut p.contrast, S::new("Regional Contrast", -1.0..=1.0, 0.0))
            | slider(ui, &mut p.brightness, S::new("Regional Brightness", -1.0..=1.0, 0.0))
            | slider(ui, &mut p.protect_highlights, S::new("Protect Highlights", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.protect_shadows, S::new("Protect Shadows", 0.0..=1.0, 0.0))
    });

    stage!(details, "sec_details", ICON_SPARKLES, "Detail", true, |ui, p| {
        let mut c = slider(ui, &mut p.strength, S::new("Strength", -1.0..=3.0, 0.0).tooltip("Negative values smooth, positive values sharpen micro-contrast"))
            | slider(ui, &mut p.detail_boost, S::new("Small Detail Boost", 0.0..=3.0, 1.0))
            | slider(ui, &mut p.threshold, S::new("Threshold", 0.0..=1.0, 0.10).tooltip("Ignore texture below this level"))
            | slider(ui, &mut p.radius, S::new("Radius", 0.5..=30.0, 1.0).decimals(1).tooltip("Size of detail being enhanced"))
            | slider(ui, &mut p.sharpen, S::new("Sharpen", 0.0..=2.0, 0.0));
        c |= ui.checkbox(&mut p.process_independently, "Process before tone mapping").changed();
        c
    });

    stage!(color, "sec_color", ICON_PALETTE, "Color", true, |ui, p| {
        slider(ui, &mut p.adaptive_saturation, S::new("Vibrance", -1.0..=2.0, 0.0).tooltip("Boosts muted colours, protects saturated ones"))
            | slider(ui, &mut p.saturation_boost, S::new("Vibrance Boost", 0.0..=2.0, 1.0))
            | slider(ui, &mut p.saturation, S::new("Saturation", -1.0..=1.0, 0.0))
            | slider(ui, &mut p.hue, S::new("Hue Shift", -180.0..=180.0, 0.0).decimals(0).suffix("°"))
    });

    stage!(noise, "sec_noise", ICON_DROPLET, "Noise Reduction", false, |ui, p| {
        slider(ui, &mut p.suppression, S::new("Strength", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.amount, S::new("Smoothness", 0.0..=1.0, 0.5).tooltip("Filter size"))
    });

    stage!(stylize, "sec_stylize", ICON_BRUSH, "Creative", false, |ui, p| {
        slider(ui, &mut p.sketch, S::new("Sketch", 0.0..=1.0, 0.0).tooltip("Pencil / colour drawing effect"))
            | slider(ui, &mut p.sketch_radius, S::new("Line Width", 1.0..=20.0, 4.0).decimals(1))
            | slider(ui, &mut p.sketch_color, S::new("Sketch Colour", 0.0..=1.0, 0.0).tooltip("0 = graphite, 1 = full colour"))
            | slider(ui, &mut p.glow, S::new("Glow", 0.0..=1.0, 0.0).tooltip("Soft diffusion bloom"))
            | slider(ui, &mut p.glow_radius, S::new("Glow Radius", 2.0..=60.0, 12.0).decimals(0))
            | slider(ui, &mut p.posterize, S::new("Posterize", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.solarize, S::new("Solarize", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.invert, S::new("Invert", 0.0..=1.0, 0.0))
    });

    stage!(tone, "sec_tone", ICON_CIRCLE_HALF, "Black & White and Toning", false, |ui, p| {
        let mut c = slider(ui, &mut p.monochrome, S::new("Black & White", 0.0..=1.0, 0.0));
        let [mut r, mut g, mut b] = p.mono_mix;
        c |= slider(ui, &mut r, S::new("Red Filter", -1.0..=2.0, 0.30))
            | slider(ui, &mut g, S::new("Green Filter", -1.0..=2.0, 0.59))
            | slider(ui, &mut b, S::new("Blue Filter", -1.0..=2.0, 0.11));
        p.mono_mix = [r, g, b];
        ui.add_space(SPACE_4);
        c |= slider(ui, &mut p.shadow_hue, S::new("Shadow Tint", 0.0..=360.0, 30.0).decimals(0).suffix("°").hue())
            | slider(ui, &mut p.shadow_strength, S::new("Shadow Amount", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.highlight_hue, S::new("Highlight Tint", 0.0..=360.0, 45.0).decimals(0).suffix("°").hue())
            | slider(ui, &mut p.highlight_strength, S::new("Highlight Amount", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.balance, S::new("Balance", -1.0..=1.0, 0.0));
        c
    });

    stage!(curve, "sec_curve", ICON_SWATCH, "Curves", false, |ui, p| {
        curve_editor(ui, p, histogram, curve_channel)
    });

    stage!(finishing, "sec_finish", ICON_APERTURE, "Finishing", false, |ui, p| {
        let mut c = slider(ui, &mut p.warmth, S::new("Warmth", -1.0..=1.0, 0.0).tooltip("Cool ← → Warm"));
        ui.add_space(SPACE_4);
        c |= slider(ui, &mut p.vignette.amount, S::new("Vignette", -1.0..=1.0, 0.0).tooltip("Negative darkens, positive lightens the edges"))
            | slider(ui, &mut p.vignette.size, S::new("Vignette Size", 0.1..=1.0, 0.5))
            | slider(ui, &mut p.vignette.transition, S::new("Vignette Softness", 0.0..=1.0, 0.5))
            | slider(ui, &mut p.vignette.roundness, S::new("Vignette Roundness", 0.0..=1.0, 0.0));
        ui.add_space(SPACE_4);
        c |= slider(ui, &mut p.grain.amount, S::new("Grain", 0.0..=1.0, 0.0))
            | slider(ui, &mut p.grain.size, S::new("Grain Size", 0.5..=3.0, 1.0).decimals(1));
        ui.add_space(SPACE_4);
        c |= ui.checkbox(&mut p.border.enabled, "Border").changed();
        if p.border.enabled {
            c |= slider(ui, &mut p.border.thickness, S::new("Border Width", 0.0..=0.2, 0.0).decimals(3));
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Border Colour").size(12.5));
                c |= ui.color_edit_button_rgb(&mut p.border.color).changed();
            });
        }
        c
    });

    changed
}
