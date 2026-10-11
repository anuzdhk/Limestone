use crate::filter::guided_filter;
use crate::image_buffer::RgbImageF32;
use crate::params::AdaptiveExposureParams;
use crate::pipeline::RenderContext;
use rayon::prelude::*;

/// Guided-filter radius for the local tone mapping, proportional to the full image size
/// (fewer regions -> larger, smoother illumination zones).
pub fn adaptive_radius_px(params: &AdaptiveExposureParams, ctx: &RenderContext) -> usize {
    let max_dim = ctx.full_width.max(ctx.full_height) as f32;
    let scale_factor = (32 - params.regions.clamp(1, 30)) as f32 / 10.0;
    ((max_dim / 30.0) * scale_factor).round().max(1.0) as usize
}

pub fn apply_adaptive_exposure(
    img: &mut RgbImageF32,
    params: &AdaptiveExposureParams,
    ctx: &RenderContext,
) {
    if !params.enabled {
        return;
    }

    if params.adaptive_exposure == 0.0
        && params.contrast == 0.0
        && params.brightness == 0.0
        && params.protect_highlights == 0.0
        && params.protect_shadows == 0.0
    {
        return;
    }

    let width = img.width;
    let height = img.height;
    let size = width * height;

    // 1. Compute perceptual luminance Y
    let y = img.compute_luminance();

    // 2. Map regions parameter (1..30) to a spatial radius relative to the full image
    let radius = adaptive_radius_px(params, ctx);

    // Edge-preserving guided filter on luminance Y to extract base illumination
    let eps = 0.02 * 0.02;
    let base_lum = guided_filter(&y, &y, width, height, radius, eps);

    let strength = params.adaptive_exposure;
    let contrast_factor = if params.contrast >= 0.0 {
        1.0 + params.contrast * 0.8
    } else {
        1.0 + params.contrast * 0.5
    };
    let brightness_offset = params.brightness * 0.25;
    let prot_high = params.protect_highlights;
    let prot_shad = params.protect_shadows;

    let mut y_mapped = vec![0.0; size];

    y_mapped
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, out_lum)| {
            let orig_y = y[idx].clamp(0.0001, 1.0);
            let base = base_lum[idx].clamp(0.0001, 1.0);

            // Local Gamma Tone Mapping:
            // Shadows (base < 0.5) get gamma < 1 (lifted)
            // Highlights (base > 0.5) get gamma > 1 (compressed)
            let diff = (0.5 - base) * strength * 0.7;
            let gamma = (-diff).exp().clamp(0.25, 4.0);
            let mut mapped = orig_y.powf(gamma);

            // Apply regional contrast
            if params.contrast != 0.0 {
                mapped = ((mapped - 0.5) * contrast_factor + 0.5).clamp(0.0, 1.0);
            }

            // Apply regional brightness
            if brightness_offset != 0.0 {
                mapped = (mapped + brightness_offset).clamp(0.0, 1.0);
            }

            // Protect Highlights (smooth rational shoulder for > 0.65)
            if prot_high > 0.0 && mapped > 0.65 {
                let excess = (mapped - 0.65) / 0.35;
                mapped = 0.65 + 0.35 * (excess / (1.0 + prot_high * excess * 1.6));
            }

            // Protect Shadows (lift shadows smoothly while anchoring black point at 0.0)
            if prot_shad > 0.0 && mapped < 0.35 {
                let weight = mapped * (1.0 - mapped / 0.35);
                mapped += prot_shad * 0.35 * weight;
            }

            *out_lum = mapped.clamp(0.0, 1.0);
        });

    // 5. Transfer luminance gain to RGB channels preserving chromatic ratios
    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .enumerate()
        .for_each(|(idx, ((r, g), b))| {
            let orig_y = y[idx].max(0.0001);
            let gain = (y_mapped[idx] / orig_y).clamp(0.0, 5.0);
            *r = (*r * gain).clamp(0.0, 1.0);
            *g = (*g * gain).clamp(0.0, 1.0);
            *b = (*b * gain).clamp(0.0, 1.0);
        });
}
