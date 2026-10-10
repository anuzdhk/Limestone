#!/usr/bin/env bash
set -e

# ==============================================================================
# LimeStone Pro - macOS .app Bundle and DMG Generator
# ==============================================================================

VERSION="0.1.0-beta"
APP_NAME="LimeStone"
VOL_NAME="LimeStone Pro"
BUNDLE_DIR="dist/${APP_NAME}.app"
DMG_PATH="dist/${APP_NAME}-${VERSION}-Universal.dmg"

echo "==> Building universal macOS binaries..."
cargo build --release -p adjust-gui --target aarch64-apple-darwin
cargo build --release -p adjust-gui --target x86_64-apple-darwin

echo "==> Creating .app bundle structure..."
rm -rf "dist"
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

echo "==> Liping universal binary..."
lipo -create \
  target/aarch64-apple-darwin/release/adjust-gui \
  target/x86_64-apple-darwin/release/adjust-gui \
  -output "${BUNDLE_DIR}/Contents/MacOS/${APP_NAME}"
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
    <string>LimeStone</string>
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
    mkdir -p icon.iconset
    sips -z 16 16     logo/logo.png --out icon.iconset/icon_16x16.png
    sips -z 32 32     logo/logo.png --out icon.iconset/icon_16x16@2x.png
    sips -z 32 32     logo/logo.png --out icon.iconset/icon_32x32.png
    sips -z 64 64     logo/logo.png --out icon.iconset/icon_32x32@2x.png
    sips -z 128 128   logo/logo.png --out icon.iconset/icon_128x128.png
    sips -z 256 256   logo/logo.png --out icon.iconset/icon_128x128@2x.png
    iconutil -c icns icon.iconset -o "${BUNDLE_DIR}/Contents/Resources/AppIcon.icns"
    rm -rf icon.iconset
fi

echo "==> Generating .dmg image with hdiutil..."
rm -rf dmg_root
mkdir -p dmg_root
cp -r "${BUNDLE_DIR}" dmg_root/
ln -s /Applications dmg_root/Applications

hdiutil create -volname "${VOL_NAME}" -srcfolder dmg_root -ov -format UDZO "${DMG_PATH}"
rm -rf dmg_root

echo "==> Successfully created ${DMG_PATH}"
