//! Unified window toolbar (macOS style): sidebar & file actions on the left, compare
//! modes in the centre, history / zoom / export on the right.

use crate::icons::*;
use crate::theme::*;
use crate::widgets::controls::{icon_button, pill_button, segmented};
use eframe::egui::{self, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Single,
    Split,
    SideBySide,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolbarAction {
    ToggleSidebar,
    ToggleInspector,
    Open,
    Paste,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Fit,
    ActualSize,
    Export,
    Settings,
    Shortcuts,
}

pub struct ToolbarState<'a> {
    pub view_mode: &'a mut ViewMode,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub zoom: f32,
    pub can_undo: bool,
    pub can_redo: bool,
    pub has_image: bool,
    pub show_sidebar: bool,
    pub show_inspector: bool,
    pub busy: bool,
}

pub fn toolbar(ui: &mut Ui, s: ToolbarState) -> Vec<ToolbarAction> {
    use ToolbarAction::*;
    let p = pal(ui);
    let mut actions = Vec::new();
    let full = ui.max_rect();

    // One row: left group, then a right-to-left group filling the rest. Nesting (rather
    // than overlapping full-width scopes) keeps every button on top for hit-testing.
    ui.horizontal_centered(|ui| {
        // ---- Left group ----------------------------------------------------
        ui.add_space(SPACE_8);
        if icon_button(ui, ICON_SIDEBAR_LEFT, "Show/hide presets (Tab)", true, s.show_sidebar).clicked() {
            actions.push(ToggleSidebar);
        }
        ui.add_space(SPACE_4);
        if icon_button(ui, ICON_FOLDER_OPEN, "Open image… (Ctrl+O)", true, false).clicked() {
            actions.push(Open);
        }
        if icon_button(ui, ICON_CLIPBOARD, "Paste image (Ctrl+V)", true, false).clicked() {
            actions.push(Paste);
        }
        ui.add_space(SPACE_8);
        // Title block, truncated so it never pushes into the centre controls.
        let title_w = (full.width() * 0.5 - 3.0 * 84.0 / 2.0 - 140.0).clamp(60.0, 360.0);
        ui.allocate_ui(egui::vec2(title_w, HEIGHT_TOOLBAR - 8.0), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.add(egui::Label::new(egui::RichText::new(s.title).size(13.0).strong().color(p.text)).truncate());
                ui.add(egui::Label::new(egui::RichText::new(s.subtitle).size(11.0).color(p.text_secondary)).truncate());
            });
        });

        // ---- Right group (right-to-left) ------------------------------------
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(SPACE_8);
            if icon_button(ui, ICON_SIDEBAR_RIGHT, "Show/hide adjustments", true, s.show_inspector).clicked() {
                actions.push(ToggleInspector);
            }
            if icon_button(ui, ICON_SETTINGS, "Settings (Ctrl+,)", true, false).clicked() {
                actions.push(Settings);
            }
            if icon_button(ui, ICON_KEYBOARD, "Keyboard shortcuts (?)", true, false).clicked() {
                actions.push(Shortcuts);
            }
            ui.add_space(SPACE_4);
            if pill_button(ui, &format!("{ICON_EXPORT}  Export"), true, s.has_image)
                .on_hover_text("Export the full-resolution image (Ctrl+E)")
                .clicked()
            {
                actions.push(Export);
            }
            ui.add_space(SPACE_12);

            // Zoom cluster
            if icon_button(ui, ICON_ZOOM_IN, "Zoom in (+)", s.has_image, false).clicked() {
                actions.push(ZoomIn);
            }
            let pct = ui
                .add(
                    egui::Label::new(
                        egui::RichText::new(format!("{:>4.0}%", s.zoom * 100.0))
                            .size(12.0)
                            .monospace()
                            .color(p.text_secondary),
                    )
                    .sense(egui::Sense::click()),
                )
                .on_hover_text("Click: actual size (0)
Double-click: fit (F)");
            if pct.double_clicked() {
                actions.push(Fit);
            } else if pct.clicked() {
                actions.push(ActualSize);
            }
            if icon_button(ui, ICON_ZOOM_OUT, "Zoom out (−)", s.has_image, false).clicked() {
                actions.push(ZoomOut);
            }
            if icon_button(ui, ICON_FIT, "Fit to window (F)", s.has_image, false).clicked() {
                actions.push(Fit);
            }
            ui.add_space(SPACE_12);
            if icon_button(ui, ICON_REDO, "Redo (Ctrl+Shift+Z)", s.can_redo, false).clicked() {
                actions.push(Redo);
            }
            if icon_button(ui, ICON_UNDO, "Undo (Ctrl+Z)", s.can_undo, false).clicked() {
                actions.push(Undo);
            }
            if s.busy {
                ui.add_space(SPACE_8);
                ui.add(egui::Spinner::new().size(14.0).color(p.text_secondary));
            }

            // ---- Centre: compare modes, centred on the window if it fits ----
            let gap = ui.available_rect_before_wrap();
            let seg_w = 84.0;
            let size = egui::vec2(3.0 * seg_w + 4.0, CONTROL_HEIGHT + 2.0);
            let mut centre = egui::Rect::from_center_size(egui::pos2(full.center().x, gap.center().y), size);
            if centre.left() < gap.left() + 12.0 {
                centre = centre.translate(egui::vec2(gap.left() + 12.0 - centre.left(), 0.0));
            }
            if centre.right() <= gap.right() - 12.0 {
                ui.scope_builder(egui::UiBuilder::new().max_rect(centre), |ui| {
                    segmented(
                        ui,
                        s.view_mode,
                        &[
                            (ViewMode::Single, "Single", "Single view — hold Space to see the original (1)"),
                            (ViewMode::Split, "Split", "Before/after wipe (2)"),
                            (ViewMode::SideBySide, "Side by Side", "Before and after side by side (3)"),
                        ],
                        seg_w,
                    );
                });
            }
        });
    });

    actions
}
