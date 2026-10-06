#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
  echo "This release is built on an Apple Silicon Mac." >&2
  exit 1
fi

export MACOSX_DEPLOYMENT_TARGET=15.0
export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
target=aarch64-apple-darwin
version=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
bundle="$PWD/target/Rustscribe.app"
about="$PWD/target/release-tools/cargo-about"
if [[ ! -x "$about" ]]; then
  echo "Run bash scripts/install-release-tools.sh first." >&2
  exit 1
fi

cargo build --locked --release --target "$target" --features desktop --bin rustscribe-desktop
staging=$(mktemp -d "$PWD/target/macos-bundle.XXXXXX")
trap 'rm -rf "$staging"' EXIT
app="$staging/Rustscribe.app"
resources="$app/Contents/Resources"
mkdir -p "$app/Contents/MacOS" "$resources"
cp "target/$target/release/rustscribe-desktop" "$app/Contents/MacOS/Rustscribe"
cp LICENSE "$resources/LICENSE.txt"
cp packaging/macos/INSTALL.txt "$resources/INSTALL.txt"

# Convert the existing artwork to Apple's icon format without changing the source.
xcrun swift -module-cache-path "$PWD/target/swift-module-cache" scripts/macos-icon.swift \
  docs/images/icon.png "$staging/Rustscribe.iconset"
iconutil -c icns "$staging/Rustscribe.iconset" -o "$resources/Rustscribe.icns"

"$about" generate --locked --fail --features desktop packaging/licenses.hbs -o "$resources/ThirdPartyLicenses.html"
mkdir -p "$resources/NativeLicenses"
cp packaging/licenses/* "$resources/NativeLicenses/"

python3 - "$app/Contents/Info.plist" "$version" <<'PY'
import plistlib
import sys

with open(sys.argv[1], "wb") as output:
    plistlib.dump({
        "CFBundleName": "Rustscribe",
        "CFBundleDisplayName": "Rustscribe",
        "CFBundleIdentifier": "com.rustscribe.desktop",
        "CFBundleExecutable": "Rustscribe",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": sys.argv[2],
        "CFBundleVersion": sys.argv[2],
        "CFBundleIconFile": "Rustscribe.icns",
        "LSMinimumSystemVersion": "15.0",
        "LSApplicationCategoryType": "public.app-category.productivity",
        "NSHighResolutionCapable": True,
        "NSRequiresAquaSystemAppearance": False,
    }, output)
PY

# An ad-hoc signature requires no certificate and does not provide Developer ID
# verification or notarization. The DMG is explicitly labelled unsigned.
codesign --force --sign - "$app"
bash scripts/verify-macos-bundle.sh "$app"
# Replace only the generated bundle; the application in /Applications is untouched.
if [[ -e "$bundle" ]]; then rm -rf "$bundle"; fi
mv "$app" "$bundle"
printf 'Built %s\n' "$bundle"
