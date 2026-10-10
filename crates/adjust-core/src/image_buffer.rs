use image::{RgbImage, RgbaImage};
use rayon::prelude::*;

/// Planar 32-bit floating-point RGB image representation.
/// Linear range [0.0, 1.0] for processing.
#[derive(Clone, Debug)]
pub struct RgbImageF32 {
    pub width: usize,
    pub height: usize,
    pub r: Vec<f32>,
    pub g: Vec<f32>,
    pub b: Vec<f32>,
}

impl RgbImageF32 {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            r: vec![0.0; size],
            g: vec![0.0; size],
            b: vec![0.0; size],
        }
    }

    pub fn from_rgb8(img: &RgbImage) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let size = width * height;
        let mut r = Vec::with_capacity(size);
        let mut g = Vec::with_capacity(size);
        let mut b = Vec::with_capacity(size);

        for pixel in img.pixels() {
            r.push(pixel[0] as f32 / 255.0);
            g.push(pixel[1] as f32 / 255.0);
            b.push(pixel[2] as f32 / 255.0);
        }

        Self {
            width,
            height,
            r,
            g,
            b,
        }
    }

    pub fn from_rgba8(img: &RgbaImage) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let size = width * height;
        let mut r = Vec::with_capacity(size);
        let mut g = Vec::with_capacity(size);
        let mut b = Vec::with_capacity(size);

        for pixel in img.pixels() {
            r.push(pixel[0] as f32 / 255.0);
            g.push(pixel[1] as f32 / 255.0);
            b.push(pixel[2] as f32 / 255.0);
        }

        Self {
            width,
            height,
            r,
            g,
            b,
        }
    }

    pub fn to_rgb8(&self) -> RgbImage {
        let mut out = RgbImage::new(self.width as u32, self.height as u32);
        let size = self.width * self.height;

        let raw_pixels: Vec<u8> = (0..size)
            .into_par_iter()
            .flat_map(|i| {
                let cr = (self.r[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let cg = (self.g[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let cb = (self.b[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                [cr, cg, cb]
            })
            .collect();

        out.as_mut().copy_from_slice(&raw_pixels);
        out
    }

    /// Extract standard Rec. 709 / sRGB perceptual luminance Y
    pub fn compute_luminance(&self) -> Vec<f32> {
        let size = self.width * self.height;
        let mut y = vec![0.0; size];
        y.par_iter_mut()
            .enumerate()
            .for_each(|(i, val)| {
                *val = 0.2126 * self.r[i] + 0.7152 * self.g[i] + 0.0722 * self.b[i];
            });
        y
    }

    /// Create downsampled thumbnail for fast live-preview editing
    pub fn downsample(&self, max_dim: usize) -> Self {
        let max_current = self.width.max(self.height);
        if max_current <= max_dim {
            return self.clone();
        }

        let scale = max_dim as f32 / max_current as f32;
        let new_w = (self.width as f32 * scale).round().max(1.0) as usize;
        let new_h = (self.height as f32 * scale).round().max(1.0) as usize;
        let mut down = Self::new(new_w, new_h);

        let x_ratio = self.width as f32 / new_w as f32;
        let y_ratio = self.height as f32 / new_h as f32;

        for dy in 0..new_h {
            let sy = ((dy as f32 * y_ratio) as usize).min(self.height - 1);
            for dx in 0..new_w {
                let sx = ((dx as f32 * x_ratio) as usize).min(self.width - 1);
                let src_idx = sy * self.width + sx;
                let dst_idx = dy * new_w + dx;

                down.r[dst_idx] = self.r[src_idx];
                down.g[dst_idx] = self.g[src_idx];
                down.b[dst_idx] = self.b[src_idx];
            }
        }

        down
    }
}
