use std::{path::Path, process::Command};

pub(super) fn extract_audio(input_path: &Path, output_path: &Path) -> std::io::Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-n")
        .arg("-i")
        .arg(input_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(output_path)
        .output()?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "FFmpeg failed ({}): {}",
            output.status,
            error_msg.trim(),
        )));
    }
    Ok(())
}
