use crate::theme::*;
use eframe::egui::{self};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created: Instant,
    pub duration: Duration,
}

pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.add(msg, ToastKind::Info);
    }

    pub fn success(&mut self, msg: impl Into<String>) {
        self.add(msg, ToastKind::Success);
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.add(msg, ToastKind::Warning);
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        self.add(msg, ToastKind::Error);
    }

    fn add(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast {
            message: msg.into(),
            kind,
            created: Instant::now(),
            duration: Duration::from_millis(3200),
        });
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created) < t.duration);

        if self.toasts.is_empty() {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let mut offset_y = screen_rect.max.y - 38.0;

        for toast in self.toasts.iter().rev() {
            let (bg_color, fg_color) = match toast.kind {
                ToastKind::Info => (SURFACE_RAISED, ACCENT),
                ToastKind::Success => (SURFACE_RAISED, STATUS_SUCCESS),
                ToastKind::Warning => (SURFACE_RAISED, STATUS_WARNING),
                ToastKind::Error => (SURFACE_RAISED, STATUS_ERROR),
            };

            let toast_pos = egui::pos2(screen_rect.max.x - 22.0, offset_y);

            egui::Area::new(egui::Id::new("toast_area").with(&toast.message))
                .fixed_pos(toast_pos)
                .pivot(egui::Align2::RIGHT_BOTTOM)
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(1.0_f32, BORDER_SUBTLE))
                        .corner_radius(RADIUS_CONTROL)
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("●").size(10.0).color(fg_color));
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .size(12.5)
                                        .color(TEXT_PRIMARY),
                                );
                            });
                        });
                });

            offset_y -= 40.0;
        }
    }
}
