# Limestone - Image Enhancement Tool

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)]()
[![Release](https://img.shields.io/badge/release-v0.1.0--beta-brightgreen.svg)](https://github.com/anuzdhk/Limestone/releases)

A fast desktop image enhancement tool written in Rust, inspired by *Topaz Adjust 5.0*. Limestone brings adaptive exposure, detail and clarity enhancement, vibrant colour, HDR-style tone mapping, film looks, toning and creative effects together with 157 one-click presets, all rendered live on your photo.

Built with [egui](https://github.com/emilk/egui) and [Tabler Icons](https://tabler.io/icons). Currently in **beta**.

---

## 📦 Downloads & Installation

| Platform | Package Format | Download / Build |
| :--- | :--- | :--- |
| **Windows (x64)** | **MSI Installer** (`.msi`) | [GitHub Releases](https://github.com/anuzdhk/Limestone/releases) or run `.\scripts\build_windows_msi.ps1` |
| **macOS (Universal)** | **Apple Disk Image** (`.dmg`) | [GitHub Releases](https://github.com/anuzdhk/Limestone/releases) or run `./scripts/build_macos_dmg.sh` |

Installers are built by CI (`.github/workflows/release.yml`) when a `v*` tag is pushed; binaries are not committed to the repository.


---

## ✨ Features Overview

### 🖼️ Editing Workspace
- **Light & dark themes**: follows your system setting automatically (or choose one), with selectable accent colours and an interface-size setting.
- **Toolbar**: open / paste, *Single · Split · Side by Side* comparison, undo/redo, zoom (true 100% and auto-fit) and **Export**.
- **Preset browser**: *All Presets*, *Favorites* and *My Collection*, plus every factory collection with preset counts.
- **Live preset thumbnails**: every preset is previewed on *your* photo, rendered in the background.
- **Adjustment panel**: RGB histogram, navigator and loupe; an **Effect Amount** slider; collapsible sections with on/off toggles and per-section Reset. Double-click a slider to reset it, Shift-drag for fine control.
- **Effect stacking**: **Apply** bakes the current look into the image so you can layer another on top; **Undo Layer** removes the last one and restores its settings.
- **My Collection**: save the current adjustments as a preset (Ctrl+S), import/export preset files, right-click any preset to favourite or export it.
- **Quality output**: 16-bit PNG/TIFF and transparency are preserved on export; EXIF orientation is honoured on import.

---

### ⚡ Algorithmic DSP Pipeline (`adjust-core`)

The image processing engine operates on parallel 32-bit floating-point channels (`RgbImageF32`) in gamma-encoded sRGB space (values `0.0..=1.0`). Spatial radii are defined relative to a 1000px reference so the 960px live preview, 100% zoom tiles, and full-resolution exports all match:

```
Input Image (RGB f32, 8/16-bit, alpha preserved)
   │
   ├─► [Optional] Independent High-Frequency Detail Extraction (Guided Filter)
   ▼
[1] Light            Brightness, Contrast, Blacks, Whites
[2] Adaptive Exposure Retinex local gamma tone mapping, 1..30 regions
[3] Detail           Threshold smoothing, small-detail boost, sharpening, radius
[4] Color            Vibrance, saturation, hue shift
[5] Noise Reduction  Edge-aware guided-filter luma denoising
[6] Creative         Pencil / colour sketch, glow, posterize, solarize, invert
[7] B&W and Toning   Channel-mixer black & white, split toning (shadow/highlight tints)
[8] Curves           Master + per-channel R/G/B curves (1024-entry LUTs)
[9] Finishing        Warmth, vignette, film grain, border, effect amount
   ▼
Final Output Image
```

#### Key Technical Highlights:
1. **$O(1)$ Per-Pixel Guided Filtering**: Separable running-sum box filters (the vertical pass runs on a transposed buffer) using Rayon. Time complexity is independent of filter radius.
2. **Monotonic Local Gamma Tone Mapping**:
   $$\gamma = \exp(-(0.5 - \text{base}) \cdot \text{strength})$$
   Ensures non-inverting luminance curves that avoid shadow crushing and severe halos.
3. **Luminance-preserving toning**: sepia, selenium and split tones re-colour each pixel without brightening or darkening it.
4. **Tile-exact rendering**: `process_region` renders any crop with the right filter margin, so zoomed tiles match a full render pixel-for-pixel (covered by tests).
5. **Background rendering**: previews, full-res tiles, layer baking, exports and thumbnails all run off the UI thread.

---

## 🗂️ Factory Preset Collections

157 factory presets organised into seven collections (a preset can appear in more than one):

| Collection | Presets |
| :--- | :--- |
| **Adjust 4 Preset List** (35) | Clarity, Color Blast, Crisp, Dark - Ghostly, Dark - Night, Detail - Simple Pop, Detail - Strong, Detailed, Dramatic, Enhance Contrast, Equalize, Exposure Color Sketch, Exposure Correction, HDR - Pop, HDR - Sketch, Mild Color Pop, Mild Contrast Pop, Mild Details, Neutralizer by JorgeDiaz, Painting - Venice, Photo Pop, Portrait Drama, Portrait Smooth, Psychedelic, Recovery - Highlight, Recovery - Shadow, Simplify, Sketch - Color, Sketch - Dark Charcoal, Sketch - Med, Sketch - Pencil, Small Details, Smooth and Flat, Spicify, Vibrance |
| **Classic** (24) | Brilliant Cold, Brilliant Warm, Cool Tone I–II, Delicate Details I–II, Detail - Light, Detail - Medium, Equalize, Exposure Correction, High Key I–II, Highlight Recovery, Low Key I–II, Mild Color Pop, Mild Contrast Pop, Photo Pop, Shadow Recovery, Simple Pop, Soft Focus I–II, Warm Tone I–II |
| **Vibrant** (30) | Autumn, Bold, Boost, Clarity, Color Blast, Color Pop I–II, Crisp, Detail - Strong, Detail - Strong II, Detailed Color, Dramatic, Dramatic II, Exposure Color Stretch, French Countryside, Gritty I–II, High Contrast, Neutralizer by JorgeDiaz, Portrait Drama, Relative Contrast, Sapphire, Saturation Punch, Setting Sun, Setting Sun - Dynamic, Spicify, Spring, Summer, Vibrance, Winter |
| **HDR** (9) | Dynamic Brightness, Dynamic Pop, Dynamic Pop II, Light / Medium / Heavy Pop Grunge, Light / Medium / Heavy Pop Smooth |
| **Film** (27) | Bleach Bypass Cool / Warm, Color Negative, Cross Process I–V, Daylight Film in Tungsten I–II, Lomo I–III, Retro Style I–IV, Timeless I–IV, Tungsten Film in Daylight I–II, Vintage Grunge I–IV |
| **Toned** (19) | Black Rose, Blue, Brown, Cerulean, Colored Selenium I–II, Colored Sepia, Colored Sepia II, Detailed Black and White, Mint, Mustard, Sharp Black and White, Soft Pink, Split Tone I–VI |
| **Stylized** (39) | Blueprint, Brash, Cheerful Color, Colorized I–III, Comic Book, Comic Book Art, Dark - Ghostly, Dark - Night, Dayglow, Desaturated Detail, Dusk, Extreme Color, Extreme Color Blast, Faded Glory, Glow In The Dark, Grunge Me, Grunge Me BW, Grunge Me Desaturated, HDR - Sketch, HDR Sketch - Light, High Definition, Liquid Chrome, Mystified, Painting - Bold, Painting - Venice, Portrait Smooth, Psychedelic, Radical Realism, Simplify, Sketch - Color, Sketch - Dark Charcoal, Sketch - Med, Sketch - Pencil, Smooth and Flat, So Simple, Solarized Dreams I–II |

Render every preset onto one of your photos as a contact sheet:

```bash
cargo run --release -p adjust-core --example contact_sheet -- photo.jpg sheet.png
```

---

## 📂 Repository Structure

```
.
├── crates/
│   ├── adjust-core/               # Pure Rust DSP image filtering engine
│   │   ├── src/
│   │   │   ├── filter.rs          # O(1) box & guided edge-preserving filters
│   │   │   ├── image_buffer.rs    # RgbImageF32: 8/16-bit + alpha load/save, resampling
│   │   │   ├── params.rs          # All adjustment parameters, serde defaults, sanitizing
│   │   │   ├── pipeline.rs        # 10-stage pipeline, RenderContext, process_region
│   │   │   ├── preset.rs          # 157 factory presets and the collections
│   │   │   └── stages/            # Exposure, details, color, denoise, stylize, tone, curve, finishing
│   │   ├── examples/              # contact_sheet: render every preset onto a photo
│   │   └── tests/                 # Rendering, tiling, preview-vs-export and I/O tests
│   ├── adjust-gui/                # Desktop editor (eframe/egui)
│   │   └── src/
│   │       ├── main.rs            # App state, rendering workflow, layer stack
│   │       ├── theme.rs           # Palettes, accents, system fonts
│   │       ├── panels/            # Toolbar, sidebar, canvas, inspector, dialogs
│   │       ├── widgets/           # Sliders, segmented controls, sections, toasts
│   │       ├── render_worker.rs   # Background preview / tile / bake rendering
│   │       ├── thumbnails.rs      # Live preset thumbnails
│   │       └── user_presets.rs    # My Collection storage
│   └── adjust-cli/                # Headless command line processing & preset export
├── docs/                          # Specs and implementation plan
├── presets/                       # Example presets in JSON format
├── scripts/                       # Local MSI / DMG packaging scripts
└── wix/                           # WiX installer definition
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust 1.80+](https://rustup.rs/) (Cargo toolchain)
- Modern graphics card with OpenGL support (Standard on Windows, macOS, Linux)

### Building from Source

Clone the repository and build the workspace in release mode:

```bash
# Clone repository
git clone https://github.com/anuzdhk/Limestone.git
cd Limestone

# Build all crates in Release mode
cargo build --release
```

### Running the GUI

Launch the editor:

```bash
cargo run --release -p adjust-gui
```
*Or run the release binary directly: `.\target\release\adjust-gui.exe`.*

#### Keyboard & Mouse Shortcuts:
- **Space (hold)**: show the original. **1 / 2 / 3**: Single · Split · Side by Side.
- **Ctrl+O / Ctrl+V / Ctrl+E**: open, paste (image or copied file), export. **Ctrl+S**: save as preset. **Ctrl+,**: settings.
- **Ctrl+Z / Ctrl+Shift+Z**: undo / redo (one step per slider gesture).
- **F / 0 / + / −**: fit, actual size, zoom in / out. Mouse wheel zooms around the cursor; drag to pan.
- **Tab**: hide/show both sidebars. **?**: keyboard shortcut sheet.
- **Sliders**: double-click to reset, Shift-drag for fine control.
- **Curves**: click to add a point, drag to move, right-click to remove; R/G/B tabs edit per-channel curves.
- **Presets**: right-click to favourite, export, or (in My Collection) delete.

---

### Command-Line Interface (`adjust-cli`)

Batch process images or export presets without opening the GUI:

```bash
# Apply a factory preset to an image
cargo run --release -p adjust-cli -- --input input.jpg --output output.png --preset spicify

# Apply a custom JSON preset
cargo run --release -p adjust-cli -- --input input.jpg --output output.png --preset-file presets/dramatic.json

# List factory presets / export one as JSON
cargo run --release -p adjust-cli -- list-presets
cargo run --release -p adjust-cli -- export-preset --name dramatic --output presets/dramatic.json
```

---

## 🧪 Testing & Verification

Run the automated DSP test suite verifying that all presets render deterministic, non-clipping pixels across multithreaded chunks:

```bash
cargo test --workspace
```

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details. Inspired by Topaz Adjust 5.0. Topaz Adjust is a trademark of Topaz Labs LLC; Limestone is an independent project and is not affiliated with or endorsed by Topaz Labs.
