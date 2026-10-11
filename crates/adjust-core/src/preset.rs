//! Factory preset library, organised into the classic Adjust 4 collections.
//!
//! Presets are written with a small builder ([`P`]) so each look is one readable line
//! instead of a 60-line struct literal.

use crate::params::{pts, AdjustParams, CurveParams};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    /// Primary collection this preset belongs to.
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

/// A named group of presets. A preset can appear in several collections.
#[derive(Clone, Debug)]
pub struct PresetCollection {
    pub name: &'static str,
    pub description: &'static str,
    pub presets: Vec<&'static str>,
}

pub const CLASSIC: &str = "Classic Collection";
pub const VIBRANT: &str = "Vibrant Collection";
pub const HDR: &str = "HDR Collection";
pub const FILM: &str = "Film Collection";
pub const TONED: &str = "Toned Collection";
pub const STYLIZED: &str = "Stylized Collection";
pub const ADJUST4: &str = "Adjust 4 Preset List";

/// The curated "Adjust 4 Preset List": highlights drawn from every collection.
const ADJUST4_LIST: &[&str] = &[
    "Clarity", "Color Blast", "Crisp", "Dark - Ghostly", "Dark - Night", "Detail - Simple Pop",
    "Detail - Strong", "Detailed", "Dramatic", "Enhance Contrast", "Equalize",
    "Exposure Color Sketch", "Exposure Correction", "HDR - Pop", "HDR - Sketch",
    "Mild Color Pop", "Mild Contrast Pop", "Mild Details", "Neutralizer by JorgeDiaz",
    "Painting - Venice", "Photo Pop", "Portrait Drama", "Portrait Smooth", "Psychedelic",
    "Recovery - Highlight", "Recovery - Shadow", "Simplify", "Sketch - Color",
    "Sketch - Dark Charcoal", "Sketch - Med", "Sketch - Pencil", "Small Details",
    "Smooth and Flat", "Spicify", "Vibrance",
];

/// Collections in display order, each listing its presets alphabetically.
pub fn get_collections() -> Vec<PresetCollection> {
    let presets = get_factory_presets();
    let members = |cat: &str| -> Vec<&'static str> {
        let mut names: Vec<&'static str> = FACTORY
            .iter()
            .filter(|(_, c, _, _)| *c == cat)
            .map(|(n, _, _, _)| *n)
            .collect();
        names.sort_unstable_by_key(|n| n.to_lowercase());
        names
    };
    debug_assert!(ADJUST4_LIST.iter().all(|n| presets.iter().any(|p| p.name == *n)));

    vec![
        PresetCollection {
            name: ADJUST4,
            description: "The essential looks from every collection",
            presets: ADJUST4_LIST.to_vec(),
        },
        PresetCollection {
            name: CLASSIC,
            description: "Everyday exposure, detail and tone corrections",
            presets: members(CLASSIC),
        },
        PresetCollection {
            name: VIBRANT,
            description: "Punchy colour and contrast",
            presets: members(VIBRANT),
        },
        PresetCollection {
            name: HDR,
            description: "Tone-mapped high dynamic range looks",
            presets: members(HDR),
        },
        PresetCollection {
            name: FILM,
            description: "Film stocks, cross processing and vintage prints",
            presets: members(FILM),
        },
        PresetCollection {
            name: TONED,
            description: "Black & white, sepia and split tones",
            presets: members(TONED),
        },
        PresetCollection {
            name: STYLIZED,
            description: "Sketches, paintings and creative effects",
            presets: members(STYLIZED),
        },
    ]
}

/// All factory presets, sorted by name.
pub fn get_factory_presets() -> Vec<Preset> {
    let mut out: Vec<Preset> = FACTORY
        .iter()
        .map(|(name, cat, desc, build)| Preset::new(name, cat, desc, build().0))
        .collect();
    out.sort_by_key(|p| p.name.to_lowercase());
    out
}

type Recipe = (&'static str, &'static str, &'static str, fn() -> P);

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Fluent preset builder. Every method sets one processing stage.
#[derive(Clone)]
pub struct P(pub AdjustParams);

impl P {
    fn new() -> Self {
        P(AdjustParams::default())
    }
    /// brightness, contrast, boost blacks, boost whites
    fn exposure(mut self, brightness: f32, contrast: f32, blacks: f32, whites: f32) -> Self {
        let e = &mut self.0.basic_exposure;
        (e.brightness, e.contrast, e.boost_blacks, e.boost_whites) = (brightness, contrast, blacks, whites);
        self
    }
    /// adaptive exposure strength, regions, contrast, brightness, protect highlights, protect shadows
    fn adaptive(mut self, amount: f32, regions: u32, contrast: f32, brightness: f32, ph: f32, ps: f32) -> Self {
        let a = &mut self.0.adaptive_exposure;
        a.adaptive_exposure = amount;
        a.regions = regions;
        a.contrast = contrast;
        a.brightness = brightness;
        a.protect_highlights = ph;
        a.protect_shadows = ps;
        self
    }
    /// detail strength, boost, threshold, radius, sharpen
    fn details(mut self, strength: f32, boost: f32, threshold: f32, radius: f32, sharpen: f32) -> Self {
        let d = &mut self.0.details;
        (d.strength, d.detail_boost, d.threshold, d.radius, d.sharpen) = (strength, boost, threshold, radius, sharpen);
        self
    }
    /// adaptive saturation, saturation, saturation boost, hue shift
    fn color(mut self, adaptive_sat: f32, sat: f32, boost: f32, hue: f32) -> Self {
        let c = &mut self.0.color;
        (c.adaptive_saturation, c.saturation, c.saturation_boost, c.hue) = (adaptive_sat, sat, boost, hue);
        self
    }
    fn sat(mut self, sat: f32) -> Self {
        self.0.color.saturation = sat;
        self
    }
    fn noise(mut self, suppression: f32, amount: f32) -> Self {
        (self.0.noise.suppression, self.0.noise.amount) = (suppression, amount);
        self
    }
    fn curve(mut self, points: &[(f32, f32)]) -> Self {
        self.0.curve.enabled = true;
        self.0.curve.points = pts(points);
        self
    }
    fn curve_preset(mut self, c: CurveParams) -> Self {
        self.0.curve = c;
        self
    }
    fn rgb_curves(mut self, r: &[(f32, f32)], g: &[(f32, f32)], b: &[(f32, f32)]) -> Self {
        let c = &mut self.0.curve;
        c.enabled = true;
        (c.red, c.green, c.blue) = (pts(r), pts(g), pts(b));
        self
    }
    fn mono(mut self, amount: f32) -> Self {
        self.0.tone.monochrome = amount;
        self
    }
    fn mono_mix(mut self, r: f32, g: f32, b: f32) -> Self {
        self.0.tone.mono_mix = [r, g, b];
        self
    }
    /// One tint for the whole tonal range (sepia, selenium, cyanotype...).
    fn toned(self, hue: f32, strength: f32) -> Self {
        self.split(hue, strength, hue, strength, 0.0)
    }
    fn split(mut self, shadow_hue: f32, shadow: f32, highlight_hue: f32, highlight: f32, balance: f32) -> Self {
        let t = &mut self.0.tone;
        (t.shadow_hue, t.shadow_strength, t.highlight_hue, t.highlight_strength, t.balance) =
            (shadow_hue, shadow, highlight_hue, highlight, balance);
        self
    }
    fn sketch(mut self, amount: f32, radius: f32, color: f32) -> Self {
        let s = &mut self.0.stylize;
        (s.sketch, s.sketch_radius, s.sketch_color) = (amount, radius, color);
        self
    }
    fn glow(mut self, amount: f32, radius: f32) -> Self {
        (self.0.stylize.glow, self.0.stylize.glow_radius) = (amount, radius);
        self
    }
    fn posterize(mut self, amount: f32) -> Self {
        self.0.stylize.posterize = amount;
        self
    }
    fn solarize(mut self, amount: f32) -> Self {
        self.0.stylize.solarize = amount;
        self
    }
    fn invert(mut self, amount: f32) -> Self {
        self.0.stylize.invert = amount;
        self
    }
    fn warmth(mut self, w: f32) -> Self {
        self.0.finishing.warmth = w;
        self
    }
    fn vignette(mut self, amount: f32, size: f32, transition: f32, roundness: f32) -> Self {
        let v = &mut self.0.finishing.vignette;
        (v.amount, v.size, v.transition, v.roundness) = (amount, size, transition, roundness);
        self
    }
    fn grain(mut self, amount: f32, size: f32) -> Self {
        (self.0.finishing.grain.amount, self.0.finishing.grain.size) = (amount, size);
        self
    }
    fn border(mut self, thickness: f32, color: [f32; 3]) -> Self {
        let b = &mut self.0.finishing.border;
        (b.enabled, b.thickness, b.color) = (true, thickness, color);
        self
    }
}

// Reusable building blocks ---------------------------------------------------

/// Local-contrast "pop" with matching detail and colour.
fn pop(level: f32) -> P {
    P::new()
        .adaptive(0.5 * level, 12, 0.15 * level, 0.0, 0.3, 0.2)
        .details(0.9 * level, 1.15, 0.08, 1.2, 0.15 * level)
        .color(0.35 * level, 0.06 * level, 1.1, 0.0)
}

/// HDR-style tone mapping with detail; `grit` > 0 adds texture, < 0 smooths it.
fn hdr(level: f32, grit: f32) -> P {
    let base = P::new().adaptive(0.7 * level, 14, 0.12 * level, 0.03, 0.45, 0.45);
    if grit >= 0.0 {
        base.details(1.0 + 1.0 * grit, 1.2 + 0.5 * grit, 0.07, 1.5, 0.2 * grit)
            .color(0.35, -0.12 * grit, 1.0, 0.0)
    } else {
        base.details(-0.6 * -grit, 1.0, 0.2, 2.5, 0.0)
            .noise(0.6 * -grit, 0.6)
            .color(0.45, 0.05, 1.1, 0.0)
    }
}

/// Black & white base.
fn bw() -> P {
    P::new().mono(1.0).exposure(0.0, 0.1, 0.05, 0.05)
}

/// Faded film base: lifted blacks, rolled highlights.
fn faded() -> P {
    P::new().curve(&[(0.0, 0.10), (0.30, 0.27), (0.70, 0.74), (1.0, 0.92)]).sat(-0.2)
}

/// Pencil sketch base.
fn pencil(radius: f32) -> P {
    P::new().sketch(1.0, radius, 0.0).mono(1.0)
}

/// Oil-painting base: smoothed texture, simplified tones.
fn painting() -> P {
    P::new()
        .noise(0.9, 1.0)
        .details(-0.6, 1.0, 0.25, 3.0, 0.0)
        .posterize(0.25)
        .sketch(0.18, 2.0, 1.0)
}

// ---------------------------------------------------------------------------
// Library
// ---------------------------------------------------------------------------

#[rustfmt::skip]
static FACTORY: &[Recipe] = &[
    // ===== Classic Collection ==============================================
    ("Brilliant Cold", CLASSIC, "Crisp, luminous rendering with a cool cast", || pop(1.0).warmth(-0.25)),
    ("Brilliant Warm", CLASSIC, "Crisp, luminous rendering with a warm glow", || pop(1.0).warmth(0.25)),
    ("Cool Tone I", CLASSIC, "Gentle cool white balance", || P::new().warmth(-0.3).sat(-0.05)),
    ("Cool Tone II", CLASSIC, "Stronger cool cast with blue shadows", || P::new().warmth(-0.45).rgb_curves(&[], &[], &[(0.0, 0.06), (1.0, 1.0)])),
    ("Delicate Details I", CLASSIC, "Fine texture enhancement for subtle subjects", || P::new().adaptive(0.2, 10, 0.0, 0.0, 0.2, 0.2).details(0.8, 1.4, 0.05, 0.8, 0.1)),
    ("Delicate Details II", CLASSIC, "More fine texture, still natural", || P::new().adaptive(0.3, 10, 0.05, 0.0, 0.2, 0.2).details(1.2, 1.6, 0.04, 0.8, 0.15)),
    ("Detail - Light", CLASSIC, "Light micro-contrast boost", || P::new().details(0.7, 1.1, 0.10, 1.5, 0.1)),
    ("Detail - Medium", CLASSIC, "Moderate micro-contrast boost", || P::new().adaptive(0.15, 10, 0.05, 0.0, 0.2, 0.1).details(1.3, 1.25, 0.08, 1.5, 0.2)),
    ("Equalize", CLASSIC, "Evens out exposure across the frame", || P::new().adaptive(1.2, 6, 0.0, 0.0, 0.6, 0.6).color(0.1, 0.0, 1.0, 0.0)),
    ("Exposure Correction", CLASSIC, "Balances under- and over-exposed areas", || P::new().adaptive(0.8, 8, 0.05, 0.05, 0.5, 0.45)),
    ("High Key I", CLASSIC, "Bright, airy rendering", || P::new().exposure(0.25, -0.1, 0.0, 0.3).adaptive(0.4, 10, 0.0, 0.15, 0.2, 0.5).sat(-0.1)),
    ("High Key II", CLASSIC, "Very bright, soft and pale", || P::new().exposure(0.35, -0.15, 0.0, 0.4).details(-0.3, 1.0, 0.2, 2.0, 0.0).sat(-0.25)),
    ("Highlight Recovery", CLASSIC, "Pulls detail back into bright areas", || P::new().adaptive(0.5, 10, 0.0, -0.05, 0.9, 0.0)),
    ("Low Key I", CLASSIC, "Dark, moody rendering", || P::new().exposure(-0.25, 0.2, 0.3, 0.0).adaptive(0.4, 10, 0.1, -0.05, 0.3, 0.0).vignette(-0.3, 0.55, 0.5, 0.3)),
    ("Low Key II", CLASSIC, "Deep shadows and muted colour", || P::new().exposure(-0.35, 0.3, 0.4, 0.05).sat(-0.15).vignette(-0.45, 0.5, 0.5, 0.3)),
    ("Mild Color Pop", CLASSIC, "Subtle colour enhancement", || P::new().adaptive(0.2, 10, 0.05, 0.0, 0.2, 0.2).color(0.4, 0.08, 1.1, 0.0)),
    ("Mild Contrast Pop", CLASSIC, "Subtle local contrast for almost any scene", || P::new().adaptive(0.35, 5, 0.12, 0.0, 0.25, 0.2).details(0.5, 1.1, 0.1, 1.2, 0.1).color(0.15, 0.0, 1.0, 0.0)),
    ("Photo Pop", CLASSIC, "All-round punch: contrast, detail and colour", || pop(1.4)),
    ("Shadow Recovery", CLASSIC, "Opens up dark areas", || P::new().adaptive(0.6, 10, 0.0, 0.05, 0.0, 0.9)),
    ("Simple Pop", CLASSIC, "A light all-round lift", || pop(0.6)),
    ("Soft Focus I", CLASSIC, "Dreamy softening with a gentle glow", || P::new().details(-0.6, 1.0, 0.2, 4.0, 0.0).glow(0.25, 15.0)),
    ("Soft Focus II", CLASSIC, "Strong diffusion glow", || P::new().exposure(0.05, -0.05, 0.0, 0.0).details(-0.9, 1.0, 0.25, 5.0, 0.0).glow(0.4, 20.0)),
    ("Warm Tone I", CLASSIC, "Gentle warm white balance", || P::new().warmth(0.3)),
    ("Warm Tone II", CLASSIC, "Golden warm cast", || P::new().warmth(0.45).rgb_curves(&[(0.0, 0.03), (1.0, 1.0)], &[], &[])),

    // ===== Vibrant Collection ==============================================
    ("Autumn", VIBRANT, "Rich golds and reds", || P::new().adaptive(0.5, 12, 0.1, 0.0, 0.3, 0.2).color(0.6, 0.15, 1.2, -8.0).warmth(0.35)),
    ("Bold", VIBRANT, "Strong contrast, detail and colour", || pop(1.8).color(0.5, 0.15, 1.2, 0.0)),
    ("Boost", VIBRANT, "Quick contrast and colour boost", || P::new().exposure(0.05, 0.15, 0.1, 0.1).color(0.5, 0.1, 1.1, 0.0)),
    ("Clarity", VIBRANT, "Mid-tone clarity without halos", || P::new().adaptive(0.3, 12, 0.1, 0.0, 0.3, 0.2).details(1.5, 1.3, 0.06, 8.0, 0.1)),
    ("Color Blast", VIBRANT, "Very saturated, vivid colour", || P::new().adaptive(0.6, 12, 0.1, 0.0, 0.3, 0.2).color(1.2, 0.3, 1.4, 0.0)),
    ("Color Pop I", VIBRANT, "Lively colour", || P::new().adaptive(0.4, 12, 0.05, 0.0, 0.3, 0.2).color(0.8, 0.15, 1.2, 0.0)),
    ("Color Pop II", VIBRANT, "Livelier colour with detail", || P::new().adaptive(0.6, 12, 0.1, 0.0, 0.3, 0.2).details(0.6, 1.1, 0.08, 1.2, 0.1).color(1.0, 0.25, 1.3, 0.0)),
    ("Crisp", VIBRANT, "Sharp, crunchy edges", || P::new().details(1.6, 1.3, 0.06, 0.8, 0.5)),
    ("Detail - Strong", VIBRANT, "Heavy micro-contrast", || P::new().adaptive(0.3, 12, 0.1, 0.0, 0.3, 0.2).details(2.0, 1.4, 0.05, 1.5, 0.3)),
    ("Detail - Strong II", VIBRANT, "Extreme micro-contrast", || P::new().adaptive(0.4, 12, 0.15, 0.0, 0.3, 0.2).details(2.6, 1.6, 0.04, 2.0, 0.4)),
    ("Detailed Color", VIBRANT, "Detail plus colour", || P::new().adaptive(0.4, 12, 0.1, 0.0, 0.3, 0.2).details(1.4, 1.25, 0.07, 1.5, 0.2).color(0.6, 0.1, 1.15, 0.0)),
    ("Dramatic", VIBRANT, "Cinematic gritty contrast with muted colour", || P::new().exposure(-0.05, 0.25, 0.2, 0.1).adaptive(0.95, 16, 0.3, -0.05, 0.4, 0.3).details(2.1, 1.5, 0.06, 1.5, 0.4).color(-0.2, -0.35, 0.8, 0.0).curve_preset(CurveParams::medium_contrast()).warmth(-0.08).vignette(-0.3, 0.5, 0.4, 0.2).grain(0.15, 1.0)),
    ("Dramatic II", VIBRANT, "Even darker, grittier drama", || P::new().exposure(-0.08, 0.3, 0.25, 0.1).adaptive(1.4, 16, 0.35, -0.05, 0.45, 0.3).details(2.6, 1.6, 0.05, 1.5, 0.45).color(-0.3, -0.5, 0.8, 0.0).curve_preset(CurveParams::high_contrast()).vignette(-0.4, 0.5, 0.4, 0.2).grain(0.18, 1.0)),
    ("Exposure Color Stretch", VIBRANT, "Equalized exposure with stretched colour", || P::new().adaptive(1.3, 8, 0.2, 0.05, 0.5, 0.5).color(0.7, 0.15, 1.2, 0.0)),
    ("French Countryside", VIBRANT, "Warm, soft, sunlit pastoral look", || P::new().adaptive(0.5, 10, 0.05, 0.05, 0.4, 0.3).details(-0.2, 1.0, 0.15, 3.0, 0.0).color(0.4, 0.1, 1.1, 0.0).glow(0.2, 18.0).warmth(0.2).vignette(-0.2, 0.6, 0.6, 0.6)),
    ("Gritty I", VIBRANT, "Textured and desaturated", || P::new().adaptive(0.9, 14, 0.2, 0.0, 0.4, 0.3).details(2.2, 1.6, 0.04, 2.0, 0.4).sat(-0.3).grain(0.15, 1.0)),
    ("Gritty II", VIBRANT, "Heavier texture, less colour", || P::new().adaptive(1.2, 14, 0.25, 0.0, 0.45, 0.3).details(2.7, 1.8, 0.04, 2.0, 0.5).sat(-0.45).grain(0.2, 1.0).vignette(-0.25, 0.55, 0.5, 0.2)),
    ("High Contrast", VIBRANT, "Punchy global contrast", || P::new().exposure(0.0, 0.4, 0.15, 0.1).adaptive(0.3, 10, 0.3, 0.0, 0.3, 0.2).curve_preset(CurveParams::high_contrast())),
    ("Neutralizer by JorgeDiaz", VIBRANT, "Flattened, neutral starting point", || P::new().exposure(0.0, -0.05, 0.0, 0.0).adaptive(0.9, 10, -0.1, 0.0, 0.5, 0.5).sat(-0.15)),
    ("Portrait Drama", VIBRANT, "Moody, textured portrait", || P::new().adaptive(1.0, 14, 0.2, -0.05, 0.4, 0.3).details(1.6, 1.4, 0.06, 1.5, 0.3).sat(-0.55).warmth(0.05).vignette(-0.35, 0.5, 0.5, 0.4)),
    ("Relative Contrast", VIBRANT, "Strong regional contrast", || P::new().adaptive(0.5, 6, 0.45, 0.0, 0.3, 0.3)),
    ("Sapphire", VIBRANT, "Cool jewel-toned blues", || P::new().adaptive(0.7, 12, 0.15, 0.0, 0.4, 0.3).color(0.5, 0.2, 1.2, -6.0).warmth(-0.35).split(220.0, 0.15, 200.0, 0.1, 0.0)),
    ("Saturation Punch", VIBRANT, "Maximum saturation", || P::new().color(1.5, 0.4, 1.5, 0.0)),
    ("Setting Sun", VIBRANT, "Golden-hour warmth", || P::new().color(0.5, 0.15, 1.2, 0.0).warmth(0.6).rgb_curves(&[(0.0, 0.03), (0.5, 0.56), (1.0, 1.0)], &[], &[(0.0, 0.0), (1.0, 0.88)])),
    ("Setting Sun - Dynamic", VIBRANT, "Golden hour with HDR punch", || P::new().adaptive(1.3, 12, 0.2, 0.0, 0.5, 0.4).details(1.0, 1.2, 0.07, 1.5, 0.2).color(0.5, 0.15, 1.2, 0.0).warmth(0.6).rgb_curves(&[(0.0, 0.03), (0.5, 0.56), (1.0, 1.0)], &[], &[(0.0, 0.0), (1.0, 0.88)])),
    ("Spicify", VIBRANT, "Signature punchy dynamic range and colour", || P::new().exposure(0.0, 0.12, 0.08, 0.05).adaptive(1.25, 12, 0.18, 0.04, 0.35, 0.25).details(1.4, 1.2, 0.08, 1.2, 0.2).color(0.65, 0.1, 1.15, 0.0).noise(0.15, 0.4).warmth(0.05).vignette(-0.15, 0.6, 0.5, 0.0)),
    ("Spring", VIBRANT, "Fresh greens and soft light", || P::new().exposure(0.05, 0.0, 0.0, 0.05).color(0.6, 0.15, 1.2, 4.0).warmth(0.05)),
    ("Summer", VIBRANT, "Bright, warm and saturated", || P::new().exposure(0.08, 0.05, 0.0, 0.1).color(0.7, 0.15, 1.2, 0.0).warmth(0.2)),
    ("Vibrance", VIBRANT, "Boosts muted colours, protects skin", || P::new().color(0.9, 0.0, 1.2, 0.0)),
    ("Winter", VIBRANT, "Cold, crisp and pale", || P::new().exposure(0.08, 0.05, 0.0, 0.2).sat(-0.2).warmth(-0.4)),

    // ===== HDR Collection ==================================================
    ("Dynamic Brightness", HDR, "Brightens while compressing dynamic range", || P::new().exposure(0.08, 0.0, 0.0, 0.0).adaptive(1.0, 12, 0.1, 0.2, 0.4, 0.6)),
    ("Dynamic Pop", HDR, "HDR punch with colour", || hdr(2.0, 0.4).color(0.5, 0.1, 1.15, 0.0)),
    ("Dynamic Pop II", HDR, "Stronger HDR punch", || hdr(2.6, 0.8).color(0.6, 0.1, 1.2, 0.0)),
    ("Heavy Pop Grunge", HDR, "Maximum HDR grit", || hdr(2.8, 1.8).grain(0.12, 1.0)),
    ("Heavy Pop Smooth", HDR, "Maximum HDR, painterly smooth", || hdr(2.8, -1.0)),
    ("Light Pop Grunge", HDR, "Light HDR with texture", || hdr(1.3, 0.6)),
    ("Light Pop Smooth", HDR, "Light HDR, smooth surfaces", || hdr(1.3, -0.4)),
    ("Medium Pop Grunge", HDR, "Medium HDR with grit", || hdr(2.0, 1.2)),
    ("Medium Pop Smooth", HDR, "Medium HDR, smooth surfaces", || hdr(2.0, -0.7)),

    // ===== Film Collection =================================================
    ("Bleach Bypass Cool", FILM, "Silver-retained low saturation, cool", || P::new().exposure(0.0, 0.35, 0.15, 0.1).details(0.8, 1.2, 0.08, 1.5, 0.2).sat(-0.55).warmth(-0.15)),
    ("Bleach Bypass Warm", FILM, "Silver-retained low saturation, warm", || P::new().exposure(0.0, 0.35, 0.15, 0.1).details(0.8, 1.2, 0.08, 1.5, 0.2).sat(-0.55).warmth(0.15)),
    ("Color Negative", FILM, "Soft, orange-based negative film rendering", || P::new().exposure(0.0, -0.1, 0.0, 0.0).sat(-0.1).warmth(0.1).rgb_curves(&[(0.0, 0.05), (1.0, 0.95)], &[(0.0, 0.03), (1.0, 0.97)], &[(0.0, 0.08), (0.5, 0.45), (1.0, 0.9)])),
    ("Cross Process I", FILM, "Slide film in C-41: yellow highs, blue lows", || P::new().sat(0.2).rgb_curves(&[(0.0, 0.0), (0.25, 0.18), (0.75, 0.85), (1.0, 1.0)], &[(0.0, 0.0), (0.3, 0.27), (0.7, 0.76), (1.0, 1.0)], &[(0.0, 0.2), (1.0, 0.8)])),
    ("Cross Process II", FILM, "Warm cross process", || P::new().sat(0.15).rgb_curves(&[(0.0, 0.0), (0.5, 0.6), (1.0, 1.0)], &[], &[(0.0, 0.15), (1.0, 0.75)])),
    ("Cross Process III", FILM, "Green-tinted cross process", || P::new().sat(0.15).rgb_curves(&[], &[(0.0, 0.05), (0.5, 0.6), (1.0, 1.0)], &[(0.0, 0.1), (1.0, 0.85)])),
    ("Cross Process IV", FILM, "Magenta cross process", || P::new().sat(0.15).rgb_curves(&[(0.0, 0.05), (1.0, 1.0)], &[(0.0, 0.0), (0.5, 0.42), (1.0, 0.95)], &[(0.0, 0.1), (1.0, 0.95)])),
    ("Cross Process V", FILM, "Extreme cross process with vignette", || P::new().sat(0.35).rgb_curves(&[(0.0, 0.0), (0.25, 0.12), (0.75, 0.9), (1.0, 1.0)], &[(0.0, 0.0), (0.25, 0.2), (0.75, 0.82), (1.0, 1.0)], &[(0.0, 0.25), (1.0, 0.7)]).vignette(-0.4, 0.45, 0.5, 0.6)),
    ("Daylight Film in Tungsten I", FILM, "Daylight stock under tungsten: warm cast", || P::new().warmth(0.5)),
    ("Daylight Film in Tungsten II", FILM, "Stronger tungsten warmth", || P::new().warmth(0.7).rgb_curves(&[(0.0, 0.04), (1.0, 1.0)], &[], &[])),
    ("Lomo I", FILM, "Toy-camera saturation and vignette", || P::new().exposure(0.0, 0.3, 0.1, 0.0).sat(0.3).vignette(-0.6, 0.35, 0.5, 0.8)),
    ("Lomo II", FILM, "Stronger toy-camera look", || P::new().exposure(0.0, 0.35, 0.15, 0.0).sat(0.45).rgb_curves(&[(0.0, 0.0), (0.5, 0.55), (1.0, 1.0)], &[], &[(0.0, 0.08), (1.0, 0.9)]).vignette(-0.75, 0.35, 0.5, 0.8)),
    ("Lomo III", FILM, "Extreme toy-camera look", || P::new().exposure(0.0, 0.4, 0.2, 0.0).sat(0.6).rgb_curves(&[(0.0, 0.0), (0.5, 0.58), (1.0, 1.0)], &[(0.0, 0.0), (0.5, 0.52), (1.0, 1.0)], &[(0.0, 0.12), (1.0, 0.85)]).vignette(-0.9, 0.3, 0.45, 0.8)),
    ("Retro Style I", FILM, "Faded 70s print", || faded().warmth(0.15).grain(0.2, 1.2)),
    ("Retro Style II", FILM, "Faded print with green shadows", || faded().warmth(0.1).rgb_curves(&[], &[(0.0, 0.06), (1.0, 1.0)], &[]).grain(0.2, 1.2)),
    ("Retro Style III", FILM, "Faded print with magenta cast", || faded().rgb_curves(&[(0.0, 0.05), (1.0, 1.0)], &[(0.0, 0.0), (1.0, 0.94)], &[(0.0, 0.06), (1.0, 1.0)]).grain(0.2, 1.2)),
    ("Retro Style IV", FILM, "Heavily faded print with a white border", || faded().sat(-0.35).warmth(0.2).grain(0.25, 1.4).border(0.03, [0.96, 0.94, 0.9])),
    ("Timeless I", FILM, "Muted, warm and nostalgic", || faded().sat(-0.5).warmth(0.2)),
    ("Timeless II", FILM, "Near-monochrome warm print", || faded().mono(0.7).toned(35.0, 0.35)),
    ("Timeless III", FILM, "Warm-toned black & white", || bw().curve(&[(0.0, 0.06), (0.5, 0.5), (1.0, 0.95)]).toned(32.0, 0.45)),
    ("Timeless IV", FILM, "Antique warm print with grain", || bw().curve(&[(0.0, 0.08), (0.5, 0.48), (1.0, 0.92)]).toned(30.0, 0.55).grain(0.25, 1.3).vignette(-0.35, 0.5, 0.5, 0.5)),
    ("Tungsten Film in Daylight I", FILM, "Tungsten stock in daylight: blue cast", || P::new().warmth(-0.5)),
    ("Tungsten Film in Daylight II", FILM, "Stronger daylight blue", || P::new().warmth(-0.7).rgb_curves(&[], &[], &[(0.0, 0.05), (1.0, 1.0)])),
    ("Vintage Grunge I", FILM, "Worn, textured vintage print", || faded().details(1.8, 1.5, 0.05, 2.0, 0.3).sat(-0.4).warmth(0.25).grain(0.25, 1.2).vignette(-0.5, 0.45, 0.5, 0.5)),
    ("Vintage Grunge II", FILM, "Gritty sepia-leaning print", || faded().details(2.2, 1.6, 0.05, 2.0, 0.35).mono(0.5).toned(35.0, 0.3).grain(0.3, 1.3).vignette(-0.55, 0.45, 0.5, 0.5)),
    ("Vintage Grunge III", FILM, "Cool, worn print", || faded().details(1.8, 1.5, 0.05, 2.0, 0.3).sat(-0.45).warmth(-0.15).grain(0.25, 1.2).vignette(-0.5, 0.45, 0.5, 0.5)),
    ("Vintage Grunge IV", FILM, "Heavily distressed print", || faded().details(2.6, 1.8, 0.04, 2.5, 0.4).sat(-0.55).warmth(0.3).grain(0.35, 1.5).vignette(-0.7, 0.4, 0.45, 0.5)),

    // ===== Toned Collection ================================================
    ("Black Rose", TONED, "Deep red-black tone", || bw().exposure(0.0, 0.2, 0.15, 0.0).split(345.0, 0.5, 340.0, 0.2, -0.2)),
    ("Blue", TONED, "Blue-toned monochrome", || bw().toned(215.0, 0.45)),
    ("Brown", TONED, "Chocolate-brown tone", || bw().exposure(-0.05, 0.1, 0.05, 0.0).toned(28.0, 0.55)),
    ("Cerulean", TONED, "Cyanotype-style blue", || bw().toned(195.0, 0.5)),
    ("Colored Selenium I", TONED, "Partial colour with selenium-toned shadows", || P::new().mono(0.75).split(300.0, 0.35, 40.0, 0.1, -0.2)),
    ("Colored Selenium II", TONED, "More colour, selenium shadows", || P::new().mono(0.6).split(300.0, 0.3, 40.0, 0.1, -0.2)),
    ("Colored Sepia", TONED, "Partial colour with sepia", || P::new().mono(0.7).toned(35.0, 0.5)),
    ("Colored Sepia II", TONED, "More colour with sepia", || P::new().mono(0.55).toned(35.0, 0.45)),
    ("Detailed Black and White", TONED, "Rich, detailed monochrome", || bw().mono_mix(0.40, 0.50, 0.10).adaptive(0.8, 12, 0.15, 0.0, 0.4, 0.3).details(1.6, 1.4, 0.06, 1.5, 0.3)),
    ("Mint", TONED, "Cool mint tone", || bw().toned(150.0, 0.35)),
    ("Mustard", TONED, "Warm mustard tone", || bw().toned(50.0, 0.5)),
    ("Sharp Black and White", TONED, "Crisp high-contrast monochrome (red filter)", || bw().mono_mix(0.50, 0.42, 0.08).exposure(0.0, 0.25, 0.15, 0.1).details(1.2, 1.2, 0.06, 1.0, 0.6)),
    ("Soft Pink", TONED, "Delicate pink tone", || bw().exposure(0.08, -0.05, 0.0, 0.05).details(-0.3, 1.0, 0.2, 2.0, 0.0).toned(340.0, 0.3)),
    ("Split Tone I", TONED, "Blue shadows, amber highlights", || bw().split(220.0, 0.4, 35.0, 0.4, 0.0)),
    ("Split Tone II", TONED, "Teal shadows, yellow highlights", || bw().split(190.0, 0.4, 50.0, 0.35, 0.0)),
    ("Split Tone III", TONED, "Purple shadows, gold highlights", || bw().split(270.0, 0.4, 45.0, 0.35, 0.0)),
    ("Split Tone IV", TONED, "Green shadows, pink highlights", || bw().split(140.0, 0.35, 330.0, 0.3, 0.0)),
    ("Split Tone V", TONED, "Red shadows, cyan highlights", || bw().split(0.0, 0.35, 190.0, 0.3, 0.0)),
    ("Split Tone VI", TONED, "Split tone over partial colour", || P::new().mono(0.5).split(230.0, 0.35, 25.0, 0.35, 0.0)),

    // ===== Stylized Collection =============================================
    ("Blueprint", STYLIZED, "White lines on architectural blue", || pencil(3.0).invert(1.0).rgb_curves(&[(0.0, 0.05), (1.0, 0.9)], &[(0.0, 0.2), (1.0, 0.95)], &[(0.0, 0.5), (1.0, 1.0)])),
    ("Brash", STYLIZED, "Loud, gritty and saturated", || P::new().exposure(0.0, 0.2, 0.1, 0.1).adaptive(1.5, 14, 0.25, 0.0, 0.4, 0.3).details(2.5, 1.7, 0.04, 2.0, 0.4).color(0.9, 0.2, 1.3, 0.0)),
    ("Cheerful Color", STYLIZED, "Bright, happy colour with a soft glow", || P::new().exposure(0.1, 0.05, 0.0, 0.05).color(1.0, 0.2, 1.3, 0.0).glow(0.15, 15.0).warmth(0.1)),
    ("Colorized I", STYLIZED, "Strong amber colourisation", || P::new().mono(1.0).toned(35.0, 0.75)),
    ("Colorized II", STYLIZED, "Strong teal colourisation", || P::new().mono(1.0).toned(185.0, 0.75)),
    ("Colorized III", STYLIZED, "Strong magenta colourisation", || P::new().mono(1.0).toned(310.0, 0.7)),
    ("Comic Book", STYLIZED, "Inked edges and flat colour", || P::new().details(1.5, 1.2, 0.08, 1.5, 0.2).noise(0.6, 0.8).color(0.5, 0.4, 1.3, 0.0).posterize(0.7).sketch(0.4, 2.0, 1.0)),
    ("Comic Book Art", STYLIZED, "Bold ink and posterized colour", || P::new().noise(0.8, 1.0).color(0.7, 0.6, 1.4, 0.0).posterize(0.85).sketch(0.6, 2.5, 1.0)),
    ("Dark - Ghostly", STYLIZED, "Haunting pale glow", || P::new().exposure(-0.2, 0.1, 0.1, 0.0).mono(0.8).toned(200.0, 0.2).glow(0.4, 20.0).vignette(-0.6, 0.4, 0.5, 0.6)),
    ("Dark - Night", STYLIZED, "Day-for-night blue darkness", || P::new().exposure(-0.35, 0.15, 0.1, 0.0).adaptive(0.6, 10, 0.1, -0.1, 0.4, 0.0).sat(-0.3).warmth(-0.6).vignette(-0.5, 0.45, 0.5, 0.6)),
    ("Dayglow", STYLIZED, "Luminous glow with vivid colour", || P::new().exposure(0.1, 0.0, 0.0, 0.0).color(0.5, 0.3, 1.2, 0.0).glow(0.5, 25.0)),
    ("Desaturated Detail", STYLIZED, "Heavy detail with drained colour", || P::new().adaptive(1.0, 14, 0.2, 0.0, 0.4, 0.3).details(2.0, 1.5, 0.05, 1.5, 0.3).sat(-0.6)),
    ("Dusk", STYLIZED, "Purple-blue shadows, warm highlights", || P::new().exposure(-0.1, 0.1, 0.05, 0.0).sat(0.3).warmth(0.3).rgb_curves(&[(0.0, 0.0), (0.6, 0.66), (1.0, 1.0)], &[], &[(0.0, 0.12), (0.5, 0.48), (1.0, 0.9)])),
    ("Extreme Color", STYLIZED, "Over-the-top colour", || P::new().adaptive(1.4, 14, 0.2, 0.0, 0.4, 0.3).color(1.6, 0.6, 1.6, 0.0)),
    ("Extreme Color Blast", STYLIZED, "Maximum colour and detail", || P::new().adaptive(2.0, 14, 0.25, 0.0, 0.45, 0.35).details(1.5, 1.4, 0.06, 1.5, 0.3).color(2.0, 0.8, 2.0, 0.0)),
    ("Faded Glory", STYLIZED, "Washed-out, softly glowing nostalgia", || faded().sat(-0.45).warmth(0.15).glow(0.15, 18.0).grain(0.1, 1.0)),
    ("Glow In The Dark", STYLIZED, "Neon edges on black", || P::new().color(0.8, 0.4, 1.5, 0.0).sketch(1.0, 2.5, 1.0).invert(1.0).glow(0.5, 10.0)),
    ("Grunge Me", STYLIZED, "Gritty grunge texture", || P::new().adaptive(1.6, 16, 0.3, 0.0, 0.45, 0.35).details(2.8, 2.0, 0.03, 2.0, 0.5).sat(-0.2).grain(0.2, 1.2).vignette(-0.4, 0.5, 0.5, 0.3)),
    ("Grunge Me BW", STYLIZED, "Gritty grunge in black & white", || P::new().adaptive(1.6, 16, 0.3, 0.0, 0.45, 0.35).details(2.8, 2.0, 0.03, 2.0, 0.5).mono(1.0).grain(0.2, 1.2).vignette(-0.4, 0.5, 0.5, 0.3)),
    ("Grunge Me Desaturated", STYLIZED, "Gritty grunge, muted colour", || P::new().adaptive(1.6, 16, 0.3, 0.0, 0.45, 0.35).details(2.8, 2.0, 0.03, 2.0, 0.5).sat(-0.6).grain(0.2, 1.2).vignette(-0.4, 0.5, 0.5, 0.3)),
    ("HDR - Sketch", STYLIZED, "HDR tone mapping with drawn edges", || hdr(2.0, 0.6).sketch(0.5, 3.0, 1.0)),
    ("HDR Sketch - Light", STYLIZED, "HDR with a light sketch overlay", || hdr(1.5, 0.4).sketch(0.3, 3.0, 1.0)),
    ("High Definition", STYLIZED, "Hyper-sharp, clean and vivid", || P::new().adaptive(0.8, 12, 0.15, 0.0, 0.4, 0.3).details(1.8, 1.3, 0.06, 1.0, 0.6).color(0.3, 0.05, 1.1, 0.0).noise(0.2, 0.4)),
    ("Liquid Chrome", STYLIZED, "Metallic solarized monochrome", || P::new().exposure(0.0, 0.3, 0.1, 0.1).details(1.0, 1.2, 0.08, 1.5, 0.2).mono(1.0).solarize(0.45).toned(210.0, 0.15)),
    ("Mystified", STYLIZED, "Misty, ethereal glow", || P::new().exposure(0.05, -0.1, 0.0, 0.0).sat(-0.2).warmth(-0.2).glow(0.5, 30.0)),
    ("Painting - Bold", STYLIZED, "Bold, saturated oil painting", || painting().color(0.8, 0.25, 1.3, 0.0)),
    ("Painting - Venice", STYLIZED, "Warm, glowing Venetian painting", || painting().color(0.5, 0.1, 1.1, 0.0).warmth(0.25).glow(0.15, 15.0)),
    ("Portrait Smooth", STYLIZED, "Smooth skin, preserved edges", || P::new().details(-0.8, 1.0, 0.3, 3.0, 0.0).noise(0.4, 0.6).glow(0.1, 12.0).warmth(0.05)),
    ("Psychedelic", STYLIZED, "Solarized colour trip", || P::new().color(1.5, 0.5, 1.6, 0.0).solarize(0.5)),
    ("Radical Realism", STYLIZED, "Hyper-real tone mapping and detail", || P::new().adaptive(2.2, 16, 0.3, 0.0, 0.5, 0.4).details(2.5, 1.7, 0.04, 1.5, 0.4).color(0.8, 0.15, 1.3, 0.0)),
    ("Simplify", STYLIZED, "Removes texture, keeps form", || P::new().noise(1.0, 1.0).details(-1.0, 1.0, 0.3, 4.0, 0.0).posterize(0.3)),
    ("Sketch - Color", STYLIZED, "Coloured pencil drawing", || P::new().sketch(1.0, 4.0, 1.0)),
    ("Sketch - Dark Charcoal", STYLIZED, "Heavy, dark charcoal drawing", || pencil(8.0).exposure(-0.15, 0.4, 0.3, 0.0)),
    ("Sketch - Med", STYLIZED, "Medium graphite drawing", || pencil(4.0)),
    ("Sketch - Pencil", STYLIZED, "Fine pencil drawing", || pencil(2.0).exposure(0.05, 0.0, 0.0, 0.0)),
    ("Smooth and Flat", STYLIZED, "Flattened tones, smooth surfaces", || P::new().exposure(0.0, -0.2, 0.0, 0.0).adaptive(1.2, 8, -0.1, 0.0, 0.5, 0.5).details(-0.9, 1.0, 0.3, 3.0, 0.0).noise(0.9, 0.8)),
    ("So Simple", STYLIZED, "Graphic, posterized simplicity", || P::new().noise(0.8, 1.0).color(0.3, 0.3, 1.2, 0.0).posterize(0.45)),
    ("Solarized Dreams I", STYLIZED, "Dreamy partial solarization", || P::new().color(0.6, 0.2, 1.2, 0.0).solarize(0.6)),
    ("Solarized Dreams II", STYLIZED, "Strong solarization with glow", || P::new().color(0.9, 0.3, 1.3, 0.0).solarize(0.9).glow(0.2, 15.0)),

    // ===== Adjust 4 Preset List exclusives =================================
    ("Detail - Simple Pop", ADJUST4, "Detail with a light colour pop", || P::new().adaptive(0.3, 12, 0.1, 0.0, 0.3, 0.2).details(1.0, 1.2, 0.08, 1.5, 0.15).color(0.25, 0.05, 1.1, 0.0)),
    ("Detailed", ADJUST4, "Strong, clean detail", || P::new().adaptive(0.5, 12, 0.1, 0.0, 0.3, 0.2).details(1.6, 1.3, 0.07, 1.5, 0.25)),
    ("Enhance Contrast", ADJUST4, "Regional and global contrast", || P::new().adaptive(0.3, 10, 0.35, 0.0, 0.3, 0.3).curve_preset(CurveParams::medium_contrast())),
    ("Exposure Color Sketch", ADJUST4, "Equalized colour with sketched edges", || P::new().adaptive(1.3, 8, 0.2, 0.05, 0.5, 0.5).color(0.6, 0.15, 1.2, 0.0).sketch(0.3, 3.0, 1.0)),
    ("HDR - Pop", ADJUST4, "Classic HDR pop", || hdr(2.2, 0.5).color(0.6, 0.12, 1.2, 0.0)),
    ("Mild Details", ADJUST4, "Gentle detail enhancement", || P::new().details(0.6, 1.2, 0.08, 1.2, 0.1)),
    ("Recovery - Highlight", ADJUST4, "Strong highlight recovery", || P::new().exposure(-0.05, 0.0, 0.0, 0.0).adaptive(0.7, 10, 0.0, -0.08, 1.0, 0.0)),
    ("Recovery - Shadow", ADJUST4, "Strong shadow recovery", || P::new().exposure(0.03, 0.0, 0.0, 0.0).adaptive(0.8, 10, 0.0, 0.08, 0.0, 1.0)),
    ("Small Details", ADJUST4, "Ultra-fine texture enhancement", || P::new().details(1.4, 1.5, 0.04, 0.6, 0.3)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_are_unique() {
        let mut seen = HashSet::new();
        for (name, ..) in FACTORY {
            assert!(seen.insert(*name), "duplicate preset {name}");
        }
    }

    #[test]
    fn every_collection_entry_exists() {
        let names: HashSet<String> = get_factory_presets().into_iter().map(|p| p.name).collect();
        for col in get_collections() {
            assert!(!col.presets.is_empty(), "{} is empty", col.name);
            for p in col.presets {
                assert!(names.contains(p), "{} lists missing preset {p}", col.name);
            }
        }
    }

    #[test]
    fn all_presets_survive_sanitizing() {
        for p in get_factory_presets() {
            assert_eq!(p.params.sanitized(), p.params, "{} has out-of-range values", p.name);
        }
    }
}
