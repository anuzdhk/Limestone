use adjust_core::{get_factory_presets, process_image, RgbImageF32};
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
