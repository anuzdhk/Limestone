use crate::filter::guided_filter;
use crate::image_buffer::RgbImageF32;
use crate::params::DetailsParams;
use crate::pipeline::RenderContext;
use rayon::prelude::*;

/// Detail extraction radius in pixels: `params.radius` is in reference pixels
/// (see [`crate::pipeline::REFERENCE_DIM`]) and scales with the full image size.
pub fn detail_radius_px(params: &DetailsParams, ctx: &RenderContext) -> usize {
    (params.radius * ctx.scale()).round().max(1.0) as usize
}

pub fn extract_detail_layer(y: &[f32], width: usize, height: usize, radius_px: usize) -> Vec<f32> {
    let r = radius_px.max(1);
    let eps = 0.01 * 0.01;
    let smooth = guided_filter(y, y, width, height, r, eps);

    let mut detail = vec![0.0; width * height];
    detail
        .par_iter_mut()
        .zip(y.par_iter())
        .zip(smooth.par_iter())
        .for_each(|((d, &orig), &sm)| {
            *d = orig - sm;
        });

    detail
}

pub fn apply_detail_layer(
    img: &mut RgbImageF32,
    detail: &[f32],
    params: &DetailsParams,
) {
    if !params.enabled {
        return;
    }

    let strength = params.strength * 1.5;
    let boost = params.detail_boost;
    let threshold = (params.threshold * 0.15).max(0.0005);
    let sharpen = params.sharpen;

    if strength == 0.0 && sharpen == 0.0 {
        return;
    }

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .zip(detail.par_iter())
        .for_each(|(((r, g), b), &d)| {
            let abs_d = d.abs();

            let weight = if strength >= 0.0 {
                // Sharpening mode: suppress noise in flat areas, pass real textures
                if abs_d < threshold {
                    let t = abs_d / threshold;
                    t * t * (3.0 - 2.0 * t) // Smoothstep
                } else {
                    1.0
                }
            } else {
                // Smoothing mode: attenuate minor textures (skin blemishes), protect sharp edges
                if abs_d < threshold {
                    1.0
                } else if abs_d < threshold * 2.5 {
                    let t = (abs_d - threshold) / (threshold * 1.5);
                    1.0 - t * t * (3.0 - 2.0 * t)
                } else {
                    0.0
                }
            };

            // Detail boost amplifies low-amplitude textures
            let amp = if abs_d < 0.15 {
                1.0 + (boost - 1.0) * (1.0 - (abs_d / 0.15))
            } else {
                1.0
            };

            // Combine detail layer with unsharp mask sharpen boost
            let delta = d * (strength * amp * weight + sharpen * 0.8);

            *r = (*r + delta).clamp(0.0, 1.0);
            *g = (*g + delta).clamp(0.0, 1.0);
            *b = (*b + delta).clamp(0.0, 1.0);
        });
}

pub fn apply_details(img: &mut RgbImageF32, params: &DetailsParams, ctx: &RenderContext) {
    if !params.enabled || (params.strength == 0.0 && params.sharpen == 0.0) {
        return;
    }

    let y = img.compute_luminance();
    let detail = extract_detail_layer(&y, img.width, img.height, detail_radius_px(params, ctx));
    apply_detail_layer(img, &detail, params);
}
