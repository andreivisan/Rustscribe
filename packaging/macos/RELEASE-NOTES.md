Rustscribe's first macOS prerelease: local video/audio transcription with Whisper
or Cohere, a native GPUI desktop app, and Markdown export.

**Download:** `Rustscribe-0.1.0-macos-arm64-unsigned.dmg`

- Apple Silicon Macs (M1 or later), macOS 15 or later.
- Drag **Rustscribe.app** into **Applications**, then eject the DMG.
- No Developer ID signature or notarization. The app is ad-hoc signed and the DMG
  is labelled unsigned. After attempting to open the app, use **System Settings →
  Privacy & Security → Open Anyway** if you trust this download.
  [Apple's instructions](https://support.apple.com/en-us/102445).
- Install FFmpeg/ffprobe separately (`brew install ffmpeg`) and select an already
  downloaded model in Settings. Neither FFmpeg nor model weights are bundled.
- Whisper uses Metal; Cohere currently uses the CPU. Transcription is local and
  currently configured for English. Rust and Xcode are not needed to run the app.
- Export before quitting: the queue and transcripts are session-only.

The accompanying `.dmg.sha256` file verifies the download:

```sh
shasum -a 256 -c Rustscribe-0.1.0-macos-arm64-unsigned.dmg.sha256
```

This is an early release. Please include your macOS version, Mac model, and
transcription engine when reporting a problem. Windows, Linux, and Intel Mac
packages are not part of this release.
