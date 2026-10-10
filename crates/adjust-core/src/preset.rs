use crate::params::{
    AdaptiveExposureParams, AdjustParams, BasicExposureParams, BorderParams, ColorParams,
    CurveParams, CurvePoint, DetailsParams, FinishingParams, GrainParams, NoiseParams,
    VignetteParams,
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
                    detail_boost: 1.20,
                    threshold: 0.08,
                    radius: 1.2,
                    sharpen: 0.20,
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
                    border: BorderParams::default(),
                },
            },
        },

        // 2. Dramatic
        Preset {
            name: "Dramatic".to_string(),
            category: "HDR Collection".to_string(),
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
                    detail_boost: 1.50,
                    threshold: 0.06,
                    radius: 1.5,
                    sharpen: 0.40,
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
                    border: BorderParams::default(),
                },
            },
        },

        // 3. Mild Contrast Pop (Shown selected in Topaz Adjust 4 starfish UI)
        Preset {
            name: "Mild Contrast Pop".to_string(),
            category: "Classic Collection".to_string(),
            description: "Subtle local contrast and exposure boost suitable for almost any scene.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.07,
                    regions: 5,
                    contrast: 0.56,
                    brightness: 0.0,
                    protect_highlights: 0.02,
                    protect_shadows: 0.02,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 1.00,
                    detail_boost: 1.00,
                    threshold: 0.15,
                    radius: 25.0,
                    sharpen: 1.00,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.0,
                    color_regions: 1,
                    saturation: 1.00,
                    saturation_boost: 1.00,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 4. Detail - Light
        Preset {
            name: "Detail - Light".to_string(),
            category: "Classic Collection".to_string(),
            description: "Gentle high-frequency texture enhancement without introducing noise.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams::default(),
                details: DetailsParams {
                    enabled: true,
                    strength: 0.50,
                    detail_boost: 1.00,
                    threshold: 0.12,
                    radius: 1.0,
                    sharpen: 0.30,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 5. Detail - Medium
        Preset {
            name: "Detail - Medium".to_string(),
            category: "Classic Collection".to_string(),
            description: "Noticeable boost in clarity, micro-texture, and fine edge delineation.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams::default(),
                details: DetailsParams {
                    enabled: true,
                    strength: 1.10,
                    detail_boost: 1.15,
                    threshold: 0.08,
                    radius: 1.2,
                    sharpen: 0.60,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 6. Photo Pop
        Preset {
            name: "Photo Pop".to_string(),
            category: "Classic Collection".to_string(),
            description: "Clean, natural enhancement bringing clarity, brightness, and color vibrance.".to_string(),
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
                    detail_boost: 1.05,
                    threshold: 0.12,
                    radius: 1.0,
                    sharpen: 0.30,
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

        // 7. Exposure Correction
        Preset {
            name: "Exposure Correction".to_string(),
            category: "Classic Collection".to_string(),
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

        // 8. Equalize
        Preset {
            name: "Equalize".to_string(),
            category: "Classic Collection".to_string(),
            description: "Uniform tonal redistribution across the entire dynamic spectrum.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 1.30,
                    regions: 12,
                    contrast: -0.10,
                    brightness: 0.0,
                    protect_highlights: 0.40,
                    protect_shadows: 0.40,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 0.40,
                    detail_boost: 1.0,
                    threshold: 0.10,
                    radius: 1.0,
                    sharpen: 0.20,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 9. Highlight Recovery
        Preset {
            name: "Highlight Recovery".to_string(),
            category: "Classic Collection".to_string(),
            description: "Aggressively restores texture and detail in blown-out sky and specular areas.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: -0.10,
                    contrast: 0.05,
                    boost_blacks: 0.0,
                    boost_whites: 0.0,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.85,
                    regions: 14,
                    contrast: 0.10,
                    brightness: -0.05,
                    protect_highlights: 0.80,
                    protect_shadows: 0.10,
                },
                details: DetailsParams::default(),
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 10. Shadow Recovery
        Preset {
            name: "Shadow Recovery".to_string(),
            category: "Classic Collection".to_string(),
            description: "Lifts blocked shadow regions while suppressing dark-area luminance noise.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.05,
                    contrast: -0.05,
                    boost_blacks: 0.0,
                    boost_whites: 0.0,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 1.20,
                    regions: 10,
                    contrast: -0.05,
                    brightness: 0.08,
                    protect_highlights: 0.10,
                    protect_shadows: 0.80,
                },
                details: DetailsParams::default(),
                color: ColorParams::default(),
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.35,
                    amount: 0.50,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 11. High Key I
        Preset {
            name: "High Key I".to_string(),
            category: "Classic Collection".to_string(),
            description: "Bright, airy, ethereal atmosphere with luminous whites and soft contrast.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.25,
                    contrast: -0.10,
                    boost_blacks: 0.0,
                    boost_whites: 0.20,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.60,
                    regions: 8,
                    contrast: -0.15,
                    brightness: 0.15,
                    protect_highlights: 0.10,
                    protect_shadows: 0.30,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: -0.20,
                    detail_boost: 0.90,
                    threshold: 0.15,
                    radius: 1.5,
                    sharpen: 0.10,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: -0.10,
                    color_regions: 10,
                    saturation: -0.15,
                    saturation_boost: 0.90,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 12. Low Key I
        Preset {
            name: "Low Key I".to_string(),
            category: "Classic Collection".to_string(),
            description: "Moody, shadow-dominant aesthetic with high drama and deep blacks.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: -0.20,
                    contrast: 0.20,
                    boost_blacks: 0.30,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.40,
                    regions: 12,
                    contrast: 0.25,
                    brightness: -0.15,
                    protect_highlights: 0.40,
                    protect_shadows: 0.10,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 0.80,
                    detail_boost: 1.20,
                    threshold: 0.08,
                    radius: 1.2,
                    sharpen: 0.50,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: -0.05,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.35,
                        size: 0.55,
                        transition: 0.45,
                        roundness: 0.2,
                    },
                    grain: GrainParams::default(),
                    border: BorderParams::default(),
                },
            },
        },

        // 13. Mild Color Pop
        Preset {
            name: "Mild Color Pop".to_string(),
            category: "Vibrant Collection".to_string(),
            description: "Invigorates flat chromas while keeping natural skintones protected.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.30,
                    regions: 8,
                    contrast: 0.05,
                    brightness: 0.0,
                    protect_highlights: 0.10,
                    protect_shadows: 0.10,
                },
                details: DetailsParams::default(),
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.45,
                    color_regions: 8,
                    saturation: 0.12,
                    saturation_boost: 1.20,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 14. Simple Pop
        Preset {
            name: "Simple Pop".to_string(),
            category: "Vibrant Collection".to_string(),
            description: "One-click universal punch for mobile photos and overcast days.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.03,
                    contrast: 0.10,
                    boost_blacks: 0.05,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.50,
                    regions: 8,
                    contrast: 0.12,
                    brightness: 0.02,
                    protect_highlights: 0.20,
                    protect_shadows: 0.15,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 0.80,
                    detail_boost: 1.10,
                    threshold: 0.10,
                    radius: 1.0,
                    sharpen: 0.40,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.35,
                    color_regions: 10,
                    saturation: 0.08,
                    saturation_boost: 1.10,
                    hue: 0.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 15. Warm Tone I
        Preset {
            name: "Warm Tone I".to_string(),
            category: "Toned Collection".to_string(),
            description: "Pleasing golden-hour warmth and softened contrast.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.40,
                    regions: 10,
                    contrast: 0.05,
                    brightness: 0.02,
                    protect_highlights: 0.15,
                    protect_shadows: 0.10,
                },
                details: DetailsParams::default(),
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.20,
                    color_regions: 10,
                    saturation: 0.05,
                    saturation_boost: 1.05,
                    hue: 4.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.25,
                    transparency: 0.0,
                    vignette: VignetteParams::default(),
                    grain: GrainParams::default(),
                    border: BorderParams::default(),
                },
            },
        },

        // 16. Cool Tone I
        Preset {
            name: "Cool Tone I".to_string(),
            category: "Toned Collection".to_string(),
            description: "Crisp Nordic coolness with clean whites and cyan-tinged shadows.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams::default(),
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.40,
                    regions: 10,
                    contrast: 0.10,
                    brightness: 0.0,
                    protect_highlights: 0.20,
                    protect_shadows: 0.15,
                },
                details: DetailsParams::default(),
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.10,
                    color_regions: 10,
                    saturation: -0.05,
                    saturation_boost: 1.0,
                    hue: -5.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::default(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: -0.25,
                    transparency: 0.0,
                    vignette: VignetteParams::default(),
                    grain: GrainParams::default(),
                    border: BorderParams::default(),
                },
            },
        },

        // 17. Soft Focus I
        Preset {
            name: "Soft Focus I".to_string(),
            category: "Stylized Collection".to_string(),
            description: "Dreamy romantic glow attenuating hard digital edges.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.06,
                    contrast: -0.10,
                    boost_blacks: 0.0,
                    boost_whites: 0.05,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.50,
                    regions: 8,
                    contrast: -0.10,
                    brightness: 0.05,
                    protect_highlights: 0.20,
                    protect_shadows: 0.20,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: -0.50,
                    detail_boost: 0.75,
                    threshold: 0.18,
                    radius: 2.0,
                    sharpen: 0.0,
                    process_independently: true,
                },
                color: ColorParams::default(),
                noise: NoiseParams {
                    enabled: true,
                    suppression: 0.30,
                    amount: 0.50,
                },
                curve: CurveParams::default(),
                finishing: FinishingParams::default(),
            },
        },

        // 18. Painting
        Preset {
            name: "Painting".to_string(),
            category: "Stylized Collection".to_string(),
            description: "Illustrative look with rich edge halos and saturated oil-painting tones.".to_string(),
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
                    detail_boost: 1.80,
                    threshold: 0.04,
                    radius: 2.8,
                    sharpen: 0.80,
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
                    border: BorderParams::default(),
                },
            },
        },

        // 19. French Countryside
        Preset {
            name: "French Countryside".to_string(),
            category: "Stylized Collection".to_string(),
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
                    detail_boost: 1.10,
                    threshold: 0.09,
                    radius: 1.2,
                    sharpen: 0.40,
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
                    border: BorderParams::default(),
                },
            },
        },

        // 20. Film Collection Look
        Preset {
            name: "Vintage 35mm".to_string(),
            category: "Film Collection".to_string(),
            description: "Classic analog color reproduction with subtle grain and lifted blacks.".to_string(),
            params: AdjustParams {
                basic_exposure: BasicExposureParams {
                    enabled: true,
                    brightness: 0.02,
                    contrast: 0.08,
                    boost_blacks: -0.05,
                    boost_whites: 0.02,
                },
                adaptive_exposure: AdaptiveExposureParams {
                    enabled: true,
                    adaptive_exposure: 0.35,
                    regions: 10,
                    contrast: 0.08,
                    brightness: 0.02,
                    protect_highlights: 0.20,
                    protect_shadows: 0.20,
                },
                details: DetailsParams {
                    enabled: true,
                    strength: 0.40,
                    detail_boost: 1.0,
                    threshold: 0.10,
                    radius: 1.0,
                    sharpen: 0.20,
                    process_independently: true,
                },
                color: ColorParams {
                    enabled: true,
                    adaptive_saturation: 0.15,
                    color_regions: 10,
                    saturation: -0.08,
                    saturation_boost: 0.95,
                    hue: 3.0,
                },
                noise: NoiseParams::default(),
                curve: CurveParams::matte_film(),
                finishing: FinishingParams {
                    enabled: true,
                    warmth: 0.12,
                    transparency: 0.0,
                    vignette: VignetteParams {
                        amount: -0.15,
                        size: 0.65,
                        transition: 0.50,
                        roundness: 0.0,
                    },
                    grain: GrainParams {
                        amount: 0.25,
                        size: 1.2,
                    },
                    border: BorderParams::default(),
                },
            },
        },

        // 21. Neutral / Reset
        Preset {
            name: "Neutral".to_string(),
            category: "Classic Collection".to_string(),
            description: "Unmodified baseline with all adjustments at default settings.".to_string(),
            params: AdjustParams::default(),
        },
    ]
}
