#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
image="${1:?usage: verify-macos-dmg.sh /path/to/Rustscribe.dmg}"
hdiutil verify "$image"
mount_dir=$(mktemp -d "$PWD/target/dmg-check.XXXXXX")
mounted=false
cleanup() {
  if [[ "$mounted" == true ]]; then hdiutil detach "$mount_dir"; fi
  rmdir "$mount_dir"
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -noautoopen -mountpoint "$mount_dir" "$image"
mounted=true
[[ "$(readlink "$mount_dir/Applications")" == /Applications ]]
test -s "$mount_dir/READ ME FIRST.txt"
test -s "$mount_dir/LICENSE.txt"
bash scripts/verify-macos-bundle.sh "$mount_dir/Rustscribe.app"
printf 'Verified DMG contents and Applications shortcut.\n'
