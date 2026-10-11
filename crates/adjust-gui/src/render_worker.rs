//! Background rendering so the UI thread never blocks on image processing.
//!
//! The worker coalesces requests: if several previews are queued (e.g. while a slider
//! is being dragged) only the newest one is rendered.

use adjust_core::{compute_histogram, process_image, process_region, AdjustParams, HistogramData, RgbImageF32};
use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Longest side of the interactive preview proxy.
pub const PREVIEW_MAX_DIM: usize = 960;

/// A full-resolution region of the current base image, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

/// Identifies what a tile was rendered from, so stale tiles are never shown.
#[derive(Clone, Debug, PartialEq)]
pub struct TileKey {
    pub base_version: u64,
    pub params: AdjustParams,
    pub rect: PixelRect,
}

enum Request {
    Preview {
        generation: u64,
        image: Arc<RgbImageF32>,
        params: AdjustParams,
    },
    Tile {
        key: TileKey,
        full: Arc<RgbImageF32>,
    },
    Bake {
        base_version: u64,
        start: Arc<RgbImageF32>,
        layers: Vec<AdjustParams>,
    },
}

pub enum RenderResult {
    Preview {
        generation: u64,
        color_image: egui::ColorImage,
        histogram: Box<HistogramData>,
        elapsed_ms: f32,
    },
    Tile {
        key: Box<TileKey>,
        color_image: egui::ColorImage,
    },
    /// A new full-resolution base image (layers baked in) plus its preview proxy.
    Baked {
        base_version: u64,
        base: Arc<RgbImageF32>,
        preview: Arc<RgbImageF32>,
    },
}

pub struct RenderWorker {
    sender: Sender<Request>,
    receiver: Receiver<RenderResult>,
}

impl RenderWorker {
    pub fn new(ctx: egui::Context) -> Self {
        let (req_tx, req_rx) = channel::<Request>();
        let (res_tx, res_rx) = channel::<RenderResult>();

        thread::spawn(move || {
            while let Ok(first) = req_rx.recv() {
                // Drain the queue, keeping only the newest request of each kind.
                // Bakes are never dropped: each one changes the base image.
                let mut bakes = Vec::new();
                let mut preview = None;
                let mut tile = None;
                for req in std::iter::once(first).chain(req_rx.try_iter()) {
                    match req {
                        r @ Request::Bake { .. } => bakes.push(r),
                        r @ Request::Preview { .. } => preview = Some(r),
                        r @ Request::Tile { .. } => tile = Some(r),
                    }
                }

                for req in bakes.into_iter().chain(preview).chain(tile) {
                    let result = render(req);
                    if res_tx.send(result).is_err() {
                        return; // UI is gone
                    }
                    ctx.request_repaint();
                }
            }
        });

        Self {
            sender: req_tx,
            receiver: res_rx,
        }
    }

    pub fn request_preview(&self, generation: u64, image: Arc<RgbImageF32>, params: AdjustParams) {
        let _ = self.sender.send(Request::Preview {
            generation,
            image,
            params,
        });
    }

    pub fn request_tile(&self, key: TileKey, full: Arc<RgbImageF32>) {
        let _ = self.sender.send(Request::Tile { key, full });
    }

    /// Applies `layers` in order on top of `start`, producing a new base image.
    pub fn request_bake(&self, base_version: u64, start: Arc<RgbImageF32>, layers: Vec<AdjustParams>) {
        let _ = self.sender.send(Request::Bake {
            base_version,
            start,
            layers,
        });
    }

    pub fn poll(&self) -> Vec<RenderResult> {
        self.receiver.try_iter().collect()
    }
}

fn render(req: Request) -> RenderResult {
    match req {
        Request::Preview {
            generation,
            image,
            params,
        } => {
            let start = Instant::now();
            let processed = process_image(&image, &params);
            let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
            RenderResult::Preview {
                generation,
                color_image: to_color_image(&processed),
                histogram: Box::new(compute_histogram(&processed)),
                elapsed_ms,
            }
        }
        Request::Tile { key, full } => {
            let r = key.rect;
            let tile = process_region(&full, &key.params, r.x, r.y, r.w, r.h);
            RenderResult::Tile {
                color_image: to_color_image(&tile),
                key: Box::new(key),
            }
        }
        Request::Bake {
            base_version,
            start,
            layers,
        } => {
            let mut base = start;
            for params in &layers {
                base = Arc::new(process_image(&base, params));
            }
            let preview = Arc::new(base.downsample(PREVIEW_MAX_DIM));
            RenderResult::Baked {
                base_version,
                base,
                preview,
            }
        }
    }
}

pub fn to_color_image(img: &RgbImageF32) -> egui::ColorImage {
    let rgb8 = img.to_rgb8();
    egui::ColorImage::from_rgb([img.width, img.height], rgb8.as_raw())
}
