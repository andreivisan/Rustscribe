//! A single worker owns the model. No inference or media decoding runs on GPUI's thread.
use super::settings::Model;
use crate::api::{Transcript, TranscriptionEngine, audio::media_program, write_markdown};
use std::{
    path::PathBuf,
    process::Command as Process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
};
use tempfile::TempDir;

pub(super) enum Command {
    Inspect {
        id: usize,
        path: PathBuf,
        cache: Arc<TempDir>,
    },
    Transcribe {
        jobs: Vec<(usize, PathBuf)>,
        model: Model,
        path: PathBuf,
        cancel: Arc<AtomicBool>,
    },
    Export {
        id: usize,
        transcript: Arc<Transcript>,
        path: PathBuf,
        unique: bool,
    },
}

pub(super) enum Event {
    Inspected {
        id: usize,
        duration: Option<f64>,
        thumbnail: Option<PathBuf>,
    },
    Started {
        id: usize,
        loading: bool,
    },
    Completed {
        id: usize,
        result: Result<Arc<Transcript>, String>,
    },
    BatchFinished,
    Exported {
        id: usize,
        result: Result<PathBuf, String>,
    },
}

pub(super) fn spawn() -> (Sender<Command>, Receiver<Event>) {
    let (commands, receive) = mpsc::channel();
    let (events, results) = mpsc::channel();
    std::thread::Builder::new()
        .name("rustscribe-inference".into())
        .spawn(move || {
            let mut loaded: Option<(Model, PathBuf, TranscriptionEngine)> = None;
            for command in receive {
                match command {
                    Command::Inspect { id, path, cache } => {
                        let (duration, thumbnail) =
                            inspect(&path, &cache.path().join(format!("{id}.png")));
                        if events
                            .send(Event::Inspected {
                                id,
                                duration,
                                thumbnail,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Command::Transcribe {
                        jobs,
                        model,
                        path,
                        cancel,
                    } => {
                        for (id, input) in jobs {
                            if cancel.load(Ordering::Relaxed) {
                                break;
                            }
                            let needs_load = loaded.as_ref().is_none_or(|(kind, old_path, _)| {
                                *kind != model || *old_path != path
                            });
                            if events
                                .send(Event::Started {
                                    id,
                                    loading: needs_load,
                                })
                                .is_err()
                            {
                                return;
                            }
                            if needs_load {
                                // Release the previous model before allocating another multi-GB model.
                                loaded = None;
                                match TranscriptionEngine::load(model.engine(), &path) {
                                    Ok(engine) => loaded = Some((model, path.clone(), engine)),
                                    Err(error) => {
                                        let _ = events.send(Event::Completed {
                                            id,
                                            result: Err(format!(
                                                "Could not load {}: {error}",
                                                model.name()
                                            )),
                                        });
                                        break;
                                    }
                                }
                            }
                            let _ = events.send(Event::Started { id, loading: false });
                            if let Some((_, _, engine)) = loaded.as_mut() {
                                let result = engine
                                    .transcribe(&input)
                                    .map(Arc::new)
                                    .map_err(|error| error.to_string());
                                if events.send(Event::Completed { id, result }).is_err() {
                                    return;
                                }
                            }
                        }
                        if events.send(Event::BatchFinished).is_err() {
                            break;
                        }
                    }
                    Command::Export {
                        id,
                        transcript,
                        path,
                        unique,
                    } => {
                        let result =
                            export(&transcript, path, unique).map_err(|error| error.to_string());
                        if events.send(Event::Exported { id, result }).is_err() {
                            break;
                        }
                    }
                }
            }
        })
        .expect("could not start transcription worker");
    (commands, results)
}

fn inspect(path: &std::path::Path, thumbnail: &std::path::Path) -> (Option<f64>, Option<PathBuf>) {
    let duration = Process::new(media_program("ffprobe"))
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<f64>()
                .ok()
        })
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0);
    let thumbnail = Process::new(media_program("ffmpeg"))
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-n",
            "-ss",
            "0",
            "-i",
        ])
        .arg(path)
        .args([
            "-map",
            "0:v:0",
            "-frames:v",
            "1",
            "-vf",
            "scale=640:360:force_original_aspect_ratio=increase,crop=640:360",
        ])
        .arg(thumbnail)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|_| thumbnail.to_path_buf());
    (duration, thumbnail)
}

fn export(transcript: &Transcript, path: PathBuf, unique: bool) -> std::io::Result<PathBuf> {
    let mut candidate = path.clone();
    for suffix in 1..=10_000 {
        match write_markdown(transcript, &candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if unique && error.kind() == std::io::ErrorKind::AlreadyExists => {
                let mut name = path.file_stem().unwrap_or_default().to_os_string();
                name.push(format!(" ({suffix}).md"));
                candidate = path.with_file_name(name);
            }
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::other(
        "Too many exports with the same name; choose another folder.",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transcript() -> Arc<Transcript> {
        Arc::new(Transcript {
            source: "lecture.webm".into(),
            engine: crate::api::EngineKind::Whisper,
            text: "Local transcription.".into(),
            extraction_duration: Default::default(),
            transcription_duration: Default::default(),
        })
    }

    fn receive(events: &Receiver<Event>) -> Event {
        events
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("worker did not respond")
    }

    #[test]
    fn cancelled_queue_finishes_without_loading_models_and_worker_still_exports() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("notes.md");
        let (commands, events) = spawn();
        commands
            .send(Command::Transcribe {
                jobs: vec![(7, "missing.webm".into())],
                model: Model::Whisper,
                path: "nonexistent-model.bin".into(),
                cancel: Arc::new(AtomicBool::new(true)),
            })
            .unwrap();
        // No Started or Completed event: cancellation must happen before loading.
        assert!(matches!(receive(&events), Event::BatchFinished));
        commands
            .send(Command::Export {
                id: 42,
                transcript: transcript(),
                path: path.clone(),
                unique: false,
            })
            .unwrap();
        assert!(
            matches!(receive(&events), Event::Exported { id: 42, result: Ok(saved) } if saved == path)
        );
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            crate::api::render_markdown(&transcript())
        );
        drop(commands);
        assert!(matches!(
            events.recv_timeout(std::time::Duration::from_secs(10)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn export_error_is_reported_and_does_not_kill_the_worker() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("notes.md");
        std::fs::write(&path, "edited notes").unwrap();
        let (commands, events) = spawn();
        for unique in [false, true] {
            commands
                .send(Command::Export {
                    id: 3,
                    transcript: transcript(),
                    path: path.clone(),
                    unique,
                })
                .unwrap();
            match receive(&events) {
                Event::Exported { id: 3, result } if unique => {
                    assert_eq!(result.unwrap(), directory.path().join("notes (1).md"));
                }
                Event::Exported { id: 3, result } => assert!(result.is_err()),
                _ => panic!("expected an export event for file 3"),
            }
        }
        assert_eq!(std::fs::read_to_string(path).unwrap(), "edited notes");
    }

    #[test]
    fn inspection_returns_duration_and_a_real_thumbnail() {
        let directory = Arc::new(tempfile::tempdir().unwrap());
        let path = directory.path().join("clip.webm");
        crate::test_support::video(&path, true);
        let (commands, events) = spawn();
        commands
            .send(Command::Inspect {
                id: 9,
                path,
                cache: Arc::clone(&directory),
            })
            .unwrap();
        match receive(&events) {
            Event::Inspected {
                id: 9,
                duration,
                thumbnail,
            } => {
                assert!((0.2..0.5).contains(&duration.unwrap()));
                let thumbnail = thumbnail.unwrap();
                assert_eq!(thumbnail, directory.path().join("9.png"));
                assert!(
                    std::fs::read(thumbnail)
                        .unwrap()
                        .starts_with(b"\x89PNG\r\n\x1a\n")
                );
            }
            _ => panic!("expected an inspection event for file 9"),
        }
    }

    #[test]
    fn unrecognized_media_has_no_duration_or_thumbnail() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("broken.webm");
        std::fs::write(&path, "not media").unwrap();
        let (duration, thumbnail) = inspect(&path, &directory.path().join("preview.png"));
        assert!(duration.is_none());
        assert!(thumbnail.is_none());
    }

    #[test]
    fn exports_never_replace_existing_work() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("lecture.md");
        std::fs::write(&path, "My edited notes").unwrap();
        let transcript = Transcript {
            source: "lecture.webm".into(),
            engine: crate::api::EngineKind::Whisper,
            text: "Local transcription.".into(),
            extraction_duration: Default::default(),
            transcription_duration: Default::default(),
        };
        assert!(export(&transcript, path.clone(), false).is_err());
        assert_eq!(
            export(&transcript, path.clone(), true).unwrap(),
            directory.path().join("lecture (1).md")
        );
        assert_eq!(
            export(&transcript, path.clone(), true).unwrap(),
            directory.path().join("lecture (2).md")
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), "My edited notes");
    }
}
