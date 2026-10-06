use rustscribe::api::{EngineKind, Transcript, render_markdown, write_markdown};
use std::{fs, io::ErrorKind, sync::Barrier, thread};

fn transcript(text: &str) -> Transcript {
    Transcript {
        source: "lesson.webm".into(),
        engine: EngineKind::Cohere,
        text: text.into(),
        extraction_duration: Default::default(),
        transcription_duration: Default::default(),
    }
}

#[test]
fn exports_a_complete_document_and_creates_parent_directories() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("notes/lecții/my transcript.md");
    let transcript = transcript("A lesson about Rust.\n\nA second paragraph.");

    write_markdown(&transcript, &output).unwrap();

    let markdown = fs::read_to_string(&output).unwrap();
    assert_eq!(markdown, render_markdown(&transcript));
    assert!(markdown.contains("**Engine:** Cohere"));
    assert_eq!(fs::read_dir(output.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn existing_notes_survive_a_failed_export_without_temporary_files() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("notes.md");
    fs::write(&output, "Hand-edited notes — keep these.").unwrap();

    let error = write_markdown(&transcript("Replacement."), &output).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "Hand-edited notes — keep these."
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn invalid_parent_is_reported_without_modifying_it() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().join("not-a-directory");
    fs::write(&parent, "keep").unwrap();

    assert!(write_markdown(&transcript("Hello."), &parent.join("notes.md")).is_err());
    assert_eq!(fs::read_to_string(parent).unwrap(), "keep");
}

#[test]
fn concurrent_exports_have_one_winner_and_never_mix_documents() {
    const WRITERS: usize = 6;
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("notes.md");
    let barrier = Barrier::new(WRITERS);
    let results = thread::scope(|scope| {
        let handles: Vec<_> = (0..WRITERS)
            .map(|index| {
                let output = &output;
                let barrier = &barrier;
                scope.spawn(move || {
                    let document = transcript(&format!("Writer {index}. ").repeat(1000));
                    barrier.wait();
                    (
                        render_markdown(&document),
                        write_markdown(&document, output),
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    let winners: Vec<_> = results
        .iter()
        .filter(|(_, result)| result.is_ok())
        .collect();
    assert_eq!(winners.len(), 1);
    assert_eq!(fs::read_to_string(output).unwrap(), winners[0].0);
    for (_, result) in results.iter().filter(|(_, result)| result.is_err()) {
        assert_eq!(
            result.as_ref().unwrap_err().kind(),
            ErrorKind::AlreadyExists
        );
    }
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn exports_to_a_bare_filename() {
    // A child test process gives this case its own working directory. Changing the
    // parent's cwd would race the other tests because cwd is process-wide.
    const CHILD: &str = "RUSTSCRIBE_TEST_BARE_FILENAME";
    if std::env::var_os(CHILD).is_some() {
        write_markdown(&transcript("Hello."), std::path::Path::new("notes.md")).unwrap();
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "exports_to_a_bare_filename", "--nocapture"])
        .env(CHILD, "1")
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("notes.md")).unwrap(),
        render_markdown(&transcript("Hello."))
    );
}
