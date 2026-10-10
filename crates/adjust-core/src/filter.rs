use rayon::prelude::*;

/// Fast separable 2D box blur with O(1) complexity per pixel.
/// Handles edge conditions with clamped boundaries.
pub fn box_filter(src: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    if radius == 0 {
        return src.to_vec();
    }

    let size = width * height;
    let mut temp = vec![0.0; size];
    let mut dst = vec![0.0; size];

    let r = radius as isize;

    // 1. Horizontal pass across all rows in parallel
    temp.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row_out)| {
            let row_in = &src[y * width..(y + 1) * width];

            // Initialize running window sum with clamp boundary on left
            let mut sum: f32 = 0.0;
            for k in -r..=r {
                let clamped_x = k.clamp(0, width as isize - 1) as usize;
                sum += row_in[clamped_x];
            }

            let win_len = (2 * r + 1) as f32;
            row_out[0] = sum / win_len;

            // Slide window across row
            for x in 1..width {
                let add_x = (x as isize + r).clamp(0, width as isize - 1) as usize;
                let sub_x = (x as isize - r - 1).clamp(0, width as isize - 1) as usize;
                sum += row_in[add_x] - row_in[sub_x];
                row_out[x] = sum / win_len;
            }
        });

    // 2. Vertical pass across all columns in parallel
    // Process columns in blocks or transpose for cache efficiency
    dst.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row_out)| {
            let y_i = y as isize;
            for x in 0..width {
                let mut sum: f32 = 0.0;
                for k in -r..=r {
                    let clamped_y = (y_i + k).clamp(0, height as isize - 1) as usize;
                    sum += temp[clamped_y * width + x];
                }
                row_out[x] = sum / (2 * r + 1) as f32;
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
