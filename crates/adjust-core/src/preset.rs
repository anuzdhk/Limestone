use crate::params::{
    AdaptiveExposureParams, AdjustParams, BasicExposureParams, ColorParams, CurveParams,
    CurvePoint, DetailsParams, FinishingParams, GrainParams, NoiseParams, VignetteParams,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub category: String,
    pub description: String,
    pub params: AdjustParams,
}

impl Preset {
    pub fn new(name: &str, category: &str, description: &str, params: AdjustParams) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            params,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

pub fn get_factory_presets() -> Vec<Preset> {
    vec![
        // 1. Spicify (Signature Topaz Adjust 4 look)
        Preset {
            name: "Spicify".to_string(),
            category: "HDR & Vibrant".to_string(),
            description: "Signature punchy look with enhanced dynamic range, high micro-contrast, and vibrant colors.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.0,
                    contrast: 0.12,
                    boost_blacks: 0.08,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 1.25,
                    regions: 12,
                    contrast: 0.18,
                    brightness: 0.04,
                    protect_highlights: 0.35,
                    protect_shadows: 0.25,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 1.40,
                    boost: 1.20,
                    threshold: 0.08,
                    radius: 1.2,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.65,
                    color_regions: 10,
                    saturation: 0.10,
                    saturation_boost: 1.15,
                    hue: 0.0,
                },
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.15,
                    amount: 0.40,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.05,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.15,
                        size: 0.60,
                        transition: 0.50,
                        roundness: 0.0,
                    },
                    grain: GrainParams::default(),
                },
            },
        },

        // 2. Dramatic
        Preset {
            name: "Dramatic".to_string(),
            category: "HDR & Stylized".to_string(),
            description: "Cinematic, gritty contrast with extreme micro-details and desaturated tones.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: -0.05,
                    contrast: 0.25,
                    boost_blacks: 0.20,
                    boost_whites: 0.10,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.95,
                    regions: 16,
                    contrast: 0.30,
                    brightness: -0.05,
                    protect_highlights: 0.40,
                    protect_shadows: 0.30,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 2.10,
                    boost: 1.50,
                    threshold: 0.06,
                    radius: 1.5,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: -0.20,
                    color_regions: 10,
                    saturation: -0.35,
                    saturation_boost: 0.80,
                    hue: 0.0,
                },
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.10,
                    amount: 0.30,
                },
                curve: CurveParams {
                    enabled: true,
                    points: vec![
                        CurvePoint { x: 0.0, y: 0.0 },
                        CurvePoint { x: 0.30, y: 0.22 },
                        CurvePoint { x: 0.70, y: 0.78 },
                        CurvePoint { x: 1.0, y: 1.0 },
                    ],
                },
                finishing: FinishingParams {
                    enabled: true,
                    warmth: -0.08,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.30,
                        size: 0.50,
                        transition: 0.40,
                        roundness: 0.2,
                    },
                    grain: GrainParams {
                        amount: 0.15,
                        size: 1.0,
                    },
                },
            },
        },

        // 3. Photo Pop
        Preset {
            name: "Photo Pop".to_string(),
            category: "Exposure & Detail".to_string(),
            description: "Clean, natural enhancement bringing clarity, brightness, and color vibrance without looking overcooked.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.02,
                    contrast: 0.08,
                    boost_blacks: 0.05,
                    boost_whites: 0.03,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.45,
                    regions: 8,
                    contrast: 0.08,
                    brightness: 0.02,
                    protect_highlights: 0.20,
                    protect_shadows: 0.15,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 0.65,
                    boost: 1.05,
                    threshold: 0.12,
                    radius: 1.0,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.35,
                    color_regions: 10,
                    saturation: 0.05,
                    saturation_boost: 1.10,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 4. Exposure Correction
        Preset {
            name: "Exposure Correction".to_string(),
            category: "Exposure & Detail".to_string(),
            description: "Balanced dynamic range recovery for underexposed shadows and harsh highlights.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 1.10,
                    regions: 6,
                    contrast: 0.0,
                    brightness: 0.0,
                    protect_highlights: 0.50,
                    protect_shadows: 0.45,
                },
                details: DetailsParams::default(),
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.15,
                    color_regions: 10,
                    saturation: 0.0,
                    saturation_boost: 1.0,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 5. Detail Medium
        Preset {
            name: "Detail Medium".to_string(),
            category: "Exposure & Detail".to_string(),
            description: "Moderate texture and edge definition enhancement.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams::default(),
                details: DetailsParams {
                    enabled: true,
                    strength: 1.20,
                    boost: 1.10,
                    threshold: 0.08,
                    radius: 1.0,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 6. Detail Strong
        Preset {
            name: "Detail Strong".to_string(),
            category: "Exposure & Detail".to_string(),
            description: "Aggressive micro-contrast for architecture, machinery, and complex textures.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams::default(),
                details: DetailsParams {
                    enabled: true,
                    strength: 2.20,
                    boost: 1.40,
                    threshold: 0.05,
                    radius: 1.2,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.20,
                    amount: 0.40,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 7. Painting
        Preset {
            name: "Painting".to_string(),
            category: "Stylized & Creative".to_string(),
            description: "Stylized illustrative look with broad edge halos and saturated oil-painting tones.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.05,
                    contrast: 0.15,
                    boost_blacks: 0.0,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 1.40,
                    regions: 18,
                    contrast: 0.20,
                    brightness: 0.05,
                    protect_highlights: 0.30,
                    protect_shadows: 0.30,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 2.50,
                    boost: 1.80,
                    threshold: 0.04,
                    radius: 2.8,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.85,
                    color_regions: 12,
                    saturation: 0.25,
                    saturation_boost: 1.30,
                    hue: 0.0,
                },
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.30,
                    amount: 0.60,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.10,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.20,
                        size: 0.55,
                        transition: 0.45,
                        roundness: 0.3,
                    },
                    grain: GrainParams::default(),
                },
            },
        },

        // 8. French Countryside
        Preset {
            name: "French Countryside".to_string(),
            category: "Stylized & Creative".to_string(),
            description: "Warm golden atmospheric tone with deep shadows and soft highlights.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.0,
                    contrast: 0.10,
                    boost_blacks: 0.10,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.90,
                    regions: 10,
                    contrast: 0.15,
                    brightness: 0.0,
                    protect_highlights: 0.40,
                    protect_shadows: 0.20,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 1.10,
                    boost: 1.10,
                    threshold: 0.09,
                    radius: 1.2,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.40,
                    color_regions: 10,
                    saturation: 0.05,
                    saturation_boost: 1.10,
                    hue: 5.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.35,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.18,
                        size: 0.60,
                        transition: 0.50,
                        roundness: 0.1,
                    },
                    grain: GrainParams {
                        amount: 0.10,
                        size: 1.2,
                    },
                },
            },
        },

        // 9. Portrait Smooth
        Preset {
            name: "Portrait Smooth".to_string(),
            category: "Portrait & Beauty".to_string(),
            description: "Gentle skin smoothing with noise suppression while retaining hair and eye clarity.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.04,
                    contrast: -0.05,
                    boost_blacks: 0.0,
                    boost_whites: 0.02,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.30,
                    regions: 8,
                    contrast: -0.05,
                    brightness: 0.03,
                    protect_highlights: 0.30,
                    protect_shadows: 0.20,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: -0.35, // Negative strength softens micro-texture
                    boost: 0.80,
                    threshold: 0.15,
                    radius: 1.5,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.20,
                    color_regions: 10,
                    saturation: -0.05,
                    saturation_boost: 0.95,
                    hue: 0.0,
                },
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.45,
                    amount: 0.60,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.08,
                    transparency: 0.0,
                    vignette: VignetteParams::default(),
                    grain: GrainParams::default(),
                },
            },
        },

        // 10. Neutral / Reset
        Preset {
            name: "Neutral".to_string(),
            category: "Exposure & Detail".to_string(),
            description: "Unmodified baseline with all adjustments at default settings.".to_string(),
            params: AdjustParams::default(),
        },
    ]
}
