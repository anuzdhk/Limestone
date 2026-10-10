use adjust_core::{process_image, AdjustParams, RgbImageF32};
use eframe::egui;

/// Tiled view renderer that extracts and processes only the visible window at full 100% resolution
/// when the user zooms in, eliminating proxy pixelation without stalling the main UI loop.
pub struct TiledViewer {
    tile_texture: Option<egui::TextureHandle>,
    last_tile_rect: Option<egui::Rect>,
    last_params: Option<AdjustParams>,
}

impl TiledViewer {
    pub fn new() -> Self {
        Self {
            tile_texture: None,
            last_tile_rect: None,
            last_params: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.last_tile_rect = None;
        self.last_params = None;
    }

    /// Renders or updates high-resolution subregion tile matching visible canvas bounds
    pub fn get_or_render_full_res_tile(
        &mut self,
        ctx: &egui::Context,
        original_full: &RgbImageF32,
        params: &AdjustParams,
        canvas_visible_rect: egui::Rect,
        img_canvas_rect: egui::Rect,
    ) -> Option<(&egui::TextureHandle, egui::Rect)> {
        // Find intersection between canvas viewport and displayed photo
        let visible_box = canvas_visible_rect.intersect(img_canvas_rect);
        if visible_box.width() <= 10.0 || visible_box.height() <= 10.0 {
            return None;
        }

        // Map visible box back to normalized coordinates [0.0, 1.0] in original image
        let u0 = ((visible_box.min.x - img_canvas_rect.min.x) / img_canvas_rect.width()).clamp(0.0, 1.0);
        let v0 = ((visible_box.min.y - img_canvas_rect.min.y) / img_canvas_rect.height()).clamp(0.0, 1.0);
        let u1 = ((visible_box.max.x - img_canvas_rect.min.x) / img_canvas_rect.width()).clamp(0.0, 1.0);
        let v1 = ((visible_box.max.y - img_canvas_rect.min.y) / img_canvas_rect.height()).clamp(0.0, 1.0);

        let img_w = original_full.width;
        let img_h = original_full.height;

        let px0 = ((u0 * img_w as f32).floor() as usize).min(img_w.saturating_sub(1));
        let py0 = ((v0 * img_h as f32).floor() as usize).min(img_h.saturating_sub(1));
        let px1 = ((u1 * img_w as f32).ceil() as usize).min(img_w);
        let py1 = ((v1 * img_h as f32).ceil() as usize).min(img_h);

        let tile_w = px1.saturating_sub(px0).max(1);
        let tile_h = py1.saturating_sub(py0).max(1);

        // Cap tile computation to max 1920x1080 per frame for smooth 60fps responsiveness
        if tile_w > 1920 || tile_h > 1080 {
            return None;
        }

        let tile_norm_rect = egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1));

        // Check if we already computed this tile
        if let (Some(last_r), Some(last_p)) = (&self.last_tile_rect, &self.last_params) {
            if last_p == params && (last_r.min - tile_norm_rect.min).length() < 0.01 && (last_r.max - tile_norm_rect.max).length() < 0.01 {
                return self.tile_texture.as_ref().map(|t| (t, tile_norm_rect));
            }
        }

        // Crop subregion from original image buffer
        let mut sub_img = RgbImageF32::new(tile_w, tile_h);
        for dy in 0..tile_h {
            let sy = py0 + dy;
            let src_row = sy * img_w;
            let dst_row = dy * tile_w;
            for dx in 0..tile_w {
                let sx = px0 + dx;
                let src_idx = src_row + sx;
                let dst_idx = dst_row + dx;
                sub_img.r[dst_idx] = original_full.r[src_idx];
                sub_img.g[dst_idx] = original_full.g[src_idx];
                sub_img.b[dst_idx] = original_full.b[src_idx];
            }
        }

        // Process subregion tile
        let processed_tile = process_image(&sub_img, params);
        let rgb8 = processed_tile.to_rgb8();
        let color_img = egui::ColorImage::from_rgb([tile_w, tile_h], rgb8.as_raw());

        self.tile_texture = Some(ctx.load_texture("high_res_view_tile", color_img, egui::TextureOptions::LINEAR));
        self.last_tile_rect = Some(tile_norm_rect);
        self.last_params = Some(params.clone());

        self.tile_texture.as_ref().map(|t| (t, tile_norm_rect))
    }

    pub fn last_tile_rect(&self) -> Option<egui::Rect> {
        self.last_tile_rect
    }
}
