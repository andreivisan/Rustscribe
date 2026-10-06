#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/bundle-macos.sh
version=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
output="$PWD/target/releases"
mkdir -p "$output"
staging=$(mktemp -d "$PWD/target/macos-dmg.XXXXXX")
trap 'rm -rf "$staging"' EXIT
ditto target/Rustscribe.app "$staging/Rustscribe.app"
ln -s /Applications "$staging/Applications"
cp packaging/macos/INSTALL.txt "$staging/READ ME FIRST.txt"
cp LICENSE "$staging/LICENSE.txt"

name="Rustscribe-$version-macos-arm64-unsigned.dmg"
hdiutil create -volname "Rustscribe $version" -srcfolder "$staging" \
  -fs HFS+ -format UDZO -ov "$output/$name"
bash scripts/verify-macos-dmg.sh "$output/$name"
cd "$output"
shasum -a 256 "$name" > "$name.sha256"
shasum -a 256 -c "$name.sha256"
printf 'Packaged %s\n' "$output/$name"
