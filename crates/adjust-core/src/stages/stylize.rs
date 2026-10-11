use crate::filter::box_filter;
use crate::image_buffer::RgbImageF32;
use crate::params::StylizeParams;
use crate::pipeline::RenderContext;
use rayon::prelude::*;

/// Sketch line radius in pixels (reference pixels scaled to the full image).
pub fn sketch_radius_px(params: &StylizeParams, ctx: &RenderContext) -> usize {
    (params.sketch_radius * ctx.scale()).round().max(1.0) as usize
}

/// Glow blur radius in pixels (reference pixels scaled to the full image).
pub fn glow_radius_px(params: &StylizeParams, ctx: &RenderContext) -> usize {
    (params.glow_radius * ctx.scale()).round().max(1.0) as usize
}

/// How far a stylize output pixel reaches into its neighbourhood (for tiled rendering).
pub fn stylize_reach_px(params: &StylizeParams, ctx: &RenderContext) -> usize {
    let mut reach = 0;
    if params.sketch > 0.0 {
        reach += 2 * sketch_radius_px(params, ctx); // two box passes
    }
    if params.glow > 0.0 {
        reach += 3 * glow_radius_px(params, ctx); // three box passes
    }
    reach
}

pub fn apply_stylize(img: &mut RgbImageF32, params: &StylizeParams, ctx: &RenderContext) {
    if !params.enabled {
        return;
    }
    let (w, h) = (img.width, img.height);

    // 1. Sketch: colour-dodge of the luminance by its blurred inverse ("pencil sketch").
    if params.sketch > 0.0 {
        let r = sketch_radius_px(params, ctx);
        let lum = img.compute_luminance();
        let inverse: Vec<f32> = lum.par_iter().map(|&l| 1.0 - l).collect();
        let blurred = box_filter(&box_filter(&inverse, w, h, r), w, h, r);
        let amount = params.sketch;
        let color_keep = params.sketch_color;

        img.r
            .par_iter_mut()
            .zip(img.g.par_iter_mut())
            .zip(img.b.par_iter_mut())
            .enumerate()
            .for_each(|(i, ((pr, pg), pb))| {
                let dodge = (lum[i] / (1.0 - blurred[i]).max(1e-3)).clamp(0.0, 1.0);
                // Deepen the strokes so pencil lines read clearly.
                let line = dodge.powi(4);
                for c in [pr, pg, pb] {
                    // Colour sketch: washed-out colour multiplied by the pencil lines.
                    let colored = (0.35 + 0.65 * *c) * line;
                    let sketched = line + (colored - line) * color_keep;
                    *c += (sketched - *c) * amount;
                }
            });
    }

    // 2. Glow: screen-blend a wide blur of the image over itself.
    if params.glow > 0.0 {
        let r = glow_radius_px(params, ctx);
        let amount = params.glow;
        for channel in [&mut img.r, &mut img.g, &mut img.b] {
            let blurred = box_filter(&box_filter(&box_filter(channel, w, h, r), w, h, r), w, h, r);
            channel.par_iter_mut().zip(blurred.par_iter()).for_each(|(v, &bl)| {
                let glow = bl * bl * amount; // emphasise bright areas
                *v = 1.0 - (1.0 - *v) * (1.0 - glow);
            });
        }
    }

    // 3. Per-pixel tone effects.
    let posterize_levels = if params.posterize > 0.0 {
        Some((24.0 - params.posterize * 21.0).round().max(2.0))
    } else {
        None
    };
    let solarize = params.solarize;
    let invert = params.invert;
    if posterize_levels.is_none() && solarize == 0.0 && invert == 0.0 {
        return;
    }
    for channel in [&mut img.r, &mut img.g, &mut img.b] {
        channel.par_iter_mut().for_each(|v| {
            let mut x = *v;
            if let Some(levels) = posterize_levels {
                let steps = levels - 1.0;
                x = (x * steps).round() / steps;
            }
            if solarize > 0.0 {
                // Sabattier: tones above mid-grey are reversed.
                let sol = if x > 0.5 { 2.0 * (1.0 - x) } else { 2.0 * x };
                x += (sol - x) * solarize;
            }
            if invert > 0.0 {
                x += (1.0 - 2.0 * x) * invert;
            }
            *v = x.clamp(0.0, 1.0);
        });
    }
}
