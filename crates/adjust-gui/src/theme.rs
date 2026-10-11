use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. DYNAMIC THEME CONFIGURATION
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ThemeMode {
    DarkStudio,
    LightStudio,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum AccentTheme {
    ElectricCyan, // #00D2FF
    VibrantAmber, // #FFAA00
    EmeraldGreen, // #00E676
    RoyalPurple,  // #B388FF
    CrimsonRed,   // #FF5252
}

impl AccentTheme {
    pub fn color(&self) -> Color32 {
        match self {
            AccentTheme::ElectricCyan => Color32::from_rgb(0, 210, 255),
            AccentTheme::VibrantAmber => Color32::from_rgb(255, 170, 0),
            AccentTheme::EmeraldGreen => Color32::from_rgb(0, 230, 118),
            AccentTheme::RoyalPurple => Color32::from_rgb(179, 136, 255),
            AccentTheme::CrimsonRed => Color32::from_rgb(255, 82, 82),
        }
    }

    pub fn hover_color(&self) -> Color32 {
        let c = self.color();
        Color32::from_rgb(
            c.r().saturating_add(30),
            c.g().saturating_add(30),
            c.b().saturating_add(30),
        )
    }

    pub fn pressed_color(&self) -> Color32 {
        let c = self.color();
        Color32::from_rgb(
            c.r().saturating_sub(40),
            c.g().saturating_sub(40),
            c.b().saturating_sub(40),
        )
    }

    pub fn name(&self) -> &'static str {
        match self {
            AccentTheme::ElectricCyan => "Cyan",
            AccentTheme::VibrantAmber => "Amber",
            AccentTheme::EmeraldGreen => "Emerald",
            AccentTheme::RoyalPurple => "Purple",
            AccentTheme::CrimsonRed => "Crimson",
        }
    }
}

// ============================================================================
// 2. DEFAULT COLOR TOKENS
// ============================================================================

pub const BG_CANVAS: Color32 = Color32::from_rgb(11, 12, 15); // #0B0C0F
pub const SURFACE_PANEL: Color32 = Color32::from_rgb(17, 19, 24); // #111318
pub const SURFACE_CARD: Color32 = Color32::from_rgb(23, 26, 33); // #171A21
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(31, 35, 44); // #1F232C

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(42, 47, 58); // #2A2F3A

pub const ACCENT: Color32 = Color32::from_rgb(0, 210, 255);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(51, 220, 255);
pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(0, 168, 204);
pub const ACCENT_MUTED: Color32 = Color32::from_rgba_premultiplied(0, 30, 36, 36);
pub const ACCENT_STAR: Color32 = Color32::from_rgb(255, 200, 50);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 234, 240);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(154, 163, 178);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(120, 128, 142);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(91, 100, 114);

pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const STATUS_WARNING: Color32 = Color32::from_rgb(241, 196, 15);
pub const STATUS_ERROR: Color32 = Color32::from_rgb(231, 76, 60);

// ============================================================================
// 3. METRICS & SPACING (4px Base Grid & Standard Heights)
// ============================================================================

pub const SPACE_4: f32 = 4.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_24: f32 = 24.0;

pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(6);
pub const RADIUS_CARD: CornerRadius = CornerRadius::same(10);

pub const HEIGHT_TOOLBAR: f32 = 48.0;
pub const HEIGHT_STATUSBAR: f32 = 24.0;
pub const WIDTH_SIDEBAR_LEFT: f32 = 260.0;
pub const WIDTH_SIDEBAR_RIGHT: f32 = 330.0;

// Standard button dimensions
pub const BTN_HEIGHT_SM: f32 = 24.0;
pub const BTN_HEIGHT_MD: f32 = 28.0;

// ============================================================================
// 4. THEME APPLICATION WITH DYNAMIC CUSTOMIZATION
// ============================================================================

pub fn apply_theme_configured(
    ctx: &egui::Context,
    mode: ThemeMode,
    accent: AccentTheme,
    ui_scale: f32,
) {
    let mut visuals = match mode {
        ThemeMode::DarkStudio => egui::Visuals::dark(),
        ThemeMode::LightStudio => egui::Visuals::light(),
    };

    let accent_col = accent.color();
    let accent_hover = accent.hover_color();
    let accent_pressed = accent.pressed_color();
    let accent_muted = Color32::from_rgba_premultiplied(
        (accent_col.r() as f32 * 0.15) as u8,
        (accent_col.g() as f32 * 0.15) as u8,
        (accent_col.b() as f32 * 0.15) as u8,
        38,
    );

    match mode {
        ThemeMode::DarkStudio => {
            visuals.override_text_color = Some(TEXT_PRIMARY);
            visuals.panel_fill = SURFACE_PANEL;
            visuals.window_fill = SURFACE_CARD;
            visuals.extreme_bg_color = BG_CANVAS;
            visuals.faint_bg_color = SURFACE_CARD;

            visuals.widgets.inactive.bg_fill = SURFACE_CARD;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
            visuals.widgets.inactive.corner_radius = RADIUS_CONTROL;
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

            visuals.widgets.hovered.bg_fill = SURFACE_RAISED;
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.hovered.corner_radius = RADIUS_CONTROL;
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

            visuals.widgets.active.bg_fill = accent_pressed;
            visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.active.corner_radius = RADIUS_CONTROL;
            visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

            visuals.widgets.open.bg_fill = SURFACE_RAISED;
            visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.open.corner_radius = RADIUS_CONTROL;
            visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

            visuals.widgets.noninteractive.bg_fill = SURFACE_PANEL;
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
            visuals.widgets.noninteractive.corner_radius = RADIUS_CONTROL;
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_SECONDARY);

            visuals.selection.bg_fill = accent_muted;
            visuals.selection.stroke = Stroke::new(1.0_f32, accent_col);

            visuals.window_corner_radius = RADIUS_CARD;
            visuals.menu_corner_radius = RADIUS_CONTROL;
            visuals.window_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
        }
        ThemeMode::LightStudio => {
            let bg_light = Color32::from_rgb(222, 226, 234);     // Canvas neutral photo gray
            let panel_light = Color32::from_rgb(244, 246, 250);  // Panels light surface
            let card_light = Color32::from_rgb(255, 255, 255);   // Card elevated surface
            let raised_light = Color32::from_rgb(235, 239, 245); // Hover / raised surface
            let border_light = Color32::from_rgb(208, 214, 226); // Crisp subtle border
            let text_dark = Color32::from_rgb(18, 22, 30);       // High-contrast primary text
            let text_secondary_dark = Color32::from_rgb(76, 84, 100);

            visuals.override_text_color = Some(text_dark);
            visuals.panel_fill = panel_light;
            visuals.window_fill = card_light;
            visuals.extreme_bg_color = bg_light;
            visuals.faint_bg_color = card_light;

            visuals.widgets.inactive.bg_fill = card_light;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border_light);
            visuals.widgets.inactive.corner_radius = RADIUS_CONTROL;
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text_dark);

            visuals.widgets.hovered.bg_fill = raised_light;
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.hovered.corner_radius = RADIUS_CONTROL;
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, text_dark);

            visuals.widgets.active.bg_fill = accent_hover;
            visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.active.corner_radius = RADIUS_CONTROL;
            visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

            visuals.widgets.open.bg_fill = raised_light;
            visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, accent_col);
            visuals.widgets.open.corner_radius = RADIUS_CONTROL;
            visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, text_dark);

            visuals.widgets.noninteractive.bg_fill = panel_light;
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border_light);
            visuals.widgets.noninteractive.corner_radius = RADIUS_CONTROL;
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text_secondary_dark);

            visuals.selection.bg_fill = accent_muted;
            visuals.selection.stroke = Stroke::new(1.0_f32, accent_col);

            visuals.window_corner_radius = RADIUS_CARD;
            visuals.menu_corner_radius = RADIUS_CONTROL;
            visuals.window_stroke = Stroke::new(1.0_f32, border_light);
        }
    }

    ctx.set_visuals(visuals);

    // Apply scaling and exact font hierarchy
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Name("Title".into()),
        FontId::new(20.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.0, FontFamily::Monospace),
    );

    // Consistent button and widget padding
    style.spacing.item_spacing = egui::vec2(SPACE_8, SPACE_6);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(12);

    ctx.set_style(style);
    ctx.set_zoom_factor(ui_scale.clamp(0.75, 1.75));
}

// ============================================================================
// 5. DYNAMIC THEME-AWARE COLOR RETRIEVAL HELPERS
// ============================================================================

pub fn theme_text_primary(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        TEXT_PRIMARY
    } else {
        Color32::from_rgb(18, 22, 30)
    }
}

pub fn theme_text_secondary(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        TEXT_SECONDARY
    } else {
        Color32::from_rgb(76, 84, 100)
    }
}

pub fn theme_text_muted(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        TEXT_MUTED
    } else {
        Color32::from_rgb(112, 122, 138)
    }
}

pub fn theme_surface_panel(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        SURFACE_PANEL
    } else {
        Color32::from_rgb(244, 246, 250)
    }
}

pub fn theme_surface_card(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        SURFACE_CARD
    } else {
        Color32::from_rgb(255, 255, 255)
    }
}


pub fn theme_border(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        BORDER_SUBTLE
    } else {
        Color32::from_rgb(208, 214, 226)
    }
}

pub fn theme_canvas_bg(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        BG_CANVAS
    } else {
        Color32::from_rgb(222, 226, 234)
    }
}

