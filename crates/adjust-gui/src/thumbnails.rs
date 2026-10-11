//! Live preset thumbnails: every preset rendered on a small copy of the current image.
//!
//! Runs on its own thread so thumbnail generation never delays the main preview.
//! A newer request cancels the remainder of an older batch.

use crate::render_worker::to_color_image;
use adjust_core::{process_image, AdjustParams, RgbImageF32};
use eframe::egui;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

/// Longest side of a thumbnail source image.
pub const THUMB_DIM: usize = 200;

struct Batch {
    version: u64,
    image: Arc<RgbImageF32>,
    items: Vec<(String, AdjustParams)>,
}

pub struct ThumbnailCache {
    sender: Sender<Batch>,
    receiver: Receiver<(u64, String, egui::ColorImage)>,
    latest: Arc<AtomicU64>,
    version: u64,
    textures: HashMap<String, egui::TextureHandle>,
}

impl ThumbnailCache {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, rx) = channel::<Batch>();
        let (res_tx, res_rx) = channel();
        let latest = Arc::new(AtomicU64::new(0));
        let latest_worker = latest.clone();

        thread::spawn(move || {
            while let Ok(mut batch) = rx.recv() {
                // Skip straight to the newest batch.
                while let Ok(newer) = rx.try_recv() {
                    batch = newer;
                }
                for (name, params) in batch.items {
                    if latest_worker.load(Ordering::Relaxed) != batch.version {
                        break; // superseded
                    }
                    let img = process_image(&batch.image, &params);
                    if res_tx.send((batch.version, name, to_color_image(&img))).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
            }
        });

        Self {
            sender: tx,
            receiver: res_rx,
            latest,
            version: 0,
            textures: HashMap::new(),
        }
    }

    /// Re-renders all thumbnails for a new source image. `items` should list the
    /// presets the user is looking at first.
    pub fn rebuild(&mut self, source: &RgbImageF32, items: Vec<(String, AdjustParams)>) {
        self.version += 1;
        self.latest.store(self.version, Ordering::Relaxed);
        let _ = self.sender.send(Batch {
            version: self.version,
            image: Arc::new(source.downsample(THUMB_DIM)),
            items,
        });
    }

    /// Renders (or re-renders) a single preset, e.g. a newly saved user preset.
    pub fn render_one(&mut self, source: &RgbImageF32, name: String, params: AdjustParams) {
        let _ = self.sender.send(Batch {
            version: self.version,
            image: Arc::new(source.downsample(THUMB_DIM)),
            items: vec![(name, params)],
        });
    }

    /// Uploads finished thumbnails as textures.
    pub fn poll(&mut self, ctx: &egui::Context) {
        for (version, name, image) in self.receiver.try_iter() {
            if version != self.version {
                continue;
            }
            match self.textures.get_mut(&name) {
                Some(tex) => tex.set(image, egui::TextureOptions::LINEAR),
                None => {
                    let tex = ctx.load_texture(format!("thumb:{name}"), image, egui::TextureOptions::LINEAR);
                    self.textures.insert(name, tex);
                }
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&egui::TextureHandle> {
        self.textures.get(name)
    }
}
