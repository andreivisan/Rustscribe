use super::*;
use crate::test_support;

#[test]
fn webm_is_decoded_to_the_models_required_audio_format() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("lecție [test] with spaces.webm");
    let output = directory.path().join("audio.wav");
    test_support::video(&input, true);

    extract_audio(&input, &output).unwrap();

    let mut reader = hound::WavReader::open(&output).unwrap();
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 16_000);
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.bits_per_sample, 16);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
    assert!((4_000..6_000).contains(&reader.duration()));
    assert!(reader.samples::<i16>().any(|sample| sample.unwrap() != 0));
}

#[test]
fn video_without_audio_returns_an_extraction_error() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("silent.webm");
    test_support::video(&input, false);

    let error = extract_audio(&input, &directory.path().join("audio.wav")).unwrap_err();

    assert!(error.to_string().contains("FFmpeg failed"), "{error}");
}

#[test]
fn missing_and_corrupt_input_report_ffmpeg_errors() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("missing.webm");
    let output = directory.path().join("audio.wav");
    let error = extract_audio(&input, &output).unwrap_err();
    assert!(error.to_string().contains("FFmpeg failed"), "{error}");

    std::fs::write(&input, b"not media").unwrap();
    let error = extract_audio(&input, &output).unwrap_err();
    assert!(error.to_string().contains("FFmpeg failed"), "{error}");
}

#[test]
fn an_existing_output_is_never_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.webm");
    let output = directory.path().join("audio.wav");
    test_support::video(&input, true);
    std::fs::write(&output, b"original contents").unwrap();

    // FFmpeg versions differ in the exit status used for a refused overwrite.
    let _ = extract_audio(&input, &output);
    assert_eq!(std::fs::read(&output).unwrap(), b"original contents");
}
