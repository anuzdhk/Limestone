# Topaz Adjust 4.0 Clone — Rust Implementation Plan

This document outlines the step-by-step engineering roadmap for building the Topaz Adjust 4.0 clone in Rust.

---

## 1. System Architecture & Crate Structure

We will structure the project as a high-performance Rust workspace:

```
androp/ (topaz-adjust-rust)
├── Cargo.toml                      # Workspace root
├── crates/
│   ├── adjust-core/                # The Image Processing Engine (Pure Rust + Rayon + SIMD)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Public API & Pipeline Coordinator
│   │       ├── color.rs            # Color space conversions (sRGB, Linear, HSV, LAB, Luminance)
│   │       ├── filter.rs           # Fast O(1) Box Filter & Edge-Preserving Guided Filter
│   │       ├── basic_exposure.rs   # Brightness, Contrast, Boost Blacks, Boost Whites
│   │       ├── adaptive_exposure.rs# Local tone mapping, Regions, Highlight/Shadow protection
│   │       ├── details.rs          # Independent detail extraction, Boost, Threshold, Radius
│   │       ├── color_adjust.rs     # Adaptive saturation (Vibrance), Hue, Saturation Boost
│   │       ├── denoise.rs          # Edge-preserving spatial noise suppression
│   │       ├── curve.rs            # Natural cubic spline interpolation & histogram generator
│   │       ├── finishing.rs        # Vignette, procedural film grain, warmth, transparency blend
│   │       ├── preset.rs           # Preset model, serde JSON parser, and built-in factory presets
│   │       └── pipeline.rs         # Unified execution pipeline (tiled & full-res)
│   │
│   ├── adjust-cli/                 # Command-line batch & headless test tool
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs             # CLI args: input, output, preset, slider overrides
│   │
│   └── adjust-gui/                 # Interactive Desktop Application (egui / eframe)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs             # App entry point
│           ├── app.rs              # Main UI State & Layout
│           ├── viewport.rs         # Interactive image renderer (Zoom, Pan, Split/Dual compare)
│           ├── panels/
│           │   ├── presets_panel.rs# Preset tree & thumbnail cards
│           │   ├── controls_panel.rs# The 7 adjustment tabs with sliders & resets
│           │   └── histogram.rs    # Real-time RGB & Luminance histogram widget
│           └── worker.rs           # Background Rayon thread pool for non-blocking rendering
```

---

## 2. Phase-by-Phase Execution Roadmap

### Phase 1: Engine Foundation & Mathematical Core (`adjust-core`)
* **Goal:** Implement the complete deterministic DSP pipeline in pure Rust.
* **Deliverables:**
  1. Workspace configuration with dependencies (`rayon`, `image`, `serde`, `serde_json`).
  2. Color space transformations (`sRGB <-> Linear RGB`, fast perceptual luminance).
  3. $O(1)$ Guided Filter implementation using integral images / moving sum box filters for sub-10ms performance on multi-megapixel images.
  4. Core stages: Basic Exposure, Adaptive Exposure (local tone mapping with regions), Independent Detail Extraction, Adaptive Saturation, Denoising, Curve Spline, and Finishing Touches (Vignette, Grain, Warmth, Transparency).
  5. Built-in Preset Database with Topaz Adjust 4.0 defaults (*Spicify*, *Dramatic*, *Photo Pop*, *Detail Light*, *Painting*, *Portrait Smooth*).
  6. Comprehensive unit tests verifying each algorithm against expected outputs.

### Phase 2: CLI Runner & Visual Verification (`adjust-cli`)
* **Goal:** Enable headless image processing, testing against real photos, and benchmark latency.
* **Deliverables:**
  1. CLI tool accepting `--input <path>`, `--output <path>`, `--preset <name>`, and optional slider overrides.
  2. Automated test suite generating before/after sample comparisons for factory presets.
  3. Multi-threaded benchmark measuring execution time across 1080p, 4K, and 24MP images.

### Phase 3: High-Performance Desktop UI (`adjust-gui`)
* **Goal:** Build the authentic 3-panel Topaz Adjust 4.0 editing environment.
* **Deliverables:**
  1. **Interactive Viewport:**
     - 60 FPS panning and smooth zooming (`Fit`, `50%`, `100%`, `200%`).
     - Split-screen comparison wipe slider (drag left/right to compare before/after).
     - Hold-to-preview original shortcut (spacebar/click).
  2. **Left Panel:** Preset explorer with categories and instant one-click activation.
  3. **Right Panel:** Tabbed interface with all 7 adjustment sections, accurate slider bounds, reset buttons, and live histogram.
  4. **Worker Threading:** Viewport preview renders on low-res mipmap during active slider dragging; high-res render completes smoothly on background worker without freezing the UI.

### Phase 4: Polish, Export & Performance Optimization
* **Goal:** Production readiness and maximum throughput.
* **Deliverables:**
  1. 16-bit TIFF / PNG / JPEG lossless and lossy export.
  2. Custom preset saving and loading (import/export `.json` preset files).
  3. Snapshot buffer (A / B / C / D comparison slots).
  4. Optional SIMD vectorization (AVX2 / NEON) for maximum throughput.

---

## 3. Technology Stack Selection

| Component | Choice | Rationale |
| :--- | :--- | :--- |
| **Language** | Rust 2021 (1.99+) | Memory safety, zero-cost abstractions, predictable low latency, top-tier concurrency. |
| **Concurrency** | `rayon` | Automatic work-stealing thread pool for tile/scanline parallel image processing. |
| **Image I/O** | `image` crate | Pure Rust decoding/encoding for PNG, JPEG, TIFF, WebP, BMP. |
| **Serialization** | `serde` + `serde_json` | Fast, reliable preset format. |
| **Desktop GUI** | `eframe` / `egui` | Immediate-mode GUI in pure Rust; hardware-accelerated, lightweight, cross-platform, zero native C++ binding hassles. |
| **Math / Splines** | Custom lightweight DSP | Zero heavy external math dependencies; fully self-contained. |
