use crate::image_buffer::RgbImageF32;
use crate::params::FinishingParams;
use rayon::prelude::*;

pub fn apply_finishing(
    processed: &mut RgbImageF32,
    original: &RgbImageF32,
    params: &FinishingParams,
) {
    if !params.enabled {
        return;
    }

    let width = processed.width;
    let height = processed.height;

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

        let cx = (width as f32 - 1.0) / 2.0;
        let cy = (height as f32 - 1.0) / 2.0;
        let max_r = (cx * cx + cy * cy).sqrt();

        processed
            .r
            .par_chunks_mut(width)
            .zip(processed.g.par_chunks_mut(width))
            .zip(processed.b.par_chunks_mut(width))
            .enumerate()
            .for_each(|(y, ((row_r, row_g), row_b))| {
                let dy = y as f32 - cy;
                for x in 0..width {
                    let dx = x as f32 - cx;

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
        processed
            .r
            .par_chunks_mut(width)
            .zip(processed.g.par_chunks_mut(width))
            .zip(processed.b.par_chunks_mut(width))
            .enumerate()
            .for_each(|(y, ((row_r, row_g), row_b))| {
                for x in 0..width {
                    // Fast deterministic LCG pseudo-random noise generator
                    let seed = ((x * 1973 + y * 9277 + 26699) ^ 0x5bf03635) as u32;
                    let hash = seed.wrapping_mul(0x45d9f3b);
                    let norm = (hash & 0xffff) as f32 / 65535.0 - 0.5;

                    let lum = 0.2126 * row_r[x] + 0.7152 * row_g[x] + 0.0722 * row_b[x];
                    // Film grain is physically strongest in midtones, softer in specular highlights/blacks
                    let midtone_mod = (lum * (1.0 - lum)).sqrt().max(0.1);
                    let noise = norm * grain_amt * midtone_mod;

                    row_r[x] = (row_r[x] + noise).clamp(0.0, 1.0);
                    row_g[x] = (row_g[x] + noise).clamp(0.0, 1.0);
                    row_b[x] = (row_b[x] + noise).clamp(0.0, 1.0);
                }
            });
    }

    // 4. Border (Frame)
    if params.border.enabled && params.border.thickness > 0.0 {
        let min_dim = width.min(height) as f32;
        let border_px = (min_dim * params.border.thickness).round().max(1.0) as usize;
        let [br, bg, bb] = params.border.color;

        processed
            .r
            .par_chunks_mut(width)
            .zip(processed.g.par_chunks_mut(width))
            .zip(processed.b.par_chunks_mut(width))
            .enumerate()
            .for_each(|(y, ((row_r, row_g), row_b))| {
                let is_top_bottom = y < border_px || y >= height - border_px;
                for x in 0..width {
                    if is_top_bottom || x < border_px || x >= width - border_px {
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
