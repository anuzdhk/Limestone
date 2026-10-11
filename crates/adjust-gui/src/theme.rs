//! Apple-style visual language: system palettes, accent colours, typography and metrics.
//!
//! Widgets read colours from the active [`Palette`] (see [`pal`]) instead of constants,
//! so light/dark switching and accent changes apply everywhere at once.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};
use serde::{Deserialize, Serialize};

// ============================================================================
// Theme preferences
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub enum ThemeMode {
    /// Follow the operating system's light/dark setting.
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::System => "Automatic",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

/// macOS system accent colours.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub enum Accent {
    #[default]
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Graphite,
}

impl Accent {
    pub const ALL: [Accent; 8] = [
        Accent::Blue,
        Accent::Purple,
        Accent::Pink,
        Accent::Red,
        Accent::Orange,
        Accent::Yellow,
        Accent::Green,
        Accent::Graphite,
    ];

    pub fn color(&self, dark: bool) -> Color32 {
        let (light, dark_c) = match self {
            Accent::Blue => ((0, 122, 255), (10, 132, 255)),
            Accent::Purple => ((175, 82, 222), (191, 90, 242)),
            Accent::Pink => ((255, 45, 85), (255, 55, 95)),
            Accent::Red => ((255, 59, 48), (255, 69, 58)),
            Accent::Orange => ((255, 149, 0), (255, 159, 10)),
            Accent::Yellow => ((230, 180, 0), (255, 214, 10)),
            Accent::Green => ((40, 180, 70), (48, 209, 88)),
            Accent::Graphite => ((142, 142, 147), (152, 152, 157)),
        };
        let (r, g, b) = if dark { dark_c } else { light };
        Color32::from_rgb(r, g, b)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Accent::Blue => "Blue",
            Accent::Purple => "Purple",
            Accent::Pink => "Pink",
            Accent::Red => "Red",
            Accent::Orange => "Orange",
            Accent::Yellow => "Yellow",
            Accent::Green => "Green",
            Accent::Graphite => "Graphite",
        }
    }
}

// ============================================================================
// Palette
// ============================================================================

/// Resolved colours for the current appearance.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub dark: bool,
    pub accent: Color32,
    /// Text/icons drawn on top of `accent`.
    pub on_accent: Color32,
    pub window: Color32,
    pub sidebar: Color32,
    pub toolbar: Color32,
    pub canvas: Color32,
    /// Grouped "card" background inside panels.
    pub card: Color32,
    /// Resting fill for buttons / segmented controls / fields.
    pub control: Color32,
    pub control_hover: Color32,
    /// Selected segment of a segmented control.
    pub segment_selected: Color32,
    pub separator: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    /// Translucent accent used for sidebar selection.
    pub selection: Color32,
    pub track: Color32,
    pub knob: Color32,
    pub shadow: Color32,
    pub star: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

impl Palette {
    pub fn new(dark: bool, accent: Accent) -> Self {
        let a = accent.color(dark);
        let on_accent = if accent == Accent::Yellow { Color32::BLACK } else { Color32::WHITE };
        if dark {
            Self {
                dark,
                accent: a,
                on_accent,
                window: rgb(30, 30, 32),
                sidebar: rgb(40, 40, 43),
                toolbar: rgb(44, 44, 47),
                canvas: rgb(22, 22, 24),
                card: rgb(50, 50, 54),
                control: rgb(64, 64, 68),
                control_hover: rgb(78, 78, 83),
                segment_selected: rgb(99, 99, 102),
                separator: Color32::from_white_alpha(22),
                text: rgb(245, 245, 247),
                text_secondary: rgb(160, 160, 166),
                text_tertiary: rgb(110, 110, 116),
                selection: with_alpha(a, 70),
                track: rgb(72, 72, 76),
                knob: rgb(236, 236, 240),
                shadow: Color32::from_black_alpha(110),
                star: rgb(255, 204, 0),
                success: rgb(48, 209, 88),
                warning: rgb(255, 214, 10),
                danger: rgb(255, 69, 58),
            }
        } else {
            Self {
                dark,
                accent: a,
                on_accent,
                window: rgb(246, 246, 248),
                sidebar: rgb(234, 234, 237),
                toolbar: rgb(242, 242, 245),
                canvas: rgb(210, 210, 215),
                card: rgb(255, 255, 255),
                control: rgb(226, 226, 230),
                control_hover: rgb(214, 214, 219),
                segment_selected: rgb(255, 255, 255),
                separator: Color32::from_black_alpha(26),
                text: rgb(29, 29, 31),
                text_secondary: rgb(110, 110, 115),
                text_tertiary: rgb(160, 160, 165),
                selection: with_alpha(a, 50),
                track: rgb(214, 214, 219),
                knob: rgb(255, 255, 255),
                shadow: Color32::from_black_alpha(40),
                star: rgb(255, 170, 0),
                success: rgb(40, 180, 70),
                warning: rgb(230, 160, 0),
                danger: rgb(255, 59, 48),
            }
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

fn palette_id() -> egui::Id {
    egui::Id::new("limestone_palette")
}

/// The palette for the current frame.
pub fn pal(ui: &egui::Ui) -> Palette {
    pal_ctx(ui.ctx())
}

pub fn pal_ctx(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp::<Palette>(palette_id()))
        .unwrap_or_else(|| Palette::new(true, Accent::Blue))
}

// ============================================================================
// Metrics
// ============================================================================

pub const SPACE_4: f32 = 4.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;

pub const RADIUS_SMALL: CornerRadius = CornerRadius::same(5);
pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(7);
pub const RADIUS_CARD: CornerRadius = CornerRadius::same(10);

pub const HEIGHT_TOOLBAR: f32 = 52.0;
pub const WIDTH_SIDEBAR: f32 = 300.0;
pub const WIDTH_INSPECTOR: f32 = 320.0;
pub const CONTROL_HEIGHT: f32 = 26.0;

// ============================================================================
// Application
// ============================================================================

/// Resolves the mode against the OS setting.
pub fn is_dark(ctx: &egui::Context, mode: ThemeMode) -> bool {
    match mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => ctx.system_theme().is_none_or(|t| t == egui::Theme::Dark),
    }
}

pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode, accent: Accent, ui_scale: f32) {
    let dark = is_dark(ctx, mode);
    let p = Palette::new(dark, accent);
    ctx.data_mut(|d| d.insert_temp(palette_id(), p));

    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.override_text_color = Some(p.text);
    v.panel_fill = p.window;
    v.window_fill = p.card;
    v.extreme_bg_color = if dark { rgb(28, 28, 30) } else { rgb(255, 255, 255) };
    v.faint_bg_color = p.card;
    v.hyperlink_color = p.accent;
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = RADIUS_CONTROL;
    v.window_stroke = Stroke::new(1.0_f32, p.separator);
    v.window_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 30,
        spread: 0,
        color: p.shadow,
    };
    v.popup_shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: p.shadow,
    };
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);
    v.text_cursor.stroke = Stroke::new(2.0_f32, p.accent);
    v.slider_trailing_fill = true;

    let w = &mut v.widgets;
    for (state, fill, stroke) in [
        (&mut w.noninteractive, p.window, p.separator),
        (&mut w.inactive, p.control, Color32::TRANSPARENT),
        (&mut w.hovered, p.control_hover, Color32::TRANSPARENT),
        (&mut w.active, p.control_hover, Color32::TRANSPARENT),
        (&mut w.open, p.control_hover, Color32::TRANSPARENT),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0_f32, stroke);
        state.corner_radius = RADIUS_CONTROL;
        state.fg_stroke = Stroke::new(1.0_f32, p.text);
        state.expansion = 0.0;
    }
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text_secondary);
    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    let prop = |size| FontId::new(size, FontFamily::Proportional);
    style.text_styles = [
        (TextStyle::Small, prop(11.0)),
        (TextStyle::Body, prop(13.0)),
        (TextStyle::Button, prop(13.0)),
        (TextStyle::Heading, prop(17.0)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(SPACE_8, SPACE_6);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style.spacing.interact_size.y = CONTROL_HEIGHT;
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.scroll = egui::style::ScrollStyle::floating();
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
    ctx.set_zoom_factor(ui_scale.clamp(0.75, 1.75));
}

// ============================================================================
// Fonts
// ============================================================================

/// Uses the platform UI font (SF Pro on macOS, Segoe UI on Windows) when available,
/// with Tabler Icons as a glyph fallback.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates: &[&str] = &[
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/SFNSText.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "/usr/share/fonts/truetype/inter/Inter-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    if let Some(bytes) = candidates.iter().find_map(|p| std::fs::read(p).ok()) {
        fonts
            .font_data
            .insert("system_ui".to_owned(), egui::FontData::from_owned(bytes).into());
        if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
            prop.insert(0, "system_ui".to_owned());
        }
    }

    fonts.font_data.insert(
        "tabler_icons".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/tabler-icons.ttf")).into(),
    );
    // Icons first: Tabler only defines Private Use Area codepoints, so it never
    // shadows text glyphs, and system fonts can't shadow the icons.
    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "tabler_icons".to_owned());
    }
    ctx.set_fonts(fonts);
}
