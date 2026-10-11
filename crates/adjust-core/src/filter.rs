use rayon::prelude::*;

/// Fast separable 2D box blur with O(1) cost per pixel, independent of `radius`.
/// Edges are handled by clamping to the nearest valid pixel.
///
/// Both passes run as cache-friendly horizontal running sums: the vertical pass is
/// performed on a transposed copy of the intermediate buffer.
pub fn box_filter(src: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || width == 0 || height == 0 {
        return src.to_vec();
    }

    let horizontal = box_rows(src, width, radius);
    let transposed = transpose(&horizontal, width, height);
    let vertical = box_rows(&transposed, height, radius);
    transpose(&vertical, height, width)
}

/// Running-sum box blur along each row of a `row_len`-wide buffer.
fn box_rows(src: &[f32], row_len: usize, radius: usize) -> Vec<f32> {
    let mut dst = vec![0.0; src.len()];
    let r = radius as isize;
    let last = row_len as isize - 1;
    let inv_len = 1.0 / (2 * radius + 1) as f32;

    dst.par_chunks_mut(row_len)
        .zip(src.par_chunks(row_len))
        .for_each(|(row_out, row_in)| {
            // Accumulate in f64 so long rows with large radii don't drift.
            let mut sum: f64 = (-r..=r)
                .map(|k| row_in[k.clamp(0, last) as usize] as f64)
                .sum();
            row_out[0] = (sum as f32) * inv_len;

            for (x, out) in row_out.iter_mut().enumerate().skip(1) {
                let add_x = (x as isize + r).min(last) as usize;
                let sub_x = (x as isize - r - 1).max(0) as usize;
                sum += row_in[add_x] as f64 - row_in[sub_x] as f64;
                *out = (sum as f32) * inv_len;
            }
        });

    dst
}

/// Transposes a `width x height` row-major buffer into a `height x width` one.
fn transpose(src: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut dst = vec![0.0; src.len()];
    dst.par_chunks_mut(height).enumerate().for_each(|(x, col)| {
        for (y, out) in col.iter_mut().enumerate() {
            *out = src[y * width + x];
        }
    });
    dst
}

/// Guided Filter (He, Sun, Tang 2010).
/// Fast edge-preserving filter that smooths textures while sharply retaining strong edges.
/// - `guidance`: Guide image (e.g. luminance Y)
/// - `src`: Input signal to smooth
/// - `radius`: Window radius (spatial scale)
/// - `eps`: Regularization parameter (smoothness / edge-sensitivity)
pub fn guided_filter(
    guidance: &[f32],
    src: &[f32],
    width: usize,
    height: usize,
    radius: usize,
    eps: f32,
) -> Vec<f32> {
    let size = width * height;

    // Mean of guidance I and input p
    let mean_i = box_filter(guidance, width, height, radius);
    let mean_p = box_filter(src, width, height, radius);

    // Cross correlation of I*I and I*p
    let mut ii = vec![0.0; size];
    let mut ip = vec![0.0; size];

    ii.par_iter_mut()
        .zip(ip.par_iter_mut())
        .enumerate()
        .for_each(|(idx, (val_ii, val_ip))| {
            let i = guidance[idx];
            let p = src[idx];
            *val_ii = i * i;
            *val_ip = i * p;
        });

    let corr_i = box_filter(&ii, width, height, radius);
    let corr_ip = box_filter(&ip, width, height, radius);

    // Variance of I and Covariance of (I, p)
    let mut a = vec![0.0; size];
    let mut b = vec![0.0; size];

    a.par_iter_mut()
        .zip(b.par_iter_mut())
        .enumerate()
        .for_each(|(idx, (val_a, val_b))| {
            let m_i = mean_i[idx];
            let m_p = mean_p[idx];
            let var_i = corr_i[idx] - m_i * m_i;
            let cov_ip = corr_ip[idx] - m_i * m_p;

            let coeff_a = cov_ip / (var_i + eps);
            let coeff_b = m_p - coeff_a * m_i;

            *val_a = coeff_a;
            *val_b = coeff_b;
        });

    // Mean of coefficients a and b
    let mean_a = box_filter(&a, width, height, radius);
    let mean_b = box_filter(&b, width, height, radius);

    // Final output: q = mean_a * I + mean_b
    let mut q = vec![0.0; size];
    q.par_iter_mut()
        .enumerate()
        .for_each(|(idx, val_q)| {
            *val_q = mean_a[idx] * guidance[idx] + mean_b[idx];
        });

    q
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_box(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
        let r = r as isize;
        let mut out = vec![0.0; w * h];
        for y in 0..h as isize {
            for x in 0..w as isize {
                let mut sum = 0.0;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let sx = (x + dx).clamp(0, w as isize - 1) as usize;
                        let sy = (y + dy).clamp(0, h as isize - 1) as usize;
                        sum += src[sy * w + sx];
                    }
                }
                out[(y as usize) * w + x as usize] = sum / ((2 * r + 1) * (2 * r + 1)) as f32;
            }
        }
        out
    }

    #[test]
    fn box_filter_matches_naive_reference() {
        for &(w, h, r) in &[(17, 9, 1), (13, 21, 4), (5, 3, 12), (1, 7, 2), (8, 1, 3)] {
            let src: Vec<f32> = (0..w * h).map(|i| ((i * 37) % 101) as f32 / 100.0).collect();
            let fast = box_filter(&src, w, h, r);
            let slow = naive_box(&src, w, h, r);
            for (a, b) in fast.iter().zip(&slow) {
                assert!((a - b).abs() < 1e-4, "{w}x{h} r={r}: {a} vs {b}");
            }
        }
    }
}
