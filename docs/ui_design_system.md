# Limestone Pro — GUI Design System & UX Architecture

Limestone Pro is a clean-room clone of Topaz Adjust 4.0 built in Rust and `egui`/`eframe`. This document details the visual design system, interaction patterns, layout specifications, and architecture of the user interface located in `crates/adjust-gui`.

---

## 1. Visual Hierarchy & Design System

The visual design system follows a modern, content-first studio dark theme inspired by Lightroom, DaVinci Resolve, and Figma.

### 1.1 Color Tokens (`crates/adjust-gui/src/theme.rs`)

| Token | Hex / Value | Description |
|---|---|---|
| `BG_CANVAS` | `#0B0C0F` | Pure neutral near-black canvas so photographs can be evaluated without optical color tinting or false luminance bias. |
| `SURFACE_PANEL` | `#111318` | Base elevation for persistent sidebars, top toolbar, and bottom status bar. |
| `SURFACE_CARD` | `#171A21` | Elevation 2 for nested cards, inspector containers, and inactive controls. |
| `SURFACE_RAISED` | `#1F232C` | Elevation 3 for hovered items, active inputs, and elevated popups. |
| `BORDER_SUBTLE` | `#2A2F3A` | 1px boundary dividing panels with high subtlety. |
| `ACCENT` | `#00D2FF` | Electric cyan accent for active selection, sliders, focus rings, and action bakes. |
| `ACCENT_HOVER` | `#33DCFF` | Slightly lighter cyan on mouse hover. |
| `ACCENT_PRESSED` | `#00A8CC` | Deep cyan during mouse press. |
| `ACCENT_MUTED` | `rgba(0, 210, 255, 0.14)` | Low-opacity cyan tint for active rows and selections. |
| `ACCENT_STAR` | `#FFC832` | Warm amber star for favorite preset toggles. |
| `TEXT_PRIMARY` | `#E8EAF0` | High-contrast readable text (WCAG AA compliant). |
| `TEXT_SECONDARY` | `#9AA3B2` | Muted labels, secondary captions, and metadata. |
| `TEXT_MUTED` | `#78808E` | Subtle hints and inactive states. |
| `TEXT_DISABLED` | `#5B6472` | Disabled controls and bypassed stages. |
| `STATUS_SUCCESS` | `#2ECC71` | Green status dot and success toasts. |
| `STATUS_WARNING` | `#F1C40F` | Amber warning badges and toasts. |
| `STATUS_ERROR` | `#E74C3C` | Red error messages and export failures. |

---

### 1.2 Typography Hierarchy

A unified typographic scale enforces strict visual consistency:
- **Panel Title**: 20px / Bold (`Limestone Pro`)
- **Section Headers**: 15px / Strong (`Basic Exposure`, `Details & Clarity`)
- **Body & Labels**: 13px / Regular
- **Captions & Metadata**: 11px / Muted (`Frame latency: 14.2 ms`)
- **Monospace Readouts**: 11.5px Monospace (Non-jittering numeric slider fields and keyboard shortcuts)

---

### 1.3 4px Base Grid & Radii

- **Base Grid**: `SPACE_4` (4px), `SPACE_6` (6px), `SPACE_8` (8px), `SPACE_12` (12px), `SPACE_16` (16px), `SPACE_24` (24px).
- **Corner Radii**:
  - `RADIUS_CONTROL` = 6px (buttons, sliders, inputs)
  - `RADIUS_CARD` = 10px (floating overlays, drop zones, inspector containers)
  - `RADIUS_PILL` = 14px (segmented controls, handles)

---

## 2. Layout & Workspace Panels

```
+---------------------------------------------------------------------------------------------------------+
| TOP TOOLBAR (48px): [Brand] | [Open] [Export] | [Undo] [Redo] | View: [Single|Split|Dual] | Zoom | Reset| ? |
+------------------+------------------------------------------------------------------+-------------------+
| PRESETS (260px)  | CENTER CANVAS VIEWPORT                                           | INSPECTOR (330px) |
| - Search         | - Neutral #0B0C0F background with photo drop shadow              | - [Nav|Loupe|Hist]|
| - Collections    | - Interactive Pan & Zoom (Mouse drag & wheel)                    |-------------------|
| - Live Thumbs    | - Polished Split Wipe handle with grip pill & tags               | ADJUSTMENTS       |
| - Star Favorites | - Drag-and-drop file receiver & clean empty state                | - 8-stage accordion|
| - Custom I/O     | - Spacebar hold-to-compare instant toggle badge                  | - Tooltips & dots |
|                  |                                                                  | - [Bake/Apply] [↺]|
+------------------+------------------------------------------------------------------+-------------------+
| STATUS BAR (24px): [● Ready] | 3840×2160 px | RGB f32 Linear | Interactive 960px | Frame: 12.4 ms | Zoom 100%|
+---------------------------------------------------------------------------------------------------------+
```

---

## 3. Adjustment Accordion (The 8 DSP Stages)

Each stage can be collapsed, bypassed (eye toggle), or reset individually:
1. **Basic Exposure** (`ICON_SUN`): Brightness, Contrast, Boost Blacks, Boost Whites.
2. **Adaptive Exposure** (`ICON_EYE`): Adaptive Exp, Regions (1..30), Contrast, Brightness, Protect Highlights, Protect Shadows.
3. **Details & Clarity** (`ICON_SPARKLES`): Strength, Detail Boost, Threshold, Radius, Sharpen, Process Details Independently checkbox.
4. **Color & Vibrance** (`ICON_PALETTE`): Adaptive Saturation, Color Regions, Saturation, Saturation Boost, Hue Shift (-180°..+180°).
5. **Noise Suppression** (`ICON_VOLUME_OFF`): Suppression strength, Amount threshold.
6. **Tone Curves** (`ICON_VECTOR_BEZIER`): 4-channel spline editor (RGB, R, G, B) with 25% grid lines, control point coordinates, and linear/S-curve presets.
7. **Finishing Touches** (`ICON_WAND`): Transparency, Warmth, Vignette (Amount, Size, Transition, Roundness), Film Grain (Amount, Size), Border frame.
8. **Master Transparency & Layer Stacking**: Multi-layer effect stacking with the `Apply (Bake)` and `Pop Layer` workflow.

---

## 4. Key UX Enhancements & Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| **Space (Hold)** | Quick compare: instant preview of the original unprocessed image |
| **Ctrl + O / Cmd + O** | File open dialog |
| **Ctrl + E / Cmd + E** | Full-resolution multi-format export dialog (PNG, JPEG, TIFF) |
| **Ctrl + Z** | Undo parameter change |
| **Ctrl + Shift + Z** | Redo parameter change |
| **1 / 2 / 3** | Switch view mode: Single, Split Wipe, Side-by-Side |
| **F** | Fit image to canvas center |
| **0** | Reset zoom to 100% actual pixel size |
| **Tab** | Focus mode: toggle left and right sidebars on/off |
| **?** | Open keyboard shortcuts studio cheat sheet |
| **Shift + Slider Drag** | Fine-grained precision adjustment |
| **Double-Click Slider** | Instantly reset slider to default neutral value |
| **Drag & Drop** | Drop any image onto the canvas to open immediately |
