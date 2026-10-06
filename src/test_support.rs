use std::{path::Path, process::Command};

/// Tiny generated media fixtures keep private videos and binary assets out of Git.
pub(crate) fn video(path: &Path, audio: bool) {
    let mut command = Command::new(crate::api::audio::media_program("ffmpeg"));
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n"]);
    command.args(["-f", "lavfi", "-i", "color=c=blue:s=64x64:r=10"]);
    if audio {
        command.args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000"]);
        command.args(["-ac", "2", "-c:a", "libopus"]);
    }
    command.args(["-t", "0.3", "-c:v", "libvpx", "-threads", "1"]);
    let output = command
        .arg(path)
        .output()
        .expect("tests require FFmpeg; run `brew install ffmpeg`");
    assert!(
        output.status.success(),
        "fixture generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
