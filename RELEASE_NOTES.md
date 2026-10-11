## Limestone 0.3.0 (beta)

Limestone is a desktop image enhancement tool inspired by Topaz Adjust 5.0. This release adds the full preset library, a redesigned editor, and many fixes.

### Downloads

| Platform | File |
| :--- | :--- |
| **Windows** (64-bit) | `Limestone-<version>-Windows-x64.msi` |
| **macOS** (Apple Silicon & Intel) | `Limestone-<version>-macOS-Universal.dmg` |

> **macOS:** the app is not yet signed with an Apple Developer certificate. The first time you open it, right-click the app and choose **Open**, or run `xattr -dr com.apple.quarantine /Applications/Limestone.app`.

### What's new

**157 presets in seven collections:** Adjust 4 Preset List, Classic, Vibrant, HDR, Film, Toned and Stylized. Every preset shows a live thumbnail rendered on your own photo.

**New effects**
- Black & white with a channel mixer, sepia/selenium toning and split toning
- Creative effects: pencil and colour sketch, glow, posterize, solarize, invert
- Per-channel R/G/B curves (cross process, Lomo, colour negative looks)

**Redesigned editor**
- Light and dark themes that follow your system, with selectable accent colours
- Single, Split and Side by Side comparison; hold Space to see the original
- Effect stacking: **Apply** a look, layer another on top, **Undo Layer** to step back
- **My Collection:** save your own presets (Ctrl+S), import, export and favourite them
- True 100% zoom with full-resolution detail; the image stays fitted as you resize
- Paste images or copied files with Ctrl+V

**Quality and fixes**
- The preview now matches the exported image exactly
- 16-bit PNG/TIFF and transparency are kept on export; photo rotation (EXIF) is respected
- Fixed colour blotches in grey areas (e.g. skies) when boosting saturation
- Each slider drag is one undo step; editing never freezes the window
- Faster processing: blur cost no longer grows with the effect radius
