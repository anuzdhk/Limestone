use crate::image_buffer::RgbImageF32;
use crate::params::{CurveParams, CurvePoint};
use rayon::prelude::*;

pub struct HistogramData {
    pub r: [u32; 256],
    pub g: [u32; 256],
    pub b: [u32; 256],
    pub lum: [u32; 256],
}

pub fn compute_histogram(img: &RgbImageF32) -> HistogramData {
    let mut hist = HistogramData {
        r: [0; 256],
        g: [0; 256],
        b: [0; 256],
        lum: [0; 256],
    };

    let size = img.width * img.height;
    for i in 0..size {
        let r_bin = (img.r[i].clamp(0.0, 1.0) * 255.0).round() as usize;
        let g_bin = (img.g[i].clamp(0.0, 1.0) * 255.0).round() as usize;
        let b_bin = (img.b[i].clamp(0.0, 1.0) * 255.0).round() as usize;
        let lum = 0.2126 * img.r[i] + 0.7152 * img.g[i] + 0.0722 * img.b[i];
        let l_bin = (lum.clamp(0.0, 1.0) * 255.0).round() as usize;

        hist.r[r_bin] += 1;
        hist.g[g_bin] += 1;
        hist.b[b_bin] += 1;
        hist.lum[l_bin] += 1;
    }

    hist
}

/// Generates a 1024-entry LUT using natural cubic spline or monotonic interpolation.
pub fn generate_curve_lut(points: &[CurvePoint]) -> [f32; 1024] {
    let mut lut = [0.0; 1024];

    if points.len() < 2 {
        for (i, val) in lut.iter_mut().enumerate() {
            *val = i as f32 / 1023.0;
        }
        return lut;
    }

    // Non-finite points would poison the interpolation (and panic a partial_cmp sort).
    let mut sorted: Vec<CurvePoint> = points
        .iter()
        .filter(|p| p.x.is_finite() && p.y.is_finite())
        .cloned()
        .collect();
    if sorted.len() < 2 {
        for (i, val) in lut.iter_mut().enumerate() {
            *val = i as f32 / 1023.0;
        }
        return lut;
    }
    sorted.sort_by(|a, b| a.x.total_cmp(&b.x));

    for (i, val) in lut.iter_mut().enumerate() {
        let x = i as f32 / 1023.0;

        if x <= sorted[0].x {
            *val = sorted[0].y;
            continue;
        }
        if x >= sorted.last().unwrap().x {
            *val = sorted.last().unwrap().y;
            continue;
        }

        // Piecewise cubic Hermite interpolation between control points
        let mut idx = 0;
        while idx < sorted.len() - 1 && sorted[idx + 1].x < x {
            idx += 1;
        }

        let p0 = if idx > 0 { &sorted[idx - 1] } else { &sorted[idx] };
        let p1 = &sorted[idx];
        let p2 = &sorted[idx + 1];
        let p3 = if idx + 2 < sorted.len() { &sorted[idx + 2] } else { p2 };

        let dx = (p2.x - p1.x).max(1e-5);
        let t = (x - p1.x) / dx;

        // Catmull-Rom tangent approximations
        let m1 = (p2.y - p0.y) / (p2.x - p0.x).max(1e-5);
        let m2 = (p3.y - p1.y) / (p3.x - p1.x).max(1e-5);

        let t2 = t * t;
        let t3 = t2 * t;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        let y = h00 * p1.y + h10 * dx * m1 + h01 * p2.y + h11 * dx * m2;
        *val = y.clamp(0.0, 1.0);
    }

    lut
}

/// Master curve followed by optional per-channel curves (empty channel = identity).
pub fn apply_curve(img: &mut RgbImageF32, params: &CurveParams) {
    if !params.enabled {
        return;
    }
    let active = |pts: &[CurvePoint]| pts.len() >= 2;
    if ![&params.points, &params.red, &params.green, &params.blue]
        .iter()
        .any(|c| active(c))
    {
        return;
    }

    // Compose master + channel curve into one LUT per channel.
    let master = generate_curve_lut(&params.points);
    let compose = |channel: &[CurvePoint]| -> [f32; 1024] {
        if !active(channel) {
            return master;
        }
        let ch = generate_curve_lut(channel);
        let mut out = [0.0; 1024];
        for (o, &m) in out.iter_mut().zip(master.iter()) {
            *o = sample_lut(&ch, m);
        }
        out
    };
    let (lut_r, lut_g, lut_b) = (compose(&params.red), compose(&params.green), compose(&params.blue));

    img.r
        .par_iter_mut()
        .zip(img.g.par_iter_mut())
        .zip(img.b.par_iter_mut())
        .for_each(|((r, g), b)| {
            *r = sample_lut(&lut_r, *r);
            *g = sample_lut(&lut_g, *g);
            *b = sample_lut(&lut_b, *b);
        });
}

#[inline(always)]
fn sample_lut(lut: &[f32; 1024], val: f32) -> f32 {
    let idx = (val.clamp(0.0, 1.0) * 1023.0).round() as usize;
    lut[idx.min(1023)]
}
