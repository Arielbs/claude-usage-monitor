#!/bin/bash
# Generate all Tauri app icons from the source image

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
ICONS_DIR="$ROOT_DIR/src-tauri/icons"
SRC="$ROOT_DIR/assets/claude_usage_monitor_transparent.png"

cd "$ICONS_DIR"

echo "Generating PNG icons..."
sips -z 32 32 "$SRC" --out 32x32.png
sips -z 128 128 "$SRC" --out 128x128.png
sips -z 256 256 "$SRC" --out "128x128@2x.png"
sips -z 1024 1024 "$SRC" --out icon.png

echo "Creating iconset for .icns..."
mkdir -p icon.iconset
sips -z 16 16 "$SRC" --out icon.iconset/icon_16x16.png
sips -z 32 32 "$SRC" --out icon.iconset/icon_16x16@2x.png
sips -z 32 32 "$SRC" --out icon.iconset/icon_32x32.png
sips -z 64 64 "$SRC" --out icon.iconset/icon_32x32@2x.png
sips -z 128 128 "$SRC" --out icon.iconset/icon_128x128.png
sips -z 256 256 "$SRC" --out icon.iconset/icon_128x128@2x.png
sips -z 256 256 "$SRC" --out icon.iconset/icon_256x256.png
sips -z 512 512 "$SRC" --out icon.iconset/icon_256x256@2x.png
sips -z 512 512 "$SRC" --out icon.iconset/icon_512x512.png
sips -z 1024 1024 "$SRC" --out icon.iconset/icon_512x512@2x.png

echo "Generating .icns file..."
iconutil -c icns icon.iconset -o icon.icns
rm -rf icon.iconset

echo "Done! Icons generated in src-tauri/icons/"
