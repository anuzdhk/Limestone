use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
pub struct DetailsParams {
    pub enabled: bool,
    pub strength: f32,              // -1.0 .. 3.0, default 0.0 (negative softens/smooths)
    pub detail_boost: f32,          // 0.0 .. 3.0, default 1.0
    pub threshold: f32,             // 0.0 .. 1.0, default 0.10
    pub radius: f32,                // 0.5 .. 30.0, default 1.0
    pub sharpen: f32,               // 0.0 .. 2.0, default 0.0 (high frequency unsharp mask)
    pub process_independently: bool,// default true
}

impl Default for DetailsParams {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 0.0,
            detail_boost: 1.0,
            threshold: 0.10,
            radius: 1.0,
            sharpen: 0.0,
            process_independently: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
pub struct CurveParams {
    pub enabled: bool,
    pub points: Vec<CurvePoint>, // Master (RGB) control points
    /// Per-channel curves applied after the master curve. Empty = identity.
    pub red: Vec<CurvePoint>,
    pub green: Vec<CurvePoint>,
    pub blue: Vec<CurvePoint>,
}

impl Default for CurveParams {
    fn default() -> Self {
        Self::linear()
    }
}

impl CurveParams {
    pub fn linear() -> Self {
        Self::from_points(false, &[(0.0, 0.0), (1.0, 1.0)])
    }

    /// Builds an enabled-or-not master curve from `(x, y)` pairs.
    pub fn from_points(enabled: bool, points: &[(f32, f32)]) -> Self {
        Self {
            enabled,
            points: pts(points),
            red: Vec::new(),
            green: Vec::new(),
            blue: Vec::new(),
        }
    }

    pub fn medium_contrast() -> Self {
        Self::from_points(true, &[(0.0, 0.0), (0.25, 0.18), (0.75, 0.82), (1.0, 1.0)])
    }

    pub fn high_contrast() -> Self {
        Self::from_points(true, &[(0.0, 0.0), (0.25, 0.12), (0.75, 0.88), (1.0, 1.0)])
    }

    pub fn lift_shadows() -> Self {
        Self::from_points(true, &[(0.0, 0.08), (0.35, 0.42), (1.0, 1.0)])
    }

    pub fn matte_film() -> Self {
        Self::from_points(true, &[(0.0, 0.10), (0.30, 0.25), (0.70, 0.75), (1.0, 0.92)])
    }
}

/// Converts `(x, y)` pairs into curve points.
pub fn pts(points: &[(f32, f32)]) -> Vec<CurvePoint> {
    points.iter().map(|&(x, y)| CurvePoint { x, y }).collect()
}

/// Black & white conversion and split toning.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToneParams {
    pub enabled: bool,
    pub monochrome: f32,        // 0.0 .. 1.0, blend toward black & white
    pub mono_mix: [f32; 3],     // R/G/B channel mixer weights for the B&W conversion
    pub shadow_hue: f32,        // 0 .. 360 degrees
    pub shadow_strength: f32,   // 0.0 .. 1.0
    pub highlight_hue: f32,     // 0 .. 360 degrees
    pub highlight_strength: f32,// 0.0 .. 1.0
    pub balance: f32,           // -1.0 .. 1.0 (negative favours shadows tint)
}

impl Default for ToneParams {
    fn default() -> Self {
        Self {
            enabled: true,
            monochrome: 0.0,
            mono_mix: [0.30, 0.59, 0.11],
            shadow_hue: 30.0,
            shadow_strength: 0.0,
            highlight_hue: 45.0,
            highlight_strength: 0.0,
            balance: 0.0,
        }
    }
}

/// Creative effects: sketch, glow, posterize, solarize, invert.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StylizeParams {
    pub enabled: bool,
    pub sketch: f32,        // 0.0 .. 1.0 strength of the pencil/colour sketch
    pub sketch_radius: f32, // 1.0 .. 20.0 reference pixels (line thickness)
    pub sketch_color: f32,  // 0.0 (graphite) .. 1.0 (keep colours)
    pub glow: f32,          // 0.0 .. 1.0 soft bloom
    pub glow_radius: f32,   // 2.0 .. 60.0 reference pixels
    pub posterize: f32,     // 0.0 .. 1.0 (0 = off, 1 = 3 tone levels)
    pub solarize: f32,      // 0.0 .. 1.0
    pub invert: f32,        // 0.0 .. 1.0
}

impl Default for StylizeParams {
    fn default() -> Self {
        Self {
            enabled: true,
            sketch: 0.0,
            sketch_radius: 4.0,
            sketch_color: 0.0,
            glow: 0.0,
            glow_radius: 12.0,
            posterize: 0.0,
            solarize: 0.0,
            invert: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
pub struct BorderParams {
    pub enabled: bool,
    pub thickness: f32,  // 0.0 .. 0.20 (percentage of min dimension), default 0.0
    pub color: [f32; 3], // RGB in 0.0 .. 1.0, default [1.0, 1.0, 1.0] (White)
}

impl Default for BorderParams {
    fn default() -> Self {
        Self {
            enabled: false,
            thickness: 0.0,
            color: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FinishingParams {
    pub enabled: bool,
    pub transparency: f32, // 0.0 .. 1.0, default 0.0 (0.0 = 100% effect, 1.0 = original image)
    pub warmth: f32,       // -1.0 .. 1.0, default 0.0 (negative = cool, positive = warm)
    pub vignette: VignetteParams,
    pub grain: GrainParams,
    pub border: BorderParams,
}

impl Default for FinishingParams {
    fn default() -> Self {
        Self {
            enabled: true,
            transparency: 0.0,
            warmth: 0.0,
            vignette: VignetteParams::default(),
            grain: GrainParams::default(),
            border: BorderParams::default(),
        }
    }
}

/// The complete adjustment parameter set.
///
/// Every struct uses `#[serde(default)]`, so presets or recovery files that are missing
/// fields (e.g. written by an older version) still load with defaults for the gaps.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdjustParams {
    pub basic_exposure: BasicExposureParams,
    pub adaptive_exposure: AdaptiveExposureParams,
    pub details: DetailsParams,
    pub color: ColorParams,
    pub noise: NoiseParams,
    pub stylize: StylizeParams,
    pub tone: ToneParams,
    pub curve: CurveParams,
    pub finishing: FinishingParams,
}

/// Clamps `v` into `[lo, hi]`, replacing NaN/infinite values with `default`.
fn fin(v: f32, lo: f32, hi: f32, default: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        default
    }
}

impl AdjustParams {
    /// Returns a copy with every value forced into its supported range.
    ///
    /// Presets are user-editable JSON, so the pipeline never trusts them directly:
    /// out-of-range values (e.g. a border thicker than half the image) or NaNs would
    /// otherwise cause panics or garbage output.
    pub fn sanitized(&self) -> Self {
        let mut p = self.clone();

        let b = &mut p.basic_exposure;
        b.brightness = fin(b.brightness, -1.0, 1.0, 0.0);
        b.contrast = fin(b.contrast, -1.0, 1.0, 0.0);
        b.boost_blacks = fin(b.boost_blacks, 0.0, 1.0, 0.0);
        b.boost_whites = fin(b.boost_whites, 0.0, 1.0, 0.0);

        let a = &mut p.adaptive_exposure;
        a.adaptive_exposure = fin(a.adaptive_exposure, 0.0, 3.0, 0.0);
        a.regions = a.regions.clamp(1, 30);
        a.contrast = fin(a.contrast, -1.0, 1.0, 0.0);
        a.brightness = fin(a.brightness, -1.0, 1.0, 0.0);
        a.protect_highlights = fin(a.protect_highlights, 0.0, 1.0, 0.0);
        a.protect_shadows = fin(a.protect_shadows, 0.0, 1.0, 0.0);

        let d = &mut p.details;
        d.strength = fin(d.strength, -1.0, 3.0, 0.0);
        d.detail_boost = fin(d.detail_boost, 0.0, 3.0, 1.0);
        d.threshold = fin(d.threshold, 0.0, 1.0, 0.10);
        d.radius = fin(d.radius, 0.5, 30.0, 1.0);
        d.sharpen = fin(d.sharpen, 0.0, 2.0, 0.0);

        let c = &mut p.color;
        c.adaptive_saturation = fin(c.adaptive_saturation, -1.0, 2.0, 0.0);
        c.color_regions = c.color_regions.clamp(1, 30);
        c.saturation = fin(c.saturation, -1.0, 1.0, 0.0);
        c.saturation_boost = fin(c.saturation_boost, 0.0, 2.0, 1.0);
        c.hue = fin(c.hue, -180.0, 180.0, 0.0);

        let n = &mut p.noise;
        n.suppression = fin(n.suppression, 0.0, 1.0, 0.0);
        n.amount = fin(n.amount, 0.0, 1.0, 0.5);

        for curve in [
            &mut p.curve.points,
            &mut p.curve.red,
            &mut p.curve.green,
            &mut p.curve.blue,
        ] {
            curve.retain(|pt| pt.x.is_finite() && pt.y.is_finite());
            for pt in curve.iter_mut() {
                pt.x = pt.x.clamp(0.0, 1.0);
                pt.y = pt.y.clamp(0.0, 1.0);
            }
        }

        let st = &mut p.stylize;
        st.sketch = fin(st.sketch, 0.0, 1.0, 0.0);
        st.sketch_radius = fin(st.sketch_radius, 1.0, 20.0, 4.0);
        st.sketch_color = fin(st.sketch_color, 0.0, 1.0, 0.0);
        st.glow = fin(st.glow, 0.0, 1.0, 0.0);
        st.glow_radius = fin(st.glow_radius, 2.0, 60.0, 12.0);
        st.posterize = fin(st.posterize, 0.0, 1.0, 0.0);
        st.solarize = fin(st.solarize, 0.0, 1.0, 0.0);
        st.invert = fin(st.invert, 0.0, 1.0, 0.0);

        let t = &mut p.tone;
        t.monochrome = fin(t.monochrome, 0.0, 1.0, 0.0);
        for w in &mut t.mono_mix {
            *w = fin(*w, -1.0, 2.0, 0.33);
        }
        t.shadow_hue = fin(t.shadow_hue, 0.0, 360.0, 30.0);
        t.shadow_strength = fin(t.shadow_strength, 0.0, 1.0, 0.0);
        t.highlight_hue = fin(t.highlight_hue, 0.0, 360.0, 45.0);
        t.highlight_strength = fin(t.highlight_strength, 0.0, 1.0, 0.0);
        t.balance = fin(t.balance, -1.0, 1.0, 0.0);

        let f = &mut p.finishing;
        f.transparency = fin(f.transparency, 0.0, 1.0, 0.0);
        f.warmth = fin(f.warmth, -1.0, 1.0, 0.0);
        f.vignette.amount = fin(f.vignette.amount, -1.0, 1.0, 0.0);
        f.vignette.size = fin(f.vignette.size, 0.1, 1.0, 0.5);
        f.vignette.transition = fin(f.vignette.transition, 0.0, 1.0, 0.5);
        f.vignette.roundness = fin(f.vignette.roundness, 0.0, 1.0, 0.0);
        f.grain.amount = fin(f.grain.amount, 0.0, 1.0, 0.0);
        f.grain.size = fin(f.grain.size, 0.5, 3.0, 1.0);
        f.border.thickness = fin(f.border.thickness, 0.0, 0.2, 0.0);
        for ch in &mut f.border.color {
            *ch = fin(*ch, 0.0, 1.0, 1.0);
        }

        p
    }
}
