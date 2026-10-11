use crate::image_buffer::RgbImageF32;
use crate::params::FinishingParams;
use crate::pipeline::RenderContext;
use rayon::prelude::*;

pub fn apply_finishing(
    processed: &mut RgbImageF32,
    original: &RgbImageF32,
    params: &FinishingParams,
    ctx: &RenderContext,
) {
    if !params.enabled {
        return;
    }

    let width = processed.width;
    if width == 0 || processed.height == 0 {
        return;
    }
    // Positions below are in full-image coordinates so tiles match a full render.
    let full_w = ctx.full_width as f32;
    let full_h = ctx.full_height as f32;
    let off_x = ctx.offset_x;
    let off_y = ctx.offset_y;

    // 1. Warmth (White Balance adjustment)
    if params.warmth != 0.0 {
        let w = params.warmth * 0.15;
        processed
            .r
            .par_iter_mut()
            .zip(processed.g.par_iter_mut())
            .zip(processed.b.par_iter_mut())
            .for_each(|((r, g), b)| {
                *r = (*r + w).clamp(0.0, 1.0);
                *g = (*g + w * 0.2).clamp(0.0, 1.0);
                *b = (*b - w).clamp(0.0, 1.0);
            });
    }

    // 2. Vignette
    if params.vignette.amount != 0.0 {
        let amount = params.vignette.amount;
        let v_size = params.vignette.size;
        let transition = params.vignette.transition.max(0.01);
        let roundness = params.vignette.roundness;

        // Half-extents; `max(0.5)` keeps 1px-wide/tall images from dividing by zero.
        let cx = ((full_w - 1.0) / 2.0).max(0.5);
        let cy = ((full_h - 1.0) / 2.0).max(0.5);
        let max_r = (cx * cx + cy * cy).sqrt();

        for_each_row(processed, |y, row_r, row_g, row_b| {
            let dy = (y + off_y) as f32 - cy;
            for x in 0..row_r.len() {
                let dx = (x + off_x) as f32 - cx;

                // Interpolate between rectangle Euclidean and circular metric
                let dist_circle = (dx * dx + dy * dy).sqrt() / max_r;
                let dist_rect = (dx.abs() / cx).max(dy.abs() / cy);
                let dist = (1.0 - roundness) * dist_rect + roundness * dist_circle;

                let t = ((dist - v_size) / transition).clamp(0.0, 1.0);
                let mask = t * t * (3.0 - 2.0 * t); // Smoothstep falloff
                let factor = (1.0 + amount * mask).clamp(0.0, 2.0);

                row_r[x] = (row_r[x] * factor).clamp(0.0, 1.0);
                row_g[x] = (row_g[x] * factor).clamp(0.0, 1.0);
                row_b[x] = (row_b[x] * factor).clamp(0.0, 1.0);
            }
        });
    }

    // 3. Procedural Film Grain
    if params.grain.amount > 0.0 {
        let grain_amt = params.grain.amount * 0.15;
        // Grain cells are `size` reference pixels wide, so grain looks the same at any resolution.
        let cell = (params.grain.size * ctx.scale()).max(1.0);

        for_each_row(processed, |y, row_r, row_g, row_b| {
            let gy = ((y + off_y) as f32 / cell) as u32;
            for x in 0..row_r.len() {
                let gx = ((x + off_x) as f32 / cell) as u32;
                let norm = grain_hash(gx, gy);

                let lum = 0.2126 * row_r[x] + 0.7152 * row_g[x] + 0.0722 * row_b[x];
                // Film grain is physically strongest in midtones, softer in specular highlights/blacks
                let midtone_mod = (lum * (1.0 - lum)).max(0.0).sqrt().max(0.1);
                let noise = norm * grain_amt * midtone_mod;

                row_r[x] = (row_r[x] + noise).clamp(0.0, 1.0);
                row_g[x] = (row_g[x] + noise).clamp(0.0, 1.0);
                row_b[x] = (row_b[x] + noise).clamp(0.0, 1.0);
            }
        });
    }

    // 4. Border (Frame)
    if params.border.enabled && params.border.thickness > 0.0 {
        let full_min = ctx.full_width.min(ctx.full_height);
        // Never wider than half the image, so the inner edge can't underflow.
        let border_px = ((full_min as f32 * params.border.thickness).round().max(1.0) as usize)
            .min(full_min.div_ceil(2));
        let [br, bg, bb] = params.border.color;
        let (fw, fh) = (ctx.full_width, ctx.full_height);

        for_each_row(processed, |y, row_r, row_g, row_b| {
            let gy = y + off_y;
            let is_top_bottom = gy < border_px || gy + border_px >= fh;
            for x in 0..row_r.len() {
                let gx = x + off_x;
                if is_top_bottom || gx < border_px || gx + border_px >= fw {
                    row_r[x] = br;
                    row_g[x] = bg;
                    row_b[x] = bb;
                }
            }
        });
    }

    // 5. Master Transparency Blending (blend with original)
    if params.transparency > 0.0 {
        let t = params.transparency.clamp(0.0, 1.0);
        let eff = 1.0 - t;

        processed
            .r
            .par_iter_mut()
            .zip(processed.g.par_iter_mut())
            .zip(processed.b.par_iter_mut())
            .zip(original.r.par_iter())
            .zip(original.g.par_iter())
            .zip(original.b.par_iter())
            .for_each(|(((((pr, pg), pb), &or), &og), &ob)| {
                *pr = eff * *pr + t * or;
                *pg = eff * *pg + t * og;
                *pb = eff * *pb + t * ob;
            });
    }
}

/// Runs `f(y, r_row, g_row, b_row)` over every row in parallel.
fn for_each_row<F>(img: &mut RgbImageF32, f: F)
where
    F: Fn(usize, &mut [f32], &mut [f32], &mut [f32]) + Sync,
{
    let width = img.width;
    img.r
        .par_chunks_mut(width)
        .zip(img.g.par_chunks_mut(width))
        .zip(img.b.par_chunks_mut(width))
        .enumerate()
        .for_each(|(y, ((r, g), b))| f(y, r, g, b));
}

/// Deterministic per-cell noise in `[-0.5, 0.5]`.
#[inline(always)]
fn grain_hash(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ 0x5bf0_3635;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0 - 0.5
}
