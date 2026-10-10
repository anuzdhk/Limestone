use crate::image_buffer::RgbImageF32;
use crate::params::AdjustParams;
use crate::stages::{
    adaptive_exposure::apply_adaptive_exposure,
    basic_exposure::apply_basic_exposure,
    color_adjust::apply_color_adjust,
    curve::apply_curve,
    denoise::apply_denoise,
    details::{apply_detail_layer, apply_details, extract_detail_layer},
    finishing::apply_finishing,
};

/// Executes the full Topaz Adjust 4.0 DSP pipeline on an image.
pub fn process_image(input: &RgbImageF32, params: &AdjustParams) -> RgbImageF32 {
    let mut working = input.clone();

    // 1. Independent Detail Extraction:
    // If enabled, extract high frequencies from input luminance BEFORE tone-mapping
    let has_detail_op = params.details.enabled && (params.details.strength != 0.0 || params.details.sharpen != 0.0);
    let pre_extracted_detail = if has_detail_op && params.details.process_independently {
        let y = input.compute_luminance();
        Some(extract_detail_layer(&y, input.width, input.height, params.details.radius))
    } else {
        None
    };

    // 2. Basic Exposure Stage
    apply_basic_exposure(&mut working, &params.basic_exposure);

    // 3. Adaptive Exposure (Local Tone Mapping) Stage
    apply_adaptive_exposure(&mut working, &params.adaptive_exposure);

    // 4. Details Stage (Recombination or Dependent Extraction)
    if let Some(ref detail) = pre_extracted_detail {
        apply_detail_layer(&mut working, detail, &params.details);
    } else if has_detail_op {
        apply_details(&mut working, &params.details);
    }

    // 5. Color & Adaptive Saturation Stage
    apply_color_adjust(&mut working, &params.color);

    // 6. Noise Suppression Stage
    apply_denoise(&mut working, &params.noise);

    // 7. Curve Tool Stage
    apply_curve(&mut working, &params.curve);

    // 8. Finishing Touches (Vignette, Grain, Warmth, and Master Transparency)
    apply_finishing(&mut working, input, &params.finishing);

    working
}
