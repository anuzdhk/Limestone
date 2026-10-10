use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BasicExposureParams {
    pub enabled: bool,
    pub brightness: f32,    // -1.0 .. 1.0, default 0.0
    pub contrast: f32,      // -1.0 .. 1.0, default 0.0
    pub boost_blacks: f32,  //  0.0 .. 1.0, default 0.0
    pub boost_whites: f32,  //  0.0 .. 1.0, default 0.0
}

impl Default for BasicExposureParams {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 0.0,
            contrast: 0.0,
            boost_blacks: 0.0,
            boost_whites: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveExposureParams {
    pub enabled: bool,
    pub adaptive_exposure: f32,   // 0.0 .. 3.0, default 0.0
    pub regions: u32,             // 1 .. 30, default 10
    pub contrast: f32,            // -1.0 .. 1.0, default 0.0
    pub brightness: f32,          // -1.0 .. 1.0, default 0.0
    pub protect_highlights: f32,  // 0.0 .. 1.0, default 0.0
    pub protect_shadows: f32,     // 0.0 .. 1.0, default 0.0
}

impl Default for AdaptiveExposureParams {
    fn default() -> Self {
        Self {
            enabled: true,
            adaptive_exposure: 0.0,
            regions: 10,
            contrast: 0.0,
            brightness: 0.0,
            protect_highlights: 0.0,
            protect_shadows: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetailsParams {
    pub enabled: bool,
    pub strength: f32,              // -1.0 .. 3.0, default 0.0 (negative softens/smooths)
    pub boost: f32,                 // 0.0 .. 3.0, default 1.0
    pub threshold: f32,             // 0.0 .. 1.0, default 0.10
    pub radius: f32,                // 0.5 .. 10.0, default 1.0
    pub process_independently: bool,// default true
}

impl Default for DetailsParams {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 0.0,
            boost: 1.0,
            threshold: 0.10,
            radius: 1.0,
            process_independently: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorParams {
    pub enabled: bool,
    pub adaptive_saturation: f32, // -1.0 .. 2.0, default 0.0
    pub color_regions: u32,       // 1 .. 30, default 10
    pub saturation: f32,          // -1.0 .. 1.0, default 0.0
    pub saturation_boost: f32,    // 0.0 .. 2.0, default 1.0
    pub hue: f32,                 // -180.0 .. 180.0, default 0.0
}

impl Default for ColorParams {
    fn default() -> Self {
        Self {
            enabled: true,
            adaptive_saturation: 0.0,
            color_regions: 10,
            saturation: 0.0,
            saturation_boost: 1.0,
            hue: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoiseParams {
    pub enabled: bool,
    pub suppression: f32, // 0.0 .. 1.0, default 0.0
    pub amount: f32,      // 0.0 .. 1.0, default 0.5
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            enabled: true,
            suppression: 0.0,
            amount: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurveParams {
    pub enabled: bool,
    pub points: Vec<CurvePoint>, // Control points for natural cubic spline
}

impl Default for CurveParams {
    fn default() -> Self {
        Self::linear()
    }
}

impl CurveParams {
    pub fn linear() -> Self {
        Self {
            enabled: false,
            points: vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 1.0, y: 1.0 }],
        }
    }

    pub fn medium_contrast() -> Self {
        Self {
            enabled: true,
            points: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 0.25, y: 0.18 },
                CurvePoint { x: 0.75, y: 0.82 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            enabled: true,
            points: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 0.25, y: 0.12 },
                CurvePoint { x: 0.75, y: 0.88 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
        }
    }

    pub fn lift_shadows() -> Self {
        Self {
            enabled: true,
            points: vec![
                CurvePoint { x: 0.0, y: 0.08 },
                CurvePoint { x: 0.35, y: 0.42 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
        }
    }

    pub fn matte_film() -> Self {
        Self {
            enabled: true,
            points: vec![
                CurvePoint { x: 0.0, y: 0.10 },
                CurvePoint { x: 0.30, y: 0.25 },
                CurvePoint { x: 0.70, y: 0.75 },
                CurvePoint { x: 1.0, y: 0.92 },
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VignetteParams {
    pub amount: f32,     // -1.0 .. 1.0, default 0.0 (negative = dark, positive = light)
    pub size: f32,       // 0.1 .. 1.0, default 0.5
    pub transition: f32, // 0.0 .. 1.0, default 0.5
    pub roundness: f32,  // 0.0 .. 1.0, default 0.0 (0 = rectangle, 1 = circle)
}

impl Default for VignetteParams {
    fn default() -> Self {
        Self {
            amount: 0.0,
            size: 0.5,
            transition: 0.5,
            roundness: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrainParams {
    pub amount: f32, // 0.0 .. 1.0, default 0.0
    pub size: f32,   // 0.5 .. 3.0, default 1.0
}

impl Default for GrainParams {
    fn default() -> Self {
        Self {
            amount: 0.0,
            size: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FinishingParams {
    pub enabled: bool,
    pub transparency: f32, // 0.0 .. 1.0, default 0.0 (0.0 = 100% effect, 1.0 = original image)
    pub warmth: f32,       // -1.0 .. 1.0, default 0.0 (negative = cool, positive = warm)
    pub vignette: VignetteParams,
    pub grain: GrainParams,
}

impl Default for FinishingParams {
    fn default() -> Self {
        Self {
            enabled: true,
            transparency: 0.0,
            warmth: 0.0,
            vignette: VignetteParams::default(),
            grain: GrainParams::default(),
        }
    }
}

/// The complete parameter set matching Topaz Adjust 4.0.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdjustParams {
    pub basic_exposure: BasicExposureParams,
    pub adaptive_exposure: AdaptiveExposureParams,
    pub details: DetailsParams,
    pub color: ColorParams,
    pub noise: NoiseParams,
    pub curve: CurveParams,
    pub finishing: FinishingParams,
}
