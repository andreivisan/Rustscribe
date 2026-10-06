//! Native desktop shell; the reusable API has no GPUI dependency.
mod assets;
mod settings;
mod views;
mod worker;

use crate::api::Transcript;
use gpui::AppContext;
use gpui::{
    App, Application, Bounds, ClipboardItem, Context, FocusHandle, KeyBinding, Menu, MenuItem,
    PathPromptOptions, TitlebarOptions, WindowBounds, WindowOptions, actions, point, px, size,
};
use settings::{Model, Settings};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    time::{Duration, Instant},
};
use worker::{Command, Event};

actions!(
    rustscribe,
    [
        Quit,
        AddFiles,
        OpenSettings,
        StartTranscription,
        ExportAll,
        Dismiss
    ]
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Ready,
    Queued,
    Loading,
    Transcribing,
    Done,
    Failed,
}
impl Status {
    fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Queued => "Queued",
            Self::Loading => "Loading model…",
            Self::Transcribing => "Transcribing…",
            Self::Done => "Transcribed",
            Self::Failed => "Needs attention",
        }
    }
}

struct Media {
    id: usize,
    path: PathBuf,
    bytes: u64,
    duration: Option<f64>,
    thumbnail: Option<PathBuf>,
    status: Status,
    transcript: Option<Arc<Transcript>>,
    saved: Option<PathBuf>,
    exporting: bool,
    error: Option<String>,
}
impl Media {
    fn name(&self) -> String {
        self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
    fn extension(&self) -> String {
        self.path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_uppercase()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Ready,
    Done,
}

struct Desktop {
    focus: FocusHandle,
    settings: Settings,
    settings_open: bool,
    reading: bool,
    files: Vec<Media>,
    selected: Option<usize>,
    filter: Filter,
    next_id: usize,
    commands: Sender<Command>,
    events: Receiver<Event>,
    cache: Arc<tempfile::TempDir>,
    pending: usize,
    polling: bool,
    busy: bool,
    batch_count: usize,
    completed_count: usize,
    started: Option<Instant>,
    cancel: Arc<AtomicBool>,
    notice: Option<(String, bool)>,
}

impl Desktop {
    fn new(cx: &mut Context<Self>) -> Self {
        let (settings, error) = Settings::load();
        let (commands, events) = worker::spawn();
        Self {
            focus: cx.focus_handle(),
            settings,
            settings_open: false,
            reading: false,
            files: vec![],
            selected: None,
            filter: Filter::All,
            next_id: 0,
            commands,
            events,
            cache: Arc::new(tempfile::tempdir().expect("could not create thumbnail cache")),
            pending: 0,
            polling: false,
            busy: false,
            batch_count: 0,
            completed_count: 0,
            started: None,
            cancel: Arc::new(AtomicBool::new(false)),
            notice: error.map(|message| (message, true)),
        }
    }
    fn selected_file(&self) -> Option<&Media> {
        self.files
            .iter()
            .find(|file| Some(file.id) == self.selected)
    }
    fn show_notice(&mut self, message: impl Into<String>, error: bool, cx: &mut Context<Self>) {
        self.notice = Some((message.into(), error));
        cx.notify();
    }
    fn save_settings(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.settings.save() {
            self.show_notice(format!("Could not save preferences: {error}"), true, cx);
        }
        cx.notify();
    }
    fn add_files(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let mut rejected = 0;
        for path in paths {
            let Ok(path) = path.canonicalize() else {
                rejected += 1;
                continue;
            };
            if self.files.iter().any(|file| file.path == path) {
                continue;
            }
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_ascii_lowercase();
            if !matches!(
                extension.as_str(),
                "mp4"
                    | "webm"
                    | "mov"
                    | "mkv"
                    | "m4v"
                    | "avi"
                    | "mpeg"
                    | "mpg"
                    | "ts"
                    | "mts"
                    | "wav"
                    | "mp3"
                    | "m4a"
                    | "aac"
                    | "flac"
                    | "ogg"
                    | "opus"
                    | "aiff"
                    | "aif"
                    | "wma"
            ) || !path.is_file()
            {
                rejected += 1;
                continue;
            }
            let id = self.next_id;
            self.next_id += 1;
            let bytes = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            self.files.push(Media {
                id,
                path: path.clone(),
                bytes,
                duration: None,
                thumbnail: None,
                status: Status::Ready,
                transcript: None,
                saved: None,
                exporting: false,
                error: None,
            });
            self.selected.get_or_insert(id);
            if self
                .commands
                .send(Command::Inspect {
                    id,
                    path,
                    cache: self.cache.clone(),
                })
                .is_ok()
            {
                self.pending += 1;
            }
        }
        self.filter = Filter::All;
        if rejected > 0 {
            self.show_notice(
                format!(
                    "{rejected} item(s) could not be added. Choose local video or audio files."
                ),
                true,
                cx,
            );
        }
        self.poll(cx);
        cx.notify();
    }
    fn pick_files(&mut self, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Add to Rustscribe".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = prompt.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(Ok(Some(paths))) => this.add_files(paths, cx),
                Ok(Err(error)) => this.show_notice(error.to_string(), true, cx),
                _ => {}
            });
        })
        .detach();
    }
    fn choose_model(&mut self, model: Model, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: model == Model::Whisper,
            directories: model == Model::Cohere,
            multiple: false,
            prompt: Some(
                match model {
                    Model::Whisper => "Choose Whisper .bin model",
                    Model::Cohere => "Choose Cohere int8 folder",
                }
                .into(),
            ),
        });
        cx.spawn(async move |this, cx| {
            let result = prompt.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(Ok(Some(paths))) => if let Some(path) = paths.into_iter().next() {
                    if !model.valid_path(&path) {
                        this.show_notice("Model not found. Whisper needs a model file; Cohere needs its int8 encoder, decoder, and tokens.txt in one folder.", true, cx); return;
                    }
                    match model { Model::Whisper => this.settings.whisper = Some(path), Model::Cohere => this.settings.cohere = Some(path) }
                    this.save_settings(cx);
                },
                Ok(Err(error)) => this.show_notice(error.to_string(), true, cx), _ => {},
            });
        }).detach();
    }
    fn choose_output(&mut self, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Choose export folder".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = prompt.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(Ok(Some(paths))) => {
                    this.settings.output_directory = paths.into_iter().next();
                    this.save_settings(cx);
                }
                Ok(Err(error)) => this.show_notice(error.to_string(), true, cx),
                _ => {}
            });
        })
        .detach();
    }
    fn transcribe(&mut self, selected_only: bool, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let model = self.settings.model;
        let Some(path) = self
            .settings
            .path(model)
            .filter(|path| model.valid_path(path))
            .map(Path::to_path_buf)
        else {
            self.settings_open = true;
            self.show_notice(
                format!("Choose your local {} model to get started.", model.name()),
                true,
                cx,
            );
            return;
        };
        let jobs: Vec<_> = self
            .files
            .iter()
            .filter(|file| {
                !file.exporting
                    && if selected_only {
                        Some(file.id) == self.selected
                    } else {
                        file.transcript.is_none()
                    }
            })
            .map(|file| (file.id, file.path.clone()))
            .collect();
        if jobs.is_empty() {
            return;
        }
        for file in &mut self.files {
            if jobs.iter().any(|(id, _)| *id == file.id) {
                file.status = Status::Queued;
                file.error = None;
            }
        }
        self.cancel = Arc::new(AtomicBool::new(false));
        self.batch_count = jobs.len();
        self.completed_count = 0;
        self.started = Some(Instant::now());
        self.notice = None;
        self.busy = self
            .commands
            .send(Command::Transcribe {
                jobs,
                model,
                path,
                cancel: self.cancel.clone(),
            })
            .is_ok();
        if !self.busy {
            self.show_notice(
                "The transcription worker stopped. Please restart Rustscribe.",
                true,
                cx,
            );
        }
        self.poll(cx);
        cx.notify();
    }
    fn stop(&mut self, cx: &mut Context<Self>) {
        self.cancel.store(true, Ordering::Relaxed);
        self.show_notice("Stopping after the current file finishes.", false, cx);
    }
    fn export(&mut self, selected_only: bool, cx: &mut Context<Self>) {
        let jobs: Vec<_> = self
            .files
            .iter()
            .filter(|file| !file.exporting && (!selected_only || Some(file.id) == self.selected))
            .filter_map(|file| {
                file.transcript.as_ref().map(|transcript| {
                    let name = file.path.with_extension("md");
                    let path = self
                        .settings
                        .output_directory
                        .as_ref()
                        .map(|dir| dir.join(name.file_name().unwrap_or_default()))
                        .unwrap_or(name);
                    (file.id, transcript.clone(), path)
                })
            })
            .collect();
        for (id, transcript, path) in jobs {
            self.queue_export(id, transcript, path, true, cx);
        }
    }
    fn save_as(&mut self, cx: &mut Context<Self>) {
        let Some(file) = self.selected_file().filter(|file| !file.exporting) else {
            return;
        };
        let Some(transcript) = file.transcript.clone() else {
            return;
        };
        let id = file.id;
        let name = file.path.with_extension("md");
        let directory = self
            .settings
            .output_directory
            .as_deref()
            .or_else(|| file.path.parent())
            .unwrap_or(Path::new("."));
        let prompt =
            cx.prompt_for_new_path(directory, name.file_name().and_then(|name| name.to_str()));
        cx.spawn(async move |this, cx| {
            let result = prompt.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(Ok(Some(path))) => {
                    this.queue_export(id, transcript, path.with_extension("md"), false, cx)
                }
                Ok(Err(error)) => this.show_notice(error.to_string(), true, cx),
                _ => {}
            });
        })
        .detach();
    }
    fn queue_export(
        &mut self,
        id: usize,
        transcript: Arc<Transcript>,
        path: PathBuf,
        unique: bool,
        cx: &mut Context<Self>,
    ) {
        self.notice = None;
        if self
            .commands
            .send(Command::Export {
                id,
                transcript,
                path,
                unique,
            })
            .is_ok()
        {
            self.pending += 1;
            if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                file.exporting = true;
            }
            self.poll(cx);
        } else {
            self.show_notice(
                "The export worker stopped. Please restart Rustscribe.",
                true,
                cx,
            );
        }
        cx.notify();
    }
    fn copy_transcript(&mut self, cx: &mut Context<Self>) {
        if let Some(transcript) = self
            .selected_file()
            .and_then(|file| file.transcript.as_ref())
        {
            cx.write_to_clipboard(ClipboardItem::new_string(transcript.text.clone()));
            self.show_notice("Transcript copied to clipboard.", false, cx);
        }
    }
    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.selected_file().is_some_and(|file| file.exporting) {
            return;
        }
        self.files.retain(|file| Some(file.id) != self.selected);
        self.selected = self.files.first().map(|file| file.id);
        cx.notify();
    }
    fn poll(&mut self, cx: &mut Context<Self>) {
        if self.polling || (self.pending == 0 && !self.busy) {
            return;
        }
        self.polling = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let keep_polling = this
                    .update(cx, |this, cx| {
                        loop {
                            match this.events.try_recv() {
                                Ok(event) => this.handle_event(event),
                                Err(TryRecvError::Empty) => break,
                                Err(TryRecvError::Disconnected) => {
                                    this.busy = false;
                                    this.pending = 0;
                                    this.notice = Some((
                                        "The background worker stopped. Please restart Rustscribe."
                                            .into(),
                                        true,
                                    ));
                                    break;
                                }
                            }
                        }
                        this.polling = this.pending > 0 || this.busy;
                        cx.notify();
                        this.polling
                    })
                    .unwrap_or(false);
                if !keep_polling {
                    break;
                }
            }
        })
        .detach();
    }
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Inspected {
                id,
                duration,
                thumbnail,
            } => {
                self.pending = self.pending.saturating_sub(1);
                if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                    file.duration = duration;
                    file.thumbnail = thumbnail;
                }
            }
            Event::Started { id, loading } => {
                if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                    file.status = if loading {
                        Status::Loading
                    } else {
                        Status::Transcribing
                    };
                }
            }
            Event::Completed { id, result } => {
                self.completed_count += 1;
                if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                    match result {
                        Ok(transcript) => {
                            file.transcript = Some(transcript);
                            file.status = Status::Done;
                            file.saved = None;
                            file.error = None;
                        }
                        Err(error) => {
                            file.status = Status::Failed;
                            file.error = Some(error.clone());
                            self.notice = Some((error, true));
                        }
                    }
                }
            }
            Event::BatchFinished => {
                self.busy = false;
                for file in &mut self.files {
                    if file.status == Status::Queued {
                        file.status = if file.transcript.is_some() {
                            Status::Done
                        } else {
                            Status::Ready
                        };
                    }
                }
                if !self.notice.as_ref().is_some_and(|(_, error)| *error) {
                    self.notice = Some((
                        if self.cancel.load(Ordering::Relaxed) {
                            "Queue stopped. Your completed transcripts are ready.".into()
                        } else {
                            format!(
                                "{} transcript{} ready to export.",
                                self.completed_count,
                                if self.completed_count == 1 {
                                    " is"
                                } else {
                                    "s are"
                                }
                            )
                        },
                        false,
                    ));
                }
            }
            Event::Exported { id, result } => {
                self.pending = self.pending.saturating_sub(1);
                if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                    file.exporting = false;
                    match result {
                        Ok(path) => {
                            if !self.notice.as_ref().is_some_and(|(_, error)| *error) {
                                self.notice = Some((
                                    format!(
                                        "Saved {}",
                                        path.file_name().unwrap_or_default().to_string_lossy()
                                    ),
                                    false,
                                ));
                            }
                            file.saved = Some(path);
                        }
                        Err(error) => {
                            self.notice = Some((format!("Could not export: {error}"), true));
                        }
                    }
                }
            }
        }
    }
}
impl Drop for Desktop {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Launch the native macOS desktop application.
pub fn run() {
    Application::new()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("cmd-o", AddFiles, None),
                KeyBinding::new("cmd-,", OpenSettings, None),
                KeyBinding::new("cmd-enter", StartTranscription, None),
                KeyBinding::new("cmd-shift-e", ExportAll, None),
                KeyBinding::new("escape", Dismiss, None),
            ]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.set_menus(vec![
                Menu {
                    name: "Rustscribe".into(),
                    items: vec![
                        MenuItem::action("Settings…", OpenSettings),
                        MenuItem::separator(),
                        MenuItem::action("Quit Rustscribe", Quit),
                    ],
                },
                Menu {
                    name: "File".into(),
                    items: vec![
                        MenuItem::action("Add files…", AddFiles),
                        MenuItem::action("Transcribe queue", StartTranscription),
                        MenuItem::action("Export all Markdown", ExportAll),
                    ],
                },
            ]);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1120.), px(790.)), cx);
            let result = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Rustscribe".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(20.), px(20.))),
                    }),
                    window_min_size: Some(size(px(960.), px(690.))),
                    app_id: Some("com.rustscribe.desktop".into()),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_rem_size(px(16.));
                    cx.new(|cx| {
                        let mut desktop = Desktop::new(cx);
                        window.focus(&desktop.focus);
                        desktop.add_files(
                            std::env::args_os().skip(1).map(PathBuf::from).collect(),
                            cx,
                        );
                        desktop
                    })
                },
            );
            if let Err(error) = result {
                eprintln!("Could not open Rustscribe: {error}");
                cx.quit();
            }
            cx.activate(true);
        });
}
