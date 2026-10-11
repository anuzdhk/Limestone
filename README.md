# LimeStone Pro (Beta Release)

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)]()
[![Release](https://img.shields.io/badge/release-v0.1.0--beta-brightgreen.svg)](https://github.com/anuzdhk/Limestone/releases)

A clean-room Rust desktop photo-adjustment studio inspired by the classic *Topaz Adjust 4* workflow. Features multithreaded $O(1)$ separable guided filtering, Retinex local tone mapping, micro-contrast enhancement, natural cubic spline curve manipulation, and a pro studio UI powered by [egui](https://github.com/emilk/egui) and [Tabler Icons](https://tabler.io/icons).

---

## 📦 Downloads & Installation

| Platform | Package Format | Download / Build |
| :--- | :--- | :--- |
| **Windows (x64)** | **MSI Installer** (`.msi`) | [GitHub Releases](https://github.com/anuzdhk/Limestone/releases) or run `.\scripts\build_windows_msi.ps1` |
| **macOS (Universal)** | **Apple Disk Image** (`.dmg`) | [GitHub Releases](https://github.com/anuzdhk/Limestone/releases) or run `./scripts/build_macos_dmg.sh` |

Installers are built by CI (`.github/workflows/release.yml`) when a `v*` tag is pushed; binaries are not committed to the repository.


---

## ✨ Features Overview

### 🎨 Pro UI
- **Obsidian Dark Palette**: High-contrast, fatigue-free dark theme (`#101115` / `#16171c` / `#21232b`) with vibrant cyan accenting (`#00d2ff`).
- **Tabler Icons Integration**: Crisp vector iconography throughout toolbar, adjustment tabs, and collections.
- **View Modes**:
  - **Single View**: Full canvas with hold-`Spacebar` for instant before/after toggle.
  - **Split Wipe**: Interactive divider slider comparing original vs. processed in real-time.
  - **Side-by-Side**: Dual synchronized preview comparison.
- **Top-Right Inspector**:
  - `NAVIGATE`: Miniature thumbnail overview with real-time viewport crop boundary box.
  - `LOUPE`: 400% zoom loupe targeting critical micro-textures and skin details.
  - `HISTOGRAM`: Live 256-bin RGB and perceptual luminance distribution graph.
- **Effect Stacking (`Apply` Button Workflow)**:
  - Supports stacking multiple effects in a single session (e.g. *Exposure Correction* $\rightarrow$ *Detail Light* $\rightarrow$ *Warm Tone*) with full layer undo capability (Undo Layer restores the image exactly as it was before the last Apply).

---

### ⚡ Algorithmic DSP Pipeline (`adjust-core`)

The image processing engine operates entirely on parallel 32-bit floating-point channels (`RgbImageF32`) in gamma-encoded sRGB space (values `0.0..=1.0`). Spatial radii are defined relative to a 1000px reference so the 960px live preview, 100% zoom tiles, and full-resolution exports all match:

```
Input Image (RGB f32)
   │
   ├─► [Optional] Independent High-Frequency Detail Extraction (Guided Filter)
   │
   ▼
[1] Basic Exposure (Brightness, Contrast, Whites/Blacks Boost)
   │
   ▼
[2] Adaptive Exposure (Retinex Local Gamma Tone Mapping, 1..30 Regions)
   │
   ▼
[3] Details & Clarity (Threshold Smoothing, Detail Boost, Sharpening, Radius)
   │
   ▼
[4] Color & Vibrance (Adaptive Saturation, Color Regions, Global Saturation, Hue)
   │
   ▼
[5] Noise Suppression (Edge-Aware Guided Filter Luma Denoising)
   │
   ▼
[6] Curve Tool (1024-LUT Natural Cubic Spline with Histogram Overlay)
   │
   ▼
[7] Finishing Touches (Warmth, Vignette Falloff, Midtone Film Grain, Borders)
   │
   ▼
[8] Master Transparency Blending (Dry/Wet mix against original)
   │
   ▼
Final Output Image
```

#### Key Technical Highlights:
1. **$O(1)$ Per-Pixel Guided Filtering**: Separable horizontal and vertical passes using parallel Rayon chunk iterators. Time complexity is independent of spatial filter radius.
2. **Monotonic Local Gamma Tone Mapping**:
   $$\gamma = \exp(-(0.5 - \text{base}) \cdot \text{strength})$$
   Ensures non-inverting luminance curves that eliminate shadow crushing and severe halos.
3. **Interactive Spline Curves**: Piecewise cubic Hermite interpolation generating 1024-entry lookup tables (LUT) evaluated via vectorized parallel array lookups.
4. **Background Live Preview Engine**: A 960px downsampled proxy is processed on a worker thread so the UI never blocks; exports and zoomed tiles render against the full original resolution with resolution-independent radii.

---

## 🗂️ Factory Preset Collections

Includes factory presets with slider and curve configurations:

| Collection | Included Presets |
| :--- | :--- |
| **Classic Collection** | *Mild Contrast Pop*, *Detail - Light*, *Detail - Medium*, *Photo Pop*, *Exposure Correction*, *Equalize*, *Highlight Recovery*, *Shadow Recovery*, *High Key I*, *Low Key I*, *Neutral* |
| **Vibrant Collection** | *Mild Color Pop*, *Simple Pop* |
| **HDR Collection** | *Spicify* (Signature Topaz look), *Dramatic* |
| **Stylized Collection** | *Painting* (Oil/Illustrative), *French Countryside*, *Soft Focus I* |
| **Toned Collection** | *Warm Tone I*, *Cool Tone I* |
| **Film Collection** | *Vintage 35mm* |
| **Favorites** | Toggle preset stars (`★`) to filter customized sets instantly. |

---

## 📂 Repository Structure

```
.
├── crates/
│   ├── adjust-core/               # Pure Rust DSP image filtering engine
│   │   ├── src/
│   │   │   ├── filter.rs          # O(1) Box & Guided edge-preserving filters
│   │   │   ├── image_buffer.rs    # RgbImageF32 SIMD/Rayon memory layouts
│   │   │   ├── params.rs          # Complete Adjust 4 parameters struct & serde
│   │   │   ├── pipeline.rs        # Full 8-stage image processing pipeline
│   │   │   ├── preset.rs          # 21 Factory presets & custom preset JSON I/O
│   │   │   └── stages/            # Individual processing stages (Exposure, Details, Curves...)
│   │   └── tests/                 # Unit & integration rendering test suite
│   ├── adjust-gui/                # Antigravity Dark Studio eframe/egui GUI
│   │   ├── assets/                # Embedded Tabler Icons TTF font
│   │   └── src/
│   │       ├── curve_widget.rs    # Interactive Spline Curve widget with histogram
│   │       ├── icons.rs           # Tabler Icons unicode bindings
│   │       └── main.rs            # Inspector panels, view modes, and event loop
│   └── adjust-cli/                # Headless command line processing & preset export
├── docs/
│   ├── topaz_adjust_4_reverse_engineering_spec.md  # Complete mathematical & UI spec
│   └── rust_implementation_plan.md                 # Phased architecture roadmap
├── presets/                       # Exported factory presets in JSON format
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

Launch the standalone studio:

```bash
cargo run --release -p adjust-gui
```
*Or run the release binary directly: `.\target\release\adjust-gui.exe`.*

#### Keyboard & Mouse Shortcuts:
- **`Spacebar` (Hold)**: Toggle instantaneous comparison with the original image.
- **Left-Click + Drag**: Pan canvas (or drag the split wipe handle in Split View).
- **Mouse Wheel**: Smooth zoom in / out ($10\% \dots 1000\%$).
- **Left-Click on Curve**: Add a new control point (up to 8 points).
- **Left-Click + Drag Point**: Move curve control point in real time.
- **Right-Click on Point**: Delete an intermediate curve control point.
- **Double-Click Slider**: Reset slider value to default.

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

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details. Topaz Adjust is a trademark of Topaz Labs LLC. LimeStone is an independent project and is not affiliated with, endorsed by, or sponsored by Topaz Labs.
