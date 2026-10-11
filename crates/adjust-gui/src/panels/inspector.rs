//! Right-hand inspector: image info card (histogram / navigator / loupe), effect amount,
//! the adjustment sections, and the layer bar (Apply / Undo Layer).

use crate::curve_widget::CurveChannel;
use crate::icons::*;
use crate::panels::adjustments::adjustments;
use crate::theme::*;
use crate::widgets::controls::{pill_button, segmented, separator};
use crate::widgets::slider::{slider, SliderSpec};
use adjust_core::{AdjustParams, HistogramData};
use eframe::egui::{self, Color32, Sense, Stroke, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoTab {
    Histogram,
    Navigator,
    Loupe,
}

pub enum InspectorAction {
    Changed,
    Apply,
    UndoLayer,
    ResetAll,
}

pub struct InspectorState<'a> {
    pub params: &'a mut AdjustParams,
    pub info_tab: &'a mut InfoTab,
    pub curve_channel: &'a mut CurveChannel,
    pub histogram: Option<&'a HistogramData>,
    pub nav_texture: Option<&'a egui::TextureHandle>,
    pub proc_texture: Option<&'a egui::TextureHandle>,
    pub pan: &'a mut egui::Vec2,
    /// Image size on the canvas in points (for the navigator viewport box).
    pub display_size: Option<egui::Vec2>,
    pub canvas_size: egui::Vec2,
    pub layer_count: usize,
    pub baking: bool,
    pub has_image: bool,
    pub preset_name: &'a str,
}

pub fn inspector(ui: &mut Ui, s: InspectorState) -> Vec<InspectorAction> {
    let p = pal(ui);
    let mut actions = Vec::new();

    // ---- Pinned layer bar ---------------------------------------------------
    egui::TopBottomPanel::bottom("inspector_footer")
        .frame(egui::Frame::new().fill(p.sidebar).inner_margin(egui::Margin::symmetric(12, 8)))
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            separator(ui);
            ui.add_space(SPACE_6);
            ui.horizontal(|ui| {
                let label = match s.layer_count {
                    0 => "No layers applied".to_string(),
                    1 => "1 layer applied".to_string(),
                    n => format!("{n} layers applied"),
                };
                ui.label(egui::RichText::new(format!("{ICON_LAYERS}  {label}")).size(11.5).color(p.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Label::new(egui::RichText::new("Reset All").size(11.5).color(p.accent)).sense(Sense::click()))
                        .on_hover_text("Reset every slider to its default")
                        .clicked()
                    {
                        actions.push(InspectorAction::ResetAll);
                    }
                });
            });
            ui.add_space(SPACE_6);
            ui.horizontal(|ui| {
                let can_undo = s.layer_count > 0 && !s.baking;
                if pill_button(ui, &format!("{ICON_LAYER_POP}  Undo Layer"), false, can_undo)
                    .on_hover_text("Remove the last applied layer and bring its settings back")
                    .clicked()
                {
                    actions.push(InspectorAction::UndoLayer);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if s.baking { "Applying…" } else { "Apply" };
                    if pill_button(ui, &format!("{ICON_CHECK}  {label}"), true, s.has_image && !s.baking)
                        .on_hover_text("Bake these adjustments into the image so you can stack another look on top")
                        .clicked()
                    {
                        actions.push(InspectorAction::Apply);
                    }
                });
            });
        });

    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(SPACE_8);
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 4.0) / 3.0;
                segmented(
                    ui,
                    s.info_tab,
                    &[
                        (InfoTab::Histogram, "Histogram", ""),
                        (InfoTab::Navigator, "Navigator", "Drag to pan"),
                        (InfoTab::Loupe, "Loupe", "4× centre detail"),
                    ],
                    w,
                );
            });
            ui.add_space(SPACE_6);
            let card_h = 120.0;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), card_h), Sense::click_and_drag());
            ui.painter().rect_filled(rect, RADIUS_CARD, p.card);
            match *s.info_tab {
                InfoTab::Histogram => match s.histogram {
                    Some(h) => draw_histogram(ui, rect.shrink(8.0), h),
                    None => placeholder(ui, rect, "No image"),
                },
                InfoTab::Navigator => match s.nav_texture {
                    Some(tex) => navigator(ui, rect, &resp, tex, s.pan, s.display_size, s.canvas_size),
                    None => placeholder(ui, rect, "No image"),
                },
                InfoTab::Loupe => match s.proc_texture {
                    Some(tex) => loupe(ui, rect, tex),
                    None => placeholder(ui, rect, "No image"),
                },
            }

            ui.add_space(SPACE_12);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Adjust").size(17.0).strong().color(p.text));
                ui.label(egui::RichText::new(s.preset_name).size(12.0).color(p.text_secondary));
            });
            ui.add_space(SPACE_4);

            // Effect amount (inverse of the master transparency)
            let mut amount = (1.0 - s.params.finishing.transparency) * 100.0;
            if slider(
                ui,
                &mut amount,
                SliderSpec::new("Effect Amount", 0.0..=100.0, 100.0)
                    .decimals(0)
                    .suffix("%")
                    .tooltip("Blend between the original (0%) and the full effect (100%)"),
            ) {
                s.params.finishing.transparency = 1.0 - amount / 100.0;
                actions.push(InspectorAction::Changed);
            }
            ui.add_space(SPACE_4);
            separator(ui);
            ui.add_space(SPACE_6);

            if adjustments(ui, s.params, s.curve_channel, s.histogram) {
                actions.push(InspectorAction::Changed);
            }
            ui.add_space(SPACE_16);
        });

    actions
}

fn placeholder(ui: &Ui, rect: egui::Rect, text: &str) {
    let p = pal(ui);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(12.0), p.text_tertiary);
}

/// Overlapping RGB histogram (additive, like Photos) with a luminance outline.
fn draw_histogram(ui: &Ui, rect: egui::Rect, h: &HistogramData) {
    let p = pal(ui);
    let painter = ui.painter_at(rect);
    let max = h
        .r
        .iter()
        .chain(&h.g)
        .chain(&h.b)
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let scale = |v: u32| (v as f32 / max).sqrt() * rect.height();
    let alpha = if p.dark { 110 } else { 95 };
    for (bins, color) in [
        (&h.r, Color32::from_rgba_unmultiplied(255, 70, 70, alpha)),
        (&h.g, Color32::from_rgba_unmultiplied(70, 220, 90, alpha)),
        (&h.b, Color32::from_rgba_unmultiplied(70, 140, 255, alpha)),
    ] {
        let mut pts = vec![rect.left_bottom()];
        pts.extend((0..256).map(|i| egui::pos2(rect.left() + i as f32 / 255.0 * rect.width(), rect.bottom() - scale(bins[i]))));
        pts.push(rect.right_bottom());
        // Fill as thin columns (convex fill would be wrong for a histogram).
        for w in pts.windows(2).skip(1).take(255) {
            painter.add(egui::Shape::convex_polygon(
                vec![w[0], w[1], egui::pos2(w[1].x, rect.bottom()), egui::pos2(w[0].x, rect.bottom())],
                color,
                Stroke::NONE,
            ));
        }
    }
    let lum: Vec<egui::Pos2> = (0..256)
        .map(|i| egui::pos2(rect.left() + i as f32 / 255.0 * rect.width(), rect.bottom() - scale(h.lum[i])))
        .collect();
    painter.add(egui::Shape::line(lum, Stroke::new(1.0_f32, with_alpha(p.text, 150))));
}

fn navigator(
    ui: &Ui,
    rect: egui::Rect,
    resp: &egui::Response,
    tex: &egui::TextureHandle,
    pan: &mut egui::Vec2,
    display_size: Option<egui::Vec2>,
    canvas: egui::Vec2,
) {
    let p = pal(ui);
    let img = tex.size_vec2();
    let fit = (rect.width() / img.x).min(rect.height() / img.y) * 0.92;
    let thumb = egui::Rect::from_center_size(rect.center(), img * fit);
    egui::Image::new((tex.id(), thumb.size())).corner_radius(RADIUS_SMALL).paint_at(ui, thumb);

    let Some(display) = display_size.filter(|d| d.x > 0.0 && d.y > 0.0) else {
        return;
    };
    // Visible part of the image, in thumbnail coordinates.
    let vis = egui::vec2((canvas.x / display.x).min(1.0), (canvas.y / display.y).min(1.0)) * thumb.size();
    let centre = thumb.center() - egui::vec2(pan.x / display.x * thumb.width(), pan.y / display.y * thumb.height());
    let view = egui::Rect::from_center_size(centre, vis);
    let painter = ui.painter_at(rect);
    painter.rect_stroke(view, RADIUS_SMALL, Stroke::new(2.0_f32, p.accent), egui::StrokeKind::Middle);
    painter.rect_filled(view, RADIUS_SMALL, with_alpha(p.accent, 30));

    let to_pan = |pos: egui::Pos2| {
        let off = pos - thumb.center();
        egui::vec2(-off.x / thumb.width() * display.x, -off.y / thumb.height() * display.y)
    };
    if resp.dragged() {
        let d = resp.drag_delta();
        pan.x -= d.x / thumb.width() * display.x;
        pan.y -= d.y / thumb.height() * display.y;
    } else if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            *pan = to_pan(pos);
        }
    }
}

fn loupe(ui: &Ui, rect: egui::Rect, tex: &egui::TextureHandle) {
    let p = pal(ui);
    let r = rect.shrink(6.0);
    let uv = egui::Rect::from_center_size(egui::pos2(0.5, 0.5), egui::vec2(0.25, 0.25 * r.height() / r.width() * tex.aspect_ratio()));
    egui::Image::new((tex.id(), r.size())).uv(uv).corner_radius(RADIUS_CONTROL).paint_at(ui, r);
    let c = r.center();
    let painter = ui.painter_at(rect);
    painter.line_segment([c - egui::vec2(8.0, 0.0), c + egui::vec2(8.0, 0.0)], Stroke::new(1.0_f32, p.accent));
    painter.line_segment([c - egui::vec2(0.0, 8.0), c + egui::vec2(0.0, 8.0)], Stroke::new(1.0_f32, p.accent));
}
