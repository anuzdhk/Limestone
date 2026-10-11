//! Transient HUD notifications, centred near the bottom of the canvas.

use crate::theme::*;
use eframe::egui;
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
        let message = msg.into();
        // Replace an identical visible toast instead of stacking duplicates.
        self.toasts.retain(|t| t.message != message);
        self.toasts.push(Toast {
            message,
            kind,
            created: Instant::now(),
            duration: Duration::from_millis(if kind == ToastKind::Error { 5000 } else { 2600 }),
        });
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
    }

    /// Draws toasts centred over `area` (normally the canvas), newest at the bottom.
    pub fn show(&mut self, ctx: &egui::Context, area: egui::Rect) {
        let now = Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created) < t.duration);
        if self.toasts.is_empty() {
            return;
        }
        let p = pal_ctx(ctx);
        let fill = if p.dark {
            egui::Color32::from_rgba_unmultiplied(58, 58, 62, 242)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 246)
        };
        let mut y = area.max.y - 56.0;
        for (i, toast) in self.toasts.iter().rev().enumerate() {
            let age = now.duration_since(toast.created).as_secs_f32();
            let remaining = (toast.duration.as_secs_f32() - age).max(0.0);
            let alpha = (age / 0.15).min(1.0).min(remaining / 0.3);
            let dot = match toast.kind {
                ToastKind::Info => p.accent,
                ToastKind::Success => p.success,
                ToastKind::Warning => p.warning,
                ToastKind::Error => p.danger,
            };
            egui::Area::new(egui::Id::new("toast").with(i))
                .fixed_pos(egui::pos2(area.center().x, y))
                .pivot(egui::Align2::CENTER_BOTTOM)
                .order(egui::Order::Tooltip)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.set_opacity(alpha);
                    egui::Frame::new()
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0_f32, p.separator))
                        .corner_radius(egui::CornerRadius::same(18))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 4],
                            blur: 16,
                            spread: 0,
                            color: p.shadow,
                        })
                        .inner_margin(egui::Margin::symmetric(16, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("●").size(9.0).color(dot));
                                ui.label(egui::RichText::new(&toast.message).size(13.0).color(p.text));
                            });
                        });
                });
            y -= 44.0;
        }
        // Keep animating fades and expire toasts even when nothing else repaints.
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
