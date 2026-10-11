use crate::color::{hsv_to_rgb, rgb_to_hsv};
use crate::image_buffer::RgbImageF32;
use crate::params::ColorParams;
use rayon::prelude::*;

pub fn apply_color_adjust(img: &mut RgbImageF32, params: &ColorParams) {
    if !params.enabled {
        return;
    }

    if params.adaptive_saturation == 0.0
        && params.saturation == 0.0
        && params.saturation_boost == 1.0
        && params.hue == 0.0
    {
        return;
    }

    let adapt_sat = params.adaptive_saturation * 0.7;
    let sat_boost = params.saturation_boost;
    let global_sat = params.saturation;
    let hue_shift = params.hue / 360.0;

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .for_each(|((r, g), b)| {
            let (mut h, mut s, v) = rgb_to_hsv(*r, *g, *b);

            // 1. Hue shift
            if hue_shift != 0.0 {
                h = (h + hue_shift) % 1.0;
                if h < 0.0 {
                    h += 1.0;
                }
            }

            // 2. Adaptive saturation
            if adapt_sat > 0.0 {
                // Boost muted colors, protect already saturated ones
                let headroom = (1.0 - s).clamp(0.0, 1.0);
                s += headroom * adapt_sat * sat_boost;
            } else if adapt_sat < 0.0 {
                // Desaturate vibrant colors
                s += s * adapt_sat * 0.5;
            }

            // 3. Global saturation
            if global_sat > 0.0 {
                s = s + (1.0 - s) * global_sat;
            } else if global_sat < 0.0 {
                s *= 1.0 + global_sat;
            }

            s = s.clamp(0.0, 1.0);

            let (nr, ng, nb) = hsv_to_rgb(h, s, v);
            *r = nr.clamp(0.0, 1.0);
            *g = ng.clamp(0.0, 1.0);
            *b = nb.clamp(0.0, 1.0);
        });
}
