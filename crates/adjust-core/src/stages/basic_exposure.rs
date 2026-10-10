use crate::image_buffer::RgbImageF32;
use crate::params::BasicExposureParams;
use rayon::prelude::*;

pub fn apply_basic_exposure(img: &mut RgbImageF32, params: &BasicExposureParams) {
    if !params.enabled {
        return;
    }

    if params.brightness == 0.0
        && params.contrast == 0.0
        && params.boost_blacks == 0.0
        && params.boost_whites == 0.0
    {
        return;
    }

    let brightness = params.brightness * 0.5;
    let contrast_factor = if params.contrast >= 0.0 {
        1.0 + params.contrast * 1.5
    } else {
        1.0 + params.contrast * 0.7
    };
    let boost_blacks = params.boost_blacks;
    let boost_whites = params.boost_whites;

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .for_each(|((r, g), b)| {
            *r = process_channel(*r, brightness, contrast_factor, boost_blacks, boost_whites);
            *g = process_channel(*g, brightness, contrast_factor, boost_blacks, boost_whites);
            *b = process_channel(*b, brightness, contrast_factor, boost_blacks, boost_whites);
        });
}

#[inline(always)]
fn process_channel(
    val: f32,
    brightness: f32,
    contrast_factor: f32,
    boost_blacks: f32,
    boost_whites: f32,
) -> f32 {
    // 1. Brightness offset
    let mut v = val + brightness;

    // 2. Contrast around mid-gray 0.5
    v = (v - 0.5) * contrast_factor + 0.5;

    // 3. Boost Blacks: pull lower tones down
    if boost_blacks > 0.0 && v < 0.5 {
        let shadow_weight = (1.0 - (v / 0.5)).clamp(0.0, 1.0);
        v -= boost_blacks * 0.25 * shadow_weight;
    }

    // 4. Boost Whites: push upper highlights up
    if boost_whites > 0.0 && v > 0.5 {
        let highlight_weight = ((v - 0.5) / 0.5).clamp(0.0, 1.0);
        v += boost_whites * 0.25 * highlight_weight;
    }

    v.clamp(0.0, 1.0)
}
