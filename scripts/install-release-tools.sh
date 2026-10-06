#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
  echo "Release tools currently target Apple Silicon macOS." >&2
  exit 1
fi
mkdir -p target/release-tools
archive=$(mktemp "$PWD/target/release-tools/cargo-about.XXXXXX")
trap 'rm -f "$archive"' EXIT
curl --fail --location --retry 3 \
  https://github.com/EmbarkStudios/cargo-about/releases/download/0.9.2/cargo-about-0.9.2-aarch64-apple-darwin.tar.gz \
  --output "$archive"
printf '%s  %s\n' ae72f0df0c399a1e96336f696fa55b1b28679fd725632eba8cf8e4568467cc3e "$archive" | shasum -a 256 -c -
tar -xzf "$archive" --strip-components 1 -C target/release-tools cargo-about-0.9.2-aarch64-apple-darwin/cargo-about
target/release-tools/cargo-about --version
