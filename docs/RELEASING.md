# macOS releases

The release currently supports **Apple Silicon, macOS 15+**. It is distributed as
an unsigned, unnotarized prerelease DMG. The application has an ad-hoc signature
for macOS code integrity; that is not an Apple Developer ID signature.

## Build locally

On an Apple Silicon Mac with Rust, Xcode command-line tools, CMake, FFmpeg and
Python 3 installed:

```sh
bash scripts/install-release-tools.sh
bash scripts/package-macos.sh
```

The tool installer downloads a pinned, SHA-256-verified `cargo-about` binary to
`target/release-tools/`. This is a build tool, not an app runtime dependency.
Dependency notices are generated from `Cargo.lock`; native dependency notices
are stored in `packaging/licenses/` and must be refreshed with native upgrades.

The package contains Rustscribe.app, an Applications shortcut, and installation
instructions. FFmpeg and model weights remain separate. The package script
verifies the app's architecture, code integrity, runtime library paths, resources,
and DMG contents, then generates a SHA-256 checksum under `target/releases/`.

## Publish to GitHub Releases

1. Update the version in `Cargo.toml` and `Cargo.lock`, and edit
   `packaging/macos/RELEASE-NOTES.md` for the release, including filenames.
2. Commit the changes and run the normal CI checks. Test the packaged app on a
   Mac: launch, choose models, transcribe, export, quit and reopen. CI does not
   validate native UI interaction or model accuracy.
3. Push a matching version tag, for example:

   ```sh
   git tag -a v0.1.0 -m "Rustscribe 0.1.0 macOS prerelease"
   git push origin v0.1.0
   ```

The [release workflow](../.github/workflows/release-macos.yml) checks the version,
runs formatting, Clippy and tests, builds and verifies the DMG, then publishes
the DMG and checksum as a **prerelease**. It needs only GitHub's automatically
provided token; no Apple credentials are used. An existing release is not
overwritten: use a new version for a changed package.

To test packaging without publishing, run **Release macOS → Run workflow** on a
branch in GitHub Actions. Branch runs upload workflow artifacts only. Running it
on a version tag publishes a prerelease after verification.

## Installing an unsigned build

Open the DMG and drag Rustscribe to Applications. After trying to open the app,
macOS may offer **Open Anyway** in **System Settings → Privacy & Security**.
Use that exception only for a download you trust; see
[Apple's instructions](https://support.apple.com/en-us/102445).
Do not disable Gatekeeper globally. Developer ID signing and notarization can be
added later when a certificate is available.
