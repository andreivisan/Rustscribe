//! Opt-in checks of the real inference backends. Never download weights in tests.
use rustscribe::api::{EngineKind, TranscriptionEngine, render_markdown, write_markdown};
use std::{env, path::PathBuf};

fn transcribe_and_export(kind: EngineKind, model_variable: &str) {
    let model = PathBuf::from(env::var_os(model_variable).unwrap_or_else(|| {
        panic!("set {model_variable} to an already-downloaded model; see CONTRIBUTING.md")
    }));
    let media = PathBuf::from(env::var_os("RUSTSCRIBE_TEST_MEDIA").expect(
        "set RUSTSCRIBE_TEST_MEDIA to a short English speech recording; see CONTRIBUTING.md",
    ));
    let mut engine = TranscriptionEngine::load(kind, &model).unwrap();
    let transcript = engine.transcribe(&media).unwrap();
    assert!(
        !transcript.text.trim().is_empty(),
        "use a recording containing speech"
    );
    assert_eq!(transcript.source, media);
    assert_eq!(transcript.engine.to_string(), kind.to_string());

    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("transcript.md");
    write_markdown(&transcript, &output).unwrap();
    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        render_markdown(&transcript)
    );
}

#[test]
#[ignore = "requires local Whisper weights and an English speech recording"]
fn whisper_transcribes_and_exports() {
    transcribe_and_export(EngineKind::Whisper, "RUSTSCRIBE_TEST_WHISPER_MODEL");
}

#[test]
#[ignore = "requires local Cohere weights and an English speech recording"]
fn cohere_transcribes_and_exports() {
    transcribe_and_export(EngineKind::Cohere, "RUSTSCRIBE_TEST_COHERE_MODEL");
}
