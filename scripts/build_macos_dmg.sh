#!/usr/bin/env bash
set -e

# ==============================================================================
# LimeStone Pro - macOS .app Bundle and DMG Generator
# Works automatically on both Apple Silicon (M1/M2/M3/M4) and Intel Macs
# ==============================================================================

VERSION="0.1.0-beta"
APP_NAME="LimeStone"
VOL_NAME="LimeStone Pro"
BUNDLE_DIR="dist/${APP_NAME}.app"
DMG_PATH="dist/${APP_NAME}-${VERSION}-macOS.dmg"

echo "=========================================================="
echo "  Building LimeStone Pro DMG for macOS"
echo "=========================================================="

# 1. Determine local architecture
ARCH=$(uname -m)
echo "==> Detected architecture: ${ARCH}"

if [ "${ARCH}" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

echo "==> Ensuring Rust target ${TARGET} is installed..."
rustup target add "${TARGET}" || true

echo "==> Compiling optimized release binary..."
cargo build --release -p adjust-gui --target "${TARGET}"

echo "==> Assembling .app bundle directory structure..."
rm -rf "dist/${APP_NAME}.app"
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

echo "==> Copying binary into .app bundle..."
cp "target/${TARGET}/release/adjust-gui" "${BUNDLE_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${BUNDLE_DIR}/Contents/MacOS/${APP_NAME}"

echo "==> Creating Info.plist..."
cat << 'EOF' > "${BUNDLE_DIR}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>LimeStone</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.limestone.studio</string>
    <key>CFBundleName</key>
    <string>LimeStone Pro</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "==> Generating AppIcon.icns from logo/logo.png..."
if [ -f "logo/logo.png" ]; then
    rm -rf icon.iconset
    mkdir -p icon.iconset
    sips -z 16 16     logo/logo.png --out icon.iconset/icon_16x16.png > /dev/null 2>&1
    sips -z 32 32     logo/logo.png --out icon.iconset/icon_16x16@2x.png > /dev/null 2>&1
    sips -z 32 32     logo/logo.png --out icon.iconset/icon_32x32.png > /dev/null 2>&1
    sips -z 64 64     logo/logo.png --out icon.iconset/icon_32x32@2x.png > /dev/null 2>&1
    sips -z 128 128   logo/logo.png --out icon.iconset/icon_128x128.png > /dev/null 2>&1
    sips -z 256 256   logo/logo.png --out icon.iconset/icon_128x128@2x.png > /dev/null 2>&1
    iconutil -c icns icon.iconset -o "${BUNDLE_DIR}/Contents/Resources/AppIcon.icns"
    rm -rf icon.iconset
fi

echo "==> Packaging into DMG with Drag-to-Applications support..."
rm -rf dmg_root
mkdir -p dmg_root
cp -r "${BUNDLE_DIR}" dmg_root/
ln -s /Applications dmg_root/Applications

rm -f "${DMG_PATH}"
hdiutil create -volname "${VOL_NAME}" -srcfolder dmg_root -ov -format UDZO "${DMG_PATH}"
rm -rf dmg_root

echo ""
echo "=========================================================="
echo "  SUCCESS! Your DMG is ready at:"
echo "  $(pwd)/${DMG_PATH}"
echo "=========================================================="
