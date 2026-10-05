#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build --release --features desktop --bin rustscribe-desktop

bundle="$PWD/target/Rustscribe.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp target/release/rustscribe-desktop "$bundle/Contents/MacOS/Rustscribe"
cp docs/images/icon.png "$bundle/Contents/Resources/Rustscribe.png"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Rustscribe</string>
  <key>CFBundleDisplayName</key><string>Rustscribe</string>
  <key>CFBundleIdentifier</key><string>com.rustscribe.desktop</string>
  <key>CFBundleExecutable</key><string>Rustscribe</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>CFBundleIconFile</key><string>Rustscribe.png</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSRequiresAquaSystemAppearance</key><false/>
</dict></plist>
PLIST
# Ad-hoc signing supports local launches; public distribution needs notarization.
codesign --force --deep --sign - "$bundle"
printf 'Built %s\n' "$bundle"
