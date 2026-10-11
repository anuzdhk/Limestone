use crate::color::hsv_to_rgb;
use crate::image_buffer::RgbImageF32;
use crate::params::ToneParams;
use rayon::prelude::*;

/// Black & white conversion followed by split toning.
///
/// Toning re-colours each pixel toward a tint while preserving its luminance, so a
/// sepia or selenium tone never brightens or darkens the image.
pub fn apply_tone(img: &mut RgbImageF32, params: &ToneParams) {
    if !params.enabled
        || (params.monochrome == 0.0
            && params.shadow_strength == 0.0
            && params.highlight_strength == 0.0)
    {
        return;
    }

    let mono = params.monochrome;
    let [mr, mg, mb] = params.mono_mix;
    let mix_sum = (mr + mg + mb).abs().max(1e-3);
    let (mr, mg, mb) = (mr / mix_sum, mg / mix_sum, mb / mix_sum);

    let shadow_tint = tint(params.shadow_hue);
    let highlight_tint = tint(params.highlight_hue);
    let (s_str, h_str) = (params.shadow_strength, params.highlight_strength);
    // Balance moves the shadow/highlight crossover point.
    let pivot = (0.5 + params.balance * 0.35).clamp(0.1, 0.9);

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .for_each(|((r, g), b)| {
            // 1. Monochrome (channel-mixer B&W)
            if mono > 0.0 {
                let bw = (*r * mr + *g * mg + *b * mb).clamp(0.0, 1.0);
                *r += (bw - *r) * mono;
                *g += (bw - *g) * mono;
                *b += (bw - *b) * mono;
            }

            // 2. Split toning
            // Shadow and highlight weights sum to 1, so equal hues give an even tone.
            let lum = luminance(*r, *g, *b);
            let t = smoothstep(pivot - 0.5, pivot + 0.5, lum);
            let shadow_w = (1.0 - t) * s_str;
            let highlight_w = t * h_str;
            for (tint, w) in [(shadow_tint, shadow_w), (highlight_tint, highlight_w)] {
                if w > 0.0 {
                    // Luminance-preserving colourised target
                    let target = [tint[0] * lum, tint[1] * lum, tint[2] * lum];
                    *r += (target[0] - *r) * w;
                    *g += (target[1] - *g) * w;
                    *b += (target[2] - *b) * w;
                }
            }

            *r = r.clamp(0.0, 1.0);
            *g = g.clamp(0.0, 1.0);
            *b = b.clamp(0.0, 1.0);
        });
}

/// A saturated hue scaled so that its luminance is 1.0.
fn tint(hue_deg: f32) -> [f32; 3] {
    let (r, g, b) = hsv_to_rgb((hue_deg / 360.0).rem_euclid(1.0), 0.65, 1.0);
    let lum = luminance(r, g, b).max(1e-3);
    [r / lum, g / lum, b / lum]
}

#[inline(always)]
fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Smooth 0..1 ramp as `x` goes from `edge0` to `edge1`.
#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
