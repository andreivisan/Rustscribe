use std::process::Command;

#[test]
fn help_succeeds_without_models_or_media() {
    let output = Command::new(env!("CARGO_BIN_EXE_rustscribe"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for argument in [
        "whisper",
        "cohere",
        "<MODEL_PATH>",
        "<INPUT_PATH>",
        "--output",
    ] {
        assert!(help.contains(argument), "missing {argument}: {help}");
    }
}

#[test]
fn missing_arguments_exit_with_usage_instead_of_starting_inference() {
    let output = Command::new(env!("CARGO_BIN_EXE_rustscribe"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    assert!(output.stdout.is_empty());
}

#[test]
fn unknown_engine_is_rejected_before_loading_a_model() {
    let output = Command::new(env!("CARGO_BIN_EXE_rustscribe"))
        .args(["unknown", "missing-model", "missing.webm"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("invalid value 'unknown'"), "{error}");
    assert!(error.contains("whisper"));
    assert!(error.contains("cohere"));
}
