use crate::filter::guided_filter;
use crate::image_buffer::RgbImageF32;
use crate::params::NoiseParams;
use crate::pipeline::RenderContext;
use rayon::prelude::*;

/// Denoise filter radius: `amount` maps to 0..5 reference pixels, scaled to the full image.
pub fn denoise_radius_px(params: &NoiseParams, ctx: &RenderContext) -> usize {
    (params.amount * 5.0 * ctx.scale()).round().max(1.0) as usize
}

pub fn apply_denoise(img: &mut RgbImageF32, params: &NoiseParams, ctx: &RenderContext) {
    if !params.enabled || params.suppression == 0.0 {
        return;
    }

    let y = img.compute_luminance();
    let width = img.width;
    let height = img.height;

    let radius = denoise_radius_px(params, ctx);
    let eps = (params.suppression * 0.05).max(0.001);

    let smooth_y = guided_filter(&y, &y, width, height, radius, eps * eps);
    let strength = params.suppression.clamp(0.0, 1.0);

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .enumerate()
        .for_each(|(idx, ((r, g), b))| {
            let orig_y = y[idx].max(0.0001);
            let target_y = (1.0 - strength) * orig_y + strength * smooth_y[idx];
            let gain = (target_y / orig_y).clamp(0.2, 2.0);

            *r = (*r * gain).clamp(0.0, 1.0);
            *g = (*g * gain).clamp(0.0, 1.0);
            *b = (*b * gain).clamp(0.0, 1.0);
        });
}
