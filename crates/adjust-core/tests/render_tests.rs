use adjust_core::{
    get_factory_presets, process_image, process_region, AdjustParams, CurvePoint, Preset,
    RgbImageF32,
};
use image::{Rgb, RgbImage};

fn create_test_pattern(width: u32, height: u32) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;

            // Generate high dynamic range patterns: gradients, high frequencies, shadows, skies
            let r = ((fx * 255.0) as u8).wrapping_add((fy * 100.0) as u8);
            let g = (((1.0 - fy) * 200.0) as u8).wrapping_add((fx * 80.0) as u8);
            let b = ((fx * fy) * 255.0) as u8;

            // High frequency checkerboard pattern in a quadrant to test micro-details
            let checker = if (x / 8 + y / 8) % 2 == 0 { 40 } else { 0 };

            img.put_pixel(
                x,
                y,
                Rgb([
                    r.saturating_add(checker),
                    g.saturating_add(checker),
                    b.saturating_add(checker),
                ]),
            );
        }
    }
    img
}

#[test]
fn test_all_presets_render_valid_pixels() {
    let test_img = create_test_pattern(256, 256);
    let f_img = RgbImageF32::from_rgb8(&test_img);

    for preset in get_factory_presets() {
        let out = process_image(&f_img, &preset.params);
        assert_eq!(out.width, 256);
        assert_eq!(out.height, 256);

        // Verify all floats are finite and inside [0.0, 1.0]
        for i in 0..(out.width * out.height) {
            assert!(out.r[i].is_finite(), "Preset {} generated non-finite R", preset.name);
            assert!(out.g[i].is_finite(), "Preset {} generated non-finite G", preset.name);
            assert!(out.b[i].is_finite(), "Preset {} generated non-finite B", preset.name);
            assert!(out.r[i] >= 0.0 && out.r[i] <= 1.0, "Preset {} R out of bounds", preset.name);
            assert!(out.g[i] >= 0.0 && out.g[i] <= 1.0, "Preset {} G out of bounds", preset.name);
            assert!(out.b[i] >= 0.0 && out.b[i] <= 1.0, "Preset {} B out of bounds", preset.name);
        }

        let rgb8 = out.to_rgb8();
        assert_eq!(rgb8.width(), 256);
        assert_eq!(rgb8.height(), 256);
    }
}

fn pattern_f32(w: u32, h: u32) -> RgbImageF32 {
    RgbImageF32::from_rgb8(&create_test_pattern(w, h))
}

fn max_abs_diff(a: &RgbImageF32, b: &RgbImageF32) -> f32 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    a.r.iter()
        .zip(&b.r)
        .chain(a.g.iter().zip(&b.g))
        .chain(a.b.iter().zip(&b.b))
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// A tile rendered with `process_region` must match the same area of a full render,
/// including position-dependent effects (vignette, border, grain).
#[test]
fn test_region_matches_full_render() {
    let img = pattern_f32(300, 200);
    let mut presets: Vec<AdjustParams> =
        get_factory_presets().into_iter().map(|p| p.params).collect();

    let mut framed = AdjustParams::default();
    framed.finishing.vignette.amount = -0.8;
    framed.finishing.grain.amount = 0.6;
    framed.finishing.border.enabled = true;
    framed.finishing.border.thickness = 0.05;
    framed.adaptive_exposure.adaptive_exposure = 1.5;
    presets.push(framed);

    for params in &presets {
        let full = process_image(&img, params);
        for &(x0, y0, w, h) in &[(0, 0, 60, 40), (120, 70, 90, 50), (240, 150, 60, 50)] {
            let tile = process_region(&img, params, x0, y0, w, h);
            let expected = full.crop(x0, y0, w, h);
            let diff = max_abs_diff(&tile, &expected);
            assert!(diff < 2e-3, "tile ({x0},{y0},{w},{h}) differs by {diff}");
        }
    }
}

/// The preview (downsampled input) should look like a downsample of the full render.
#[test]
fn test_preview_matches_full_resolution() {
    let full_in = pattern_f32(800, 600);
    let preview_in = full_in.downsample(200);

    let mut params = AdjustParams::default();
    params.details.strength = 1.0;
    params.details.radius = 10.0;
    params.noise.suppression = 0.5;
    params.adaptive_exposure.adaptive_exposure = 1.0;

    let from_full = process_image(&full_in, &params).downsample(200);
    let from_preview = process_image(&preview_in, &params);
    let mean_diff = from_full
        .r
        .iter()
        .zip(&from_preview.r)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / from_full.r.len() as f32;
    assert!(mean_diff < 0.03, "preview deviates from export by {mean_diff}");
}

#[test]
fn test_hostile_params_do_not_panic() {
    let mut params = AdjustParams::default();
    params.finishing.border.enabled = true;
    params.finishing.border.thickness = 0.9;
    params.finishing.vignette.amount = -1.0;
    params.finishing.grain.amount = f32::NAN;
    params.details.strength = f32::INFINITY;
    params.curve.enabled = true;
    params.curve.points.push(CurvePoint { x: f32::NAN, y: 0.5 });

    for &(w, h) in &[(1, 1), (1, 9), (9, 1), (3, 3), (64, 32)] {
        let out = process_image(&pattern_f32(w, h), &params);
        assert!(out.r.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
    }
    // Empty images are a no-op rather than a panic.
    let empty = RgbImageF32::new(0, 0);
    assert_eq!(process_image(&empty, &params).width, 0);
}

#[test]
fn test_partial_preset_json_uses_defaults() {
    let json = r#"{"name":"Old","category":"Custom","description":"",
        "params":{"basic_exposure":{"brightness":0.3}}}"#;
    let preset = Preset::from_json(json).expect("partial preset should load");
    assert_eq!(preset.params.basic_exposure.brightness, 0.3);
    assert!(preset.params.basic_exposure.enabled);
    assert_eq!(preset.params.details, AdjustParams::default().details);
}

#[test]
fn test_save_keeps_16_bit_and_alpha() {
    let dir = std::env::temp_dir().join(format!("limestone_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let mut img = RgbImageF32::new(4, 4);
    img.r[0] = 1000.0 / 65535.0;
    img.alpha = Some(vec![0.25; 16]);
    img.high_bit_depth = true;

    let path = dir.join("out.png");
    img.save(&path).unwrap();
    let back = RgbImageF32::open(&path).unwrap();
    assert!(back.high_bit_depth);
    assert!((back.r[0] - img.r[0]).abs() < 1e-5, "16-bit precision lost");
    assert!((back.alpha.as_ref().unwrap()[0] - 0.25).abs() < 1e-3, "alpha lost");

    let jpg = dir.join("out.jpg");
    img.save(&jpg).unwrap(); // JPEG falls back to 8-bit RGB instead of erroring
    let _ = std::fs::remove_dir_all(&dir);
}
