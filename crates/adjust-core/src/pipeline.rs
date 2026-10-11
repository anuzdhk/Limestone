use crate::image_buffer::RgbImageF32;
use crate::params::AdjustParams;
use crate::stages::{
    adaptive_exposure::{adaptive_radius_px, apply_adaptive_exposure},
    basic_exposure::apply_basic_exposure,
    color_adjust::apply_color_adjust,
    curve::apply_curve,
    denoise::{apply_denoise, denoise_radius_px},
    details::{apply_detail_layer, apply_details, detail_radius_px, extract_detail_layer},
    finishing::apply_finishing,
};

/// Longest image side, in pixels, at which spatial parameters (detail radius, denoise
/// radius, grain size) are expressed. Radii scale with the real image size so a 960px
/// preview and a 6000px export produce the same look.
pub const REFERENCE_DIM: f32 = 1000.0;

/// Where the pixels being processed sit inside the full image.
///
/// Position-dependent effects (vignette, border, grain) and resolution-dependent radii
/// are computed from the full image, so a cropped tile renders exactly like the same
/// area of a full render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderContext {
    pub full_width: usize,
    pub full_height: usize,
    pub offset_x: usize,
    pub offset_y: usize,
}

impl RenderContext {
    /// Context for processing a whole image.
    pub fn whole(img: &RgbImageF32) -> Self {
        Self {
            full_width: img.width,
            full_height: img.height,
            offset_x: 0,
            offset_y: 0,
        }
    }

    /// Pixels per reference pixel for this full-image size.
    pub fn scale(&self) -> f32 {
        self.full_width.max(self.full_height).max(1) as f32 / REFERENCE_DIM
    }
}

/// Executes the full adjustment pipeline on a whole image.
pub fn process_image(input: &RgbImageF32, params: &AdjustParams) -> RgbImageF32 {
    process_with_context(input, params, &RenderContext::whole(input))
}

/// Processes the `w x h` region at (`x0`, `y0`) of `full` and returns just that region.
///
/// The region is expanded by the pipeline's filter support before processing so
/// edge-aware filters see the same neighbourhood as in a full render; the result
/// matches the corresponding crop of `process_image(full, params)`.
pub fn process_region(
    full: &RgbImageF32,
    params: &AdjustParams,
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
) -> RgbImageF32 {
    let params = params.sanitized();
    let whole = RenderContext::whole(full);
    let margin = filter_margin(&params, &whole);

    let px0 = x0.saturating_sub(margin);
    let py0 = y0.saturating_sub(margin);
    let px1 = (x0 + w + margin).min(full.width);
    let py1 = (y0 + h + margin).min(full.height);

    let padded = full.crop(px0, py0, px1 - px0, py1 - py0);
    let ctx = RenderContext {
        offset_x: px0,
        offset_y: py0,
        ..whole
    };
    let processed = process_with_context(&padded, &params, &ctx);
    processed.crop(x0 - px0, y0 - py0, w, h)
}

/// How far (in pixels) a single output pixel can depend on its neighbours.
/// Each guided filter reaches `2 * radius` (two chained box filters), and stages chain.
fn filter_margin(params: &AdjustParams, ctx: &RenderContext) -> usize {
    let mut reach = 0;
    if params.adaptive_exposure.enabled {
        reach += 2 * adaptive_radius_px(&params.adaptive_exposure, ctx);
    }
    if params.details.enabled {
        // Independent extraction and dependent extraction never both run.
        reach += 2 * detail_radius_px(&params.details, ctx);
    }
    if params.noise.enabled {
        reach += 2 * denoise_radius_px(&params.noise, ctx);
    }
    reach + 2
}

/// Executes the pipeline on `input`, which sits inside a larger image described by `ctx`.
pub fn process_with_context(
    input: &RgbImageF32,
    params: &AdjustParams,
    ctx: &RenderContext,
) -> RgbImageF32 {
    let params = &params.sanitized();
    let mut working = input.clone();
    if input.width == 0 || input.height == 0 {
        return working;
    }

    // 1. Independent Detail Extraction:
    // If enabled, extract high frequencies from input luminance BEFORE tone-mapping
    let has_detail_op =
        params.details.enabled && (params.details.strength != 0.0 || params.details.sharpen != 0.0);
    let pre_extracted_detail = if has_detail_op && params.details.process_independently {
        let y = input.compute_luminance();
        Some(extract_detail_layer(
            &y,
            input.width,
            input.height,
            detail_radius_px(&params.details, ctx),
        ))
    } else {
        None
    };

    // 2. Basic Exposure Stage
    apply_basic_exposure(&mut working, &params.basic_exposure);

    // 3. Adaptive Exposure (Local Tone Mapping) Stage
    apply_adaptive_exposure(&mut working, &params.adaptive_exposure, ctx);

    // 4. Details Stage (Recombination or Dependent Extraction)
    if let Some(ref detail) = pre_extracted_detail {
        apply_detail_layer(&mut working, detail, &params.details);
    } else if has_detail_op {
        apply_details(&mut working, &params.details, ctx);
    }

    // 5. Color & Adaptive Saturation Stage
    apply_color_adjust(&mut working, &params.color);

    // 6. Noise Suppression Stage
    apply_denoise(&mut working, &params.noise, ctx);

    // 7. Curve Tool Stage
    apply_curve(&mut working, &params.curve);

    // 8. Finishing Touches (Vignette, Grain, Warmth, and Master Transparency)
    apply_finishing(&mut working, input, &params.finishing, ctx);

    working
}
