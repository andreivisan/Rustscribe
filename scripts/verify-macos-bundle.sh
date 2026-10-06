#!/bin/bash
set -euo pipefail
app="${1:?usage: verify-macos-bundle.sh /path/to/Rustscribe.app}"
binary="$app/Contents/MacOS/Rustscribe"
plutil -lint "$app/Contents/Info.plist"
codesign --verify --deep --strict --verbose=2 "$app"
[[ "$(lipo -archs "$binary")" == arm64 ]]
[[ "$(/usr/libexec/PlistBuddy -c 'Print LSMinimumSystemVersion' "$app/Contents/Info.plist")" == 15.0 ]]
for resource in Rustscribe.icns LICENSE.txt INSTALL.txt ThirdPartyLicenses.html; do
  test -s "$app/Contents/Resources/$resource"
done

# Catch accidental dependencies on the build machine's Homebrew or Cargo paths.
otool -L "$binary" | python3 -c '
import sys
libraries = [line.strip().split(" (", 1)[0] for line in sys.stdin.readlines()[1:]]
unexpected = [path for path in libraries if not path.startswith(("/System/Library/", "/usr/lib/"))]
if unexpected:
    sys.exit("Non-system runtime dependencies: " + ", ".join(unexpected))
print("Verified arm64 bundle; all runtime libraries are provided by macOS.")
'
