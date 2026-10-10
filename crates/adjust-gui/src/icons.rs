use eframe::egui;

// Tabler Icons (https://tabler.io/icons) Unicode Codepoints
pub const ICON_SUN: &str = "\u{eb30}";               // Basic Exposure
pub const ICON_EYE: &str = "\u{ea9a}";               // Adaptive Exposure
pub const ICON_SPARKLES: &str = "\u{f6d7}";          // Details & Micro-contrast
pub const ICON_PALETTE: &str = "\u{eb01}";           // Color Balance & Vibrance
pub const ICON_VOLUME_OFF: &str = "\u{f1c3}";        // Noise Suppression
pub const ICON_VECTOR_BEZIER: &str = "\u{ef1d}";     // Curve Tool (Spline)
pub const ICON_WAND: &str = "\u{ebcb}";              // Finishing Touches & Creative
pub const ICON_FOLDER_OPEN: &str = "\u{faf7}";       // File Open
pub const ICON_DEVICE_FLOPPY: &str = "\u{eb62}";     // File Save
pub const ICON_REFRESH: &str = "\u{eb13}";           // Reset
pub const ICON_LAYOUT_COLUMNS: &str = "\u{ead4}";    // Split / Comparison
pub const ICON_PHOTO: &str = "\u{eb0a}";             // Viewport
pub const ICON_CHART_LINE: &str = "\u{ea5c}";        // Live Histogram
pub const ICON_CHECK: &str = "\u{ea5e}";             // Apply / Done
pub const ICON_X: &str = "\u{eb55}";                 // Cancel / Close
pub const ICON_SEARCH: &str = "\u{eb1c}";            // Loupe / Search
pub const ICON_STAR: &str = "\u{eb2c}";              // Favorite Preset
pub const ICON_STAR_FILLED: &str = "\u{f6f8}";       // Favorite Preset Active
pub const ICON_COMPASS: &str = "\u{ea79}";           // Navigate
pub const ICON_ZOOM_IN: &str = "\u{eb54}";           // Zoom In
pub const ICON_ZOOM_OUT: &str = "\u{eb53}";          // Zoom Out
pub const ICON_HISTORY: &str = "\u{eaee}";           // History / Snapshot
pub const ICON_BRUSH: &str = "\u{ea42}";             // Local Adjustments Brush
pub const ICON_APPLY: &str = "\u{f15f}";             // Apply Button (Stacking effects)
pub const ICON_BOLT: &str = "\u{ea37}";              // Speed / GPU latency
pub const ICON_GRIP: &str = "\u{eaeb}";              // Grip Vertical / Handle
pub const ICON_ARROW_BACK_UP: &str = "\u{ea0c}";     // Undo
pub const ICON_ARROW_FORWARD_UP: &str = "\u{ea0d}";  // Redo
pub const ICON_CHEVRON_DOWN: &str = "\u{ea5f}";      // Accordion expanded
pub const ICON_CHEVRON_RIGHT: &str = "\u{ea61}";     // Accordion collapsed
pub const ICON_EYE_OFF: &str = "\u{ea99}";           // Bypass stage
pub const ICON_HELP: &str = "\u{ea9f}";              // Shortcuts cheat sheet
pub const ICON_UPLOAD: &str = "\u{eb47}";            // Drop zone upload
pub const ICON_KEYBOARD: &str = "\u{eab5}";          // Keyboard shortcuts
pub const ICON_SETTINGS: &str = "\u{eb20}";          // Settings / Preferences
pub const ICON_CLIPBOARD: &str = "\u{ea70}";         // Paste / Clipboard
pub const ICON_CPU: &str = "\u{ea85}";               // Compute / Hardware Acceleration
pub const ICON_WORLD: &str = "\u{eb54}";             // Language / Globe
pub const ICON_SLIDERS: &str = "\u{eb28}";           // Sliders / Opacity settings

pub fn setup_tabler_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Embed the official Tabler Icons TTF font
    fonts.font_data.insert(
        "tabler_icons".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/tabler-icons.ttf")).into(),
    );

    // Append Tabler Icons as fallback glyphs for Proportional font family
    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        prop.push("tabler_icons".to_owned());
    }

    ctx.set_fonts(fonts);
}
