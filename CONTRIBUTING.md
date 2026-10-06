# Contributing to Rustscribe

Rustscribe currently targets macOS on Apple Silicon. The reusable API, CLI, and
GPUI desktop app live in the same crate; the desktop is an optional Cargo feature.

## Development setup

1. Fork the repository, clone your fork, and create a branch for your change.
2. Install Rust through [rustup](https://rustup.rs/). CI uses Rust **1.98.0**.
3. Install Xcode or its command-line tools, then `brew install cmake ffmpeg`.
4. Run the checks below before opening a pull request.

Whisper compiles native C/C++ code and links Apple's Metal frameworks, even when
the tests use a fake speech model. A first build also downloads Cargo dependencies
and ONNX Runtime build artifacts. The regular test suite needs **no model weights,
API keys, private media, or running GUI**. Once built, its tests operate locally.

If the compiler and macOS SDK disagree, find the SDK belonging to your selected
developer tools with:

```sh
xcrun --sdk macosx --show-sdk-path
```

Put that path in an untracked `.cargo/config.toml`:

```toml
[env]
SDKROOT = "/absolute/path/printed/by/xcrun"
```

Never commit a machine-specific SDK path. CI discovers its SDK automatically.

## Required checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo build --locked --all-features --bins
```

The [CI workflow](.github/workflows/ci.yml) runs these checks for pushes and pull
requests on an Apple Silicon macOS runner. Two real-model smoke tests are ignored
by default; this is intentional, not a missing dependency. To match CI exactly,
install `1.98.0` with rustup and use `cargo +1.98.0` for the commands above.

## Where tests belong

| Area | Location | What it checks |
| --- | --- | --- |
| Audio extraction | `src/api/audio/tests.rs` | Real FFmpeg decoding, WebM, required WAV format, invalid inputs, preserving existing files |
| Chunked transcription | `src/api/engine/tests.rs` | Sample normalization, order, bounded chunks, short tails, invalid WAVs, model failures, extraction into transcription |
| Markdown rendering | `src/api/markdown/tests.rs` | Escaping, Unicode, metadata, whitespace and paragraph boundaries |
| CLI parsing | `src/cli.rs` | Engines, paths with spaces and output flags |
| Filesystem exports | `tests/markdown_export.rs` | Public API, nested/relative destinations, atomic no-clobber behavior and concurrent writers |
| CLI processes | `tests/cli.rs` | Help and argument errors through the compiled binary |
| Desktop settings | `src/gpui/settings/tests.rs` | JSON compatibility, persistence failures and model discovery |
| Desktop worker | `src/gpui/worker.rs` | Cancellation, export recovery, event IDs and real media inspection |
| Real model smoke tests | `tests/model_smoke.rs` | Optional local inference through Markdown export |

Keep tests deterministic and independent. Generate small media fixtures in a
temporary directory instead of adding recordings or model weights to Git. The
`RecordingModel` implements `SpeechModel` and records samples without running
inference; it tests Rustscribe's pipeline rather than model accuracy. Avoid
changing the process-wide working directory or environment in parallel tests.

## Optional real-model tests

Use already-downloaded weights and a short recording of English speech. Run one
backend at a time to avoid loading both large models together:

```sh
RUSTSCRIBE_TEST_WHISPER_MODEL="$PWD/models/ggml-large-v3-turbo.bin" \
RUSTSCRIBE_TEST_MEDIA="/absolute/path/to/speech.webm" \
cargo test --release --locked --test model_smoke whisper_transcribes_and_exports -- --ignored --exact

RUSTSCRIBE_TEST_COHERE_MODEL="$PWD/models/cohere-int8" \
RUSTSCRIBE_TEST_MEDIA="/absolute/path/to/speech.webm" \
cargo test --release --locked --test model_smoke cohere_transcribes_and_exports -- --ignored --exact
```

These validate loading, inference, and export, not transcription accuracy. Listen
to the recording and compare the output when changing model settings or chunking.
The tests write exports into temporary directories and never replace your notes.

## Desktop changes and pull requests

For UI changes, also run the app and check file selection/drop, transcription,
cancellation, switching engines, and export. CI checks desktop compilation and
worker/settings behavior; it does not click native dialogs or verify rendering.

Keep pull requests focused. Explain the problem, the resulting behavior, and the
checks you ran. Include screenshots for visual changes and a regression test for
bugs when practical. Keep `Cargo.lock` tracked, minimize new dependencies, and do
not include downloaded models, private recordings, generated transcripts, or
credentials. If you report a bug, include the macOS version, Mac architecture,
engine/model, reproduction steps, and relevant logs with private paths removed.

Contributions to Rustscribe are made under the project's [MIT license](LICENSE).
