use super::*;
use std::fs;

fn cohere_directory(root: &Path) -> PathBuf {
    let path = root.join("cohere-int8");
    fs::create_dir_all(&path).unwrap();
    for name in [
        "cohere-encoder.int8.onnx",
        "cohere-decoder.int8.onnx",
        "tokens.txt",
    ] {
        fs::write(path.join(name), b"test placeholder").unwrap();
    }
    path
}

#[test]
fn missing_settings_use_defaults_without_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let (settings, error) = Settings::load_from(directory.path(), []);
    assert!(error.is_none());
    assert_eq!(settings.model, Model::Whisper);
    assert!(settings.whisper.is_none());
    assert!(settings.cohere.is_none());
    assert!(settings.output_directory.is_none());
}

#[test]
fn corrupt_settings_report_the_problem_and_preserve_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, "{invalid json").unwrap();
    let (settings, error) = Settings::load_from(directory.path(), []);
    assert_eq!(settings.model, Model::Whisper);
    assert!(error.unwrap().contains("Could not read saved settings"));
    assert_eq!(fs::read_to_string(path).unwrap(), "{invalid json");
}

#[test]
fn older_settings_can_omit_fields_and_newer_settings_can_add_fields() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"model":"Cohere","future_setting":true}"#,
    )
    .unwrap();
    let (settings, error) = Settings::load_from(directory.path(), []);
    assert!(error.is_none());
    assert_eq!(settings.model, Model::Cohere);
    assert!(settings.path(Model::Whisper).is_none());
    assert!(settings.path(Model::Cohere).is_none());
}

#[test]
fn save_round_trips_paths_and_replaces_previous_preferences() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("nested/config");
    let mut original = Settings {
        model: Model::Cohere,
        whisper: Some("models/my whisper.bin".into()),
        cohere: Some("models/cohere-int8".into()),
        output_directory: Some("notes/lecții".into()),
    };
    original.save_to(&config).unwrap();
    original.model = Model::Whisper;
    original.save_to(&config).unwrap();
    let (saved, error) = Settings::load_from(&config, []);
    assert!(error.is_none());
    assert_eq!(saved.model, original.model);
    assert_eq!(saved.whisper, original.whisper);
    assert_eq!(saved.cohere, original.cohere);
    assert_eq!(saved.output_directory, original.output_directory);
    assert_eq!(fs::read_dir(config).unwrap().count(), 1);
}

#[test]
fn discovers_models_in_priority_order_without_overriding_explicit_paths() {
    let directory = tempfile::tempdir().unwrap();
    let roots = [
        directory.path().join("preferred"),
        directory.path().join("fallback"),
    ];
    for root in &roots {
        cohere_directory(root);
        fs::write(root.join("ggml-large-v3-turbo.bin"), b"test placeholder").unwrap();
    }
    let (settings, error) = Settings::load_from(directory.path(), roots.clone());
    assert!(error.is_none());
    assert_eq!(
        settings.whisper,
        Some(roots[0].join("ggml-large-v3-turbo.bin"))
    );
    assert_eq!(settings.cohere, Some(roots[0].join("cohere-int8")));

    let explicit = Settings {
        whisper: Some(directory.path().join("unmounted/custom.bin")),
        cohere: Some(directory.path().join("unmounted/custom-cohere")),
        ..Default::default()
    };
    explicit.save_to(directory.path()).unwrap();
    let (settings, error) = Settings::load_from(directory.path(), roots);
    assert!(error.is_none());
    assert_eq!(settings.whisper, explicit.whisper);
    assert_eq!(settings.cohere, explicit.cohere);
}

#[test]
fn cohere_discovery_requires_all_three_entry_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = cohere_directory(directory.path());
    assert!(Model::Cohere.valid_path(&path));
    fs::remove_file(path.join("tokens.txt")).unwrap();
    assert!(!Model::Cohere.valid_path(&path));
    let (settings, _) = Settings::load_from(directory.path(), [directory.path().to_path_buf()]);
    assert!(settings.cohere.is_none());
    assert!(!Model::Whisper.valid_path(&path));
    assert!(Model::Whisper.valid_path(&path.join("cohere-encoder.int8.onnx")));
}

#[test]
fn save_errors_do_not_destroy_existing_files() {
    let directory = tempfile::tempdir().unwrap();
    let blocked = directory.path().join("file-not-directory");
    fs::write(&blocked, b"keep").unwrap();
    assert!(Settings::default().save_to(&blocked).is_err());
    assert_eq!(fs::read(blocked).unwrap(), b"keep");
}
