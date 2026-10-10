/// Color conversions and utilities for gamma, HSV, and perceptual luminance.

#[inline(always)]
pub fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
pub fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert RGB [0.0, 1.0] to HSV [0.0, 1.0]
#[inline(always)]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let v = max;
    let s = if max > 1e-6 { delta / max } else { 0.0 };

    let mut h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / delta) + 2.0
    } else {
        ((r - g) / delta) + 4.0
    };

    h /= 6.0;
    if h < 0.0 {
        h += 1.0;
    }

    (h, s, v)
}

/// Convert HSV [0.0, 1.0] to RGB [0.0, 1.0]
#[inline(always)]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    if s <= 1e-6 {
        return (v, v, v);
    }

    let h6 = (h % 1.0) * 6.0;
    let i = h6.floor() as i32;
    let f = h6 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
