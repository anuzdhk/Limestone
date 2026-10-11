use image::{DynamicImage, ImageBuffer, ImageReader, Rgb, RgbImage, Rgba, RgbaImage};
use rayon::prelude::*;
use std::path::Path;

/// Planar 32-bit floating-point RGB image representation (gamma-encoded sRGB, `0.0..=1.0`).
///
/// An optional alpha plane is carried through processing untouched so transparent
/// inputs keep their transparency on export.
#[derive(Clone, Debug)]
pub struct RgbImageF32 {
    pub width: usize,
    pub height: usize,
    pub r: Vec<f32>,
    pub g: Vec<f32>,
    pub b: Vec<f32>,
    pub alpha: Option<Vec<f32>>,
    /// True when the source had more than 8 bits per channel; exports keep 16-bit where possible.
    pub high_bit_depth: bool,
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
            alpha: None,
            high_bit_depth: false,
        }
    }

    pub fn from_rgb8(img: &RgbImage) -> Self {
        let mut out = Self::new(img.width() as usize, img.height() as usize);
        for (i, pixel) in img.pixels().enumerate() {
            out.r[i] = pixel[0] as f32 / 255.0;
            out.g[i] = pixel[1] as f32 / 255.0;
            out.b[i] = pixel[2] as f32 / 255.0;
        }
        out
    }

    pub fn from_rgba8(img: &RgbaImage) -> Self {
        let mut out = Self::new(img.width() as usize, img.height() as usize);
        let mut alpha = vec![1.0; out.width * out.height];
        for (i, pixel) in img.pixels().enumerate() {
            out.r[i] = pixel[0] as f32 / 255.0;
            out.g[i] = pixel[1] as f32 / 255.0;
            out.b[i] = pixel[2] as f32 / 255.0;
            alpha[i] = pixel[3] as f32 / 255.0;
        }
        out.alpha = Some(alpha);
        out
    }

    /// Converts any decoded image, keeping >8-bit precision and alpha when present.
    pub fn from_dynamic(img: &DynamicImage) -> Self {
        let color = img.color();
        let high_bit_depth = color.bytes_per_pixel() / color.channel_count() > 1;
        let has_alpha = color.has_alpha();

        let rgba = img.to_rgba32f();
        let mut out = Self::new(rgba.width() as usize, rgba.height() as usize);
        let mut alpha = vec![1.0; out.width * out.height];
        for (i, pixel) in rgba.pixels().enumerate() {
            out.r[i] = pixel[0].clamp(0.0, 1.0);
            out.g[i] = pixel[1].clamp(0.0, 1.0);
            out.b[i] = pixel[2].clamp(0.0, 1.0);
            alpha[i] = pixel[3].clamp(0.0, 1.0);
        }
        if has_alpha {
            out.alpha = Some(alpha);
        }
        out.high_bit_depth = high_bit_depth;
        out
    }

    /// Opens an image file, applying its EXIF orientation.
    pub fn open(path: &Path) -> Result<Self, image::ImageError> {
        Ok(Self::from_dynamic(&open_oriented(path)?))
    }

    pub fn to_rgb8(&self) -> RgbImage {
        let raw: Vec<u8> = (0..self.width * self.height)
            .into_par_iter()
            .flat_map_iter(|i| [to_u8(self.r[i]), to_u8(self.g[i]), to_u8(self.b[i])])
            .collect();
        RgbImage::from_raw(self.width as u32, self.height as u32, raw)
            .expect("buffer size matches dimensions")
    }

    pub fn to_rgba8(&self) -> RgbaImage {
        let raw: Vec<u8> = (0..self.width * self.height)
            .into_par_iter()
            .flat_map_iter(|i| {
                let a = self.alpha.as_ref().map_or(1.0, |a| a[i]);
                [to_u8(self.r[i]), to_u8(self.g[i]), to_u8(self.b[i]), to_u8(a)]
            })
            .collect();
        RgbaImage::from_raw(self.width as u32, self.height as u32, raw)
            .expect("buffer size matches dimensions")
    }

    fn to_dynamic16(&self) -> DynamicImage {
        let w = self.width as u32;
        let h = self.height as u32;
        if let Some(ref alpha) = self.alpha {
            let raw: Vec<u16> = (0..self.width * self.height)
                .into_par_iter()
                .flat_map_iter(|i| {
                    [to_u16(self.r[i]), to_u16(self.g[i]), to_u16(self.b[i]), to_u16(alpha[i])]
                })
                .collect();
            DynamicImage::ImageRgba16(
                ImageBuffer::<Rgba<u16>, _>::from_raw(w, h, raw).expect("buffer size matches"),
            )
        } else {
            let raw: Vec<u16> = (0..self.width * self.height)
                .into_par_iter()
                .flat_map_iter(|i| [to_u16(self.r[i]), to_u16(self.g[i]), to_u16(self.b[i])])
                .collect();
            DynamicImage::ImageRgb16(
                ImageBuffer::<Rgb<u16>, _>::from_raw(w, h, raw).expect("buffer size matches"),
            )
        }
    }

    /// Saves the image, picking bit depth and alpha support from the file format:
    /// PNG/TIFF keep 16-bit and alpha, JPEG is always 8-bit RGB.
    pub fn save(&self, path: &Path) -> Result<(), image::ImageError> {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let supports_16 = matches!(ext.as_str(), "png" | "tif" | "tiff");
        let supports_alpha = matches!(ext.as_str(), "png" | "tif" | "tiff" | "webp");

        if self.high_bit_depth && supports_16 {
            self.to_dynamic16().save(path)
        } else if self.alpha.is_some() && supports_alpha {
            self.to_rgba8().save(path)
        } else {
            self.to_rgb8().save(path)
        }
    }

    /// Extract standard Rec. 709 / sRGB perceptual luminance Y
    pub fn compute_luminance(&self) -> Vec<f32> {
        let mut y = vec![0.0; self.width * self.height];
        y.par_iter_mut().enumerate().for_each(|(i, val)| {
            *val = 0.2126 * self.r[i] + 0.7152 * self.g[i] + 0.0722 * self.b[i];
        });
        y
    }

    /// Copies a rectangular region. The rectangle must lie inside the image.
    pub fn crop(&self, x0: usize, y0: usize, w: usize, h: usize) -> Self {
        assert!(x0 + w <= self.width && y0 + h <= self.height, "crop out of bounds");
        let mut out = Self::new(w, h);
        out.high_bit_depth = self.high_bit_depth;
        let copy = |src: &[f32], dst: &mut [f32]| {
            for dy in 0..h {
                let s = (y0 + dy) * self.width + x0;
                dst[dy * w..(dy + 1) * w].copy_from_slice(&src[s..s + w]);
            }
        };
        copy(&self.r, &mut out.r);
        copy(&self.g, &mut out.g);
        copy(&self.b, &mut out.b);
        if let Some(ref a) = self.alpha {
            let mut dst = vec![0.0; w * h];
            copy(a, &mut dst);
            out.alpha = Some(dst);
        }
        out
    }

    /// Create an area-averaged (box-filtered) thumbnail for fast live-preview editing.
    /// Averaging instead of nearest-neighbour sampling avoids aliasing, so the preview's
    /// texture statistics match the full-resolution image.
    pub fn downsample(&self, max_dim: usize) -> Self {
        let max_current = self.width.max(self.height);
        if max_current <= max_dim || max_dim == 0 {
            return self.clone();
        }

        let scale = max_dim as f32 / max_current as f32;
        let new_w = (self.width as f32 * scale).round().max(1.0) as usize;
        let new_h = (self.height as f32 * scale).round().max(1.0) as usize;

        let x_bounds: Vec<(usize, usize)> = (0..new_w)
            .map(|dx| span(dx, new_w, self.width))
            .collect();

        let resample = |src: &[f32]| -> Vec<f32> {
            let mut dst = vec![0.0; new_w * new_h];
            dst.par_chunks_mut(new_w).enumerate().for_each(|(dy, row)| {
                let (sy0, sy1) = span(dy, new_h, self.height);
                for (dx, out) in row.iter_mut().enumerate() {
                    let (sx0, sx1) = x_bounds[dx];
                    let mut sum = 0.0;
                    for sy in sy0..sy1 {
                        let base = sy * self.width;
                        sum += src[base + sx0..base + sx1].iter().sum::<f32>();
                    }
                    *out = sum / ((sy1 - sy0) * (sx1 - sx0)) as f32;
                }
            });
            dst
        };

        Self {
            width: new_w,
            height: new_h,
            r: resample(&self.r),
            g: resample(&self.g),
            b: resample(&self.b),
            alpha: self.alpha.as_ref().map(|a| resample(a)),
            high_bit_depth: self.high_bit_depth,
        }
    }
}

/// Opens an image file and applies its EXIF orientation tag (if any).
pub fn open_oriented(path: &Path) -> Result<DynamicImage, image::ImageError> {
    use image::ImageDecoder;
    let mut decoder = ImageReader::open(path)?.with_guessed_format()?.into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img)
}

/// Source pixel range `[start, end)` covered by destination index `d` when resampling `src_len -> dst_len`.
fn span(d: usize, dst_len: usize, src_len: usize) -> (usize, usize) {
    let start = d * src_len / dst_len;
    let end = ((d + 1) * src_len / dst_len).max(start + 1).min(src_len);
    (start, end)
}

#[inline(always)]
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

#[inline(always)]
fn to_u16(v: f32) -> u16 {
    (v.clamp(0.0, 1.0) * 65535.0 + 0.5) as u16
}
