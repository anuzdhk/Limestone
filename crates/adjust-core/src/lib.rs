pub mod color;
pub mod filter;
pub mod image_buffer;
pub mod params;
pub mod pipeline;
pub mod preset;
pub mod stages;

// Re-exports for convenient usage
pub use image_buffer::RgbImageF32;
pub use params::{
    AdaptiveExposureParams, AdjustParams, BasicExposureParams, ColorParams, CurveParams,
    CurvePoint, DetailsParams, FinishingParams, GrainParams, NoiseParams, VignetteParams,
};
pub use pipeline::process_image;
pub use preset::{get_factory_presets, Preset};
pub use stages::curve::{compute_histogram, generate_curve_lut, HistogramData};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_image_neutral() {
        let img = RgbImageF32::new(100, 100);
        let params = AdjustParams::default();
        let out = process_image(&img, &params);
        assert_eq!(out.width, 100);
        assert_eq!(out.height, 100);
    }

    #[test]
    fn test_factory_presets_exist() {
        let presets = get_factory_presets();
        assert!(presets.iter().any(|p| p.name == "Spicify"));
        assert!(presets.iter().any(|p| p.name == "Dramatic"));
        assert!(presets.iter().any(|p| p.name == "Photo Pop"));
    }

    #[test]
    fn test_preset_json_roundtrip() {
        let presets = get_factory_presets();
        let spicify = &presets[0];
        let json = spicify.to_json().expect("Serialization failed");
        let restored = Preset::from_json(&json).expect("Deserialization failed");
        assert_eq!(restored.name, "Spicify");
        assert_eq!(restored.params, spicify.params);
    }
}
