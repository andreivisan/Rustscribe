<p align="center">
  <img src="./docs/images/icon.png" alt="Rustscribe" width="50%">
</p>


# Welcome to Rustscribe

Rustscribe is a native macOS app for turning local video and audio into readable Markdown. Built in Rust with GPUI, with a media shelf inspired by [DropFile](https://dribbble.com/shots/22619137-DropFile-MacOS-App).

## Run the desktop app

You need Rust, the Xcode command-line tools, CMake, and FFmpeg (`brew install cmake ffmpeg`). Model files stay on your Mac. GPUI compiles its Metal shaders at launch, so the separate Xcode Metal Toolchain download is not required.

```sh
cargo run --release --features desktop --bin rustscribe-desktop
```

You can also pass media paths after `--` to open them on launch:

```sh
cargo run --release --features desktop --bin rustscribe-desktop -- "/path/to/video.webm"
```

Drop files onto the window, or use **Choose files** / **⌘O**. Select Whisper or Cohere, then transcribe the queue or just the selected file. Read and copy the transcript, export Markdown, or use **Save as…** to choose a destination. Existing exports are preserved; quick exports use numbered filenames when needed.

**⌘,** opens settings, **⌘Return** transcribes the queue, and **⌘⇧E** exports completed transcripts. A queue can stop after its current file; an in-progress inference call is not interrupted. The media shelf is session-only, so export transcripts before quitting.

### Local models

- **Whisper:** a whisper.cpp `.bin` model, such as `models/ggml-large-v3-turbo.bin`. Uses Metal on Apple Silicon.
- **Cohere:** an int8 ONNX folder, such as `models/cohere-int8`, containing the encoder, decoder, external weight files, and `tokens.txt`. Uses the CPU with the current transcribe-rs integration.

Models in the project's `models/` directory are discovered automatically. Otherwise choose them in Settings. Model paths, engine choice, and export folder are saved to `~/Library/Application Support/Rustscribe/settings.json`. No media or transcript content is uploaded. Transcription currently uses English and the core API's adaptive audio chunking.

FFmpeg and ffprobe are discovered on `PATH`, then in the standard Apple Silicon and Intel Homebrew locations. For a custom installation set `RUSTSCRIBE_FFMPEG` and `RUSTSCRIBE_FFPROBE` to the executable paths.

### Build a Mac app bundle

```sh
bash scripts/bundle-macos.sh
open target/Rustscribe.app
```

The bundle uses the existing Rustscribe icon and is ad-hoc signed for local use. Models and FFmpeg are external dependencies; select model paths in Settings if moving the app away from the repository. Distribution to other Macs needs the usual Developer ID signing and notarization.

If native builds select mismatched Apple SDKs, use a local `.cargo/config.toml` (ignored by Git) with `SDKROOT` pointing to the SDK under your selected Xcode installation. Obtain that path with `xcrun --sdk macosx --show-sdk-path`.

## CLI

The CLI remains the default binary and does not build GPUI:

```sh
cargo run --release -- whisper models/ggml-large-v3-turbo.bin "/path/to/video.webm"
cargo run --release -- cohere models/cohere-int8 "/path/to/video.webm" -o transcripts/video.md
```

## Code layout

- `src/api/`: reusable extraction, chunked transcription, and atomic Markdown export.
- `src/cli.rs`: Clap command-line interface.
- `src/gpui/views.rs`: desktop appearance and interactions.
- `src/gpui/mod.rs`: window, queue state, and native dialogs.
- `src/gpui/worker.rs`: model ownership, media inspection, and export off the UI thread.
- `src/gpui/settings.rs`: model discovery and persisted preferences.

The worker reuses a loaded model for sequential files and releases it before switching engines. This keeps the UI responsive without duplicating large model allocations. Temporary audio and thumbnails are cleaned up automatically.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --features desktop -- -D warnings
cargo test --features desktop --lib
```
