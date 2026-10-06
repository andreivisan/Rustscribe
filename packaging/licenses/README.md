# Native dependency notices

These notices accompany the statically linked native libraries; Cargo dependency
notices are generated separately with `cargo-about` when building the app bundle.

- **ONNX Runtime 1.24.2**, selected by `ort-sys 2.0.0-rc.12`:
  [LICENSE](https://github.com/microsoft/onnxruntime/blob/v1.24.2/LICENSE) and
  [ThirdPartyNotices.txt](https://github.com/microsoft/onnxruntime/blob/v1.24.2/ThirdPartyNotices.txt).
- **whisper.cpp**, vendored by `whisper-rs-sys 0.15.0`: `whisper.cpp/LICENSE`
  from that exact crate's source package.

Refresh these files when upgrading the corresponding native dependencies.
FFmpeg and model weights are external and are not redistributed in this DMG.
