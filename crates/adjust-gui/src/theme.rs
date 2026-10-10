use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

// ============================================================================
// 1. COLOR TOKENS (Pro Dark Photo Editor System)
// ============================================================================

/// Neutral canvas background: Near-black neutral without color tint
/// ensures accurate color and exposure judgment for photographs.
pub const BG_CANVAS: Color32 = Color32::from_rgb(11, 12, 15); // #0B0C0F

/// Surface 1: Sidebars, top toolbar, and bottom status bar
pub const SURFACE_PANEL: Color32 = Color32::from_rgb(17, 19, 24); // #111318

/// Surface 2: Cards, containers, inspector backdrop, curve canvas
pub const SURFACE_CARD: Color32 = Color32::from_rgb(23, 26, 33); // #171A21

/// Surface 3: Raised controls, hovered rows, active inputs
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(31, 35, 44); // #1F232C

/// Borders: 1px subtle boundary divider used sparingly
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(42, 47, 58); // #2A2F3A
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0, 210, 255); // #00D2FF

/// Primary Accent: Electric Cyan
pub const ACCENT: Color32 = Color32::from_rgb(0, 210, 255); // #00D2FF
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(51, 220, 255); // #33DCFF
pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(0, 168, 204); // #00A8CC
pub const ACCENT_MUTED: Color32 = Color32::from_rgba_premultiplied(0, 30, 36, 36);
pub const ACCENT_STAR: Color32 = Color32::from_rgb(255, 200, 50);

/// Text colors (WCAG AA Compliant contrast against dark surfaces)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 234, 240); // #E8EAF0
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(154, 163, 178); // #9AA3B2
pub const TEXT_MUTED: Color32 = Color32::from_rgb(120, 128, 142);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(91, 100, 114); // #5B6472

/// Semantic status colors
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const STATUS_WARNING: Color32 = Color32::from_rgb(241, 196, 15);
pub const STATUS_ERROR: Color32 = Color32::from_rgb(231, 76, 60);

// ============================================================================
// 2. METRICS & SPACING (4px Base Grid)
// ============================================================================

pub const SPACE_4: f32 = 4.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_24: f32 = 24.0;

pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(6);
pub const RADIUS_CARD: CornerRadius = CornerRadius::same(10);
pub const RADIUS_PILL: CornerRadius = CornerRadius::same(14);

pub const HEIGHT_TOOLBAR: f32 = 48.0;
pub const HEIGHT_STATUSBAR: f32 = 24.0;
pub const WIDTH_SIDEBAR_LEFT: f32 = 260.0;
pub const WIDTH_SIDEBAR_RIGHT: f32 = 330.0;

// Legacy aliases
pub const TOP_BAR_HEIGHT: f32 = HEIGHT_TOOLBAR;
pub const STATUS_BAR_HEIGHT: f32 = HEIGHT_STATUSBAR;
pub const SIDEBAR_LEFT_WIDTH: f32 = WIDTH_SIDEBAR_LEFT;
pub const SIDEBAR_RIGHT_WIDTH: f32 = WIDTH_SIDEBAR_RIGHT;

// ============================================================================
// 3. THEME APPLICATION
// ============================================================================

/// Applies the complete visual styling to egui Context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = SURFACE_PANEL;
    visuals.window_fill = SURFACE_CARD;
    visuals.extreme_bg_color = BG_CANVAS;
    visuals.faint_bg_color = SURFACE_CARD;

    // Controls: Inactive
    visuals.widgets.inactive.bg_fill = SURFACE_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.inactive.corner_radius = RADIUS_CONTROL;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    // Controls: Hovered
    visuals.widgets.hovered.bg_fill = SURFACE_RAISED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.corner_radius = RADIUS_CONTROL;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Controls: Active
    visuals.widgets.active.bg_fill = ACCENT_PRESSED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.corner_radius = RADIUS_CONTROL;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Controls: Open
    visuals.widgets.open.bg_fill = SURFACE_RAISED;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.open.corner_radius = RADIUS_CONTROL;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Controls: Non-interactive
    visuals.widgets.noninteractive.bg_fill = SURFACE_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.noninteractive.corner_radius = RADIUS_CONTROL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_SECONDARY);

    // Selection
    visuals.selection.bg_fill = ACCENT_MUTED;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);

    // Window & Popup
    visuals.window_corner_radius = RADIUS_CARD;
    visuals.menu_corner_radius = RADIUS_CONTROL;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);

    ctx.set_visuals(visuals);

    // Configure exact font hierarchy (11 caption, 13 body, 15 section title, 20 panel title)
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

    style.spacing.item_spacing = egui::vec2(SPACE_8, SPACE_6);
    style.spacing.window_margin = egui::Margin::same(12);

    ctx.set_style(style);
}
