use crate::api::EngineKind;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Model {
    #[default]
    Whisper,
    Cohere,
}

impl Model {
    pub fn name(self) -> &'static str {
        match self {
            Self::Whisper => "Whisper",
            Self::Cohere => "Cohere",
        }
    }
    pub fn engine(self) -> EngineKind {
        match self {
            Self::Whisper => EngineKind::Whisper,
            Self::Cohere => EngineKind::Cohere,
        }
    }
    pub fn valid_path(self, path: &Path) -> bool {
        match self {
            Self::Whisper => path.is_file(),
            Self::Cohere => [
                "cohere-encoder.int8.onnx",
                "cohere-decoder.int8.onnx",
                "tokens.txt",
            ]
            .iter()
            .all(|name| path.join(name).is_file()),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Settings {
    pub model: Model,
    pub whisper: Option<PathBuf>,
    pub cohere: Option<PathBuf>,
    pub output_directory: Option<PathBuf>,
}

impl Settings {
    pub fn load() -> (Self, Option<String>) {
        let directory = Self::directory();
        let mut roots = vec![directory.join("models")];
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd.join("models"));
        }
        if let Ok(executable) = std::env::current_exe() {
            roots.extend(
                executable
                    .ancestors()
                    .take(6)
                    .map(|path| path.join("models")),
            );
        }
        Self::load_from(&directory, roots)
    }

    // Explicit paths keep persistence testable without changing process-wide
    // environment variables or touching the user's actual preferences.
    fn load_from(
        directory: &Path,
        roots: impl IntoIterator<Item = PathBuf>,
    ) -> (Self, Option<String>) {
        let (mut settings, error) = match std::fs::read(directory.join("settings.json")) {
            Ok(bytes) => match serde_json::from_slice::<Self>(&bytes) {
                Ok(value) => (value, None),
                Err(error) => (
                    Self::default(),
                    Some(format!("Could not read saved settings: {error}")),
                ),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => (Self::default(), None),
            Err(error) => (
                Self::default(),
                Some(format!("Could not read saved settings: {error}")),
            ),
        };
        for root in roots {
            let whisper = root.join("ggml-large-v3-turbo.bin");
            let cohere = root.join("cohere-int8");
            if settings.whisper.is_none() && Model::Whisper.valid_path(&whisper) {
                settings.whisper = Some(whisper);
            }
            if settings.cohere.is_none() && Model::Cohere.valid_path(&cohere) {
                settings.cohere = Some(cohere);
            }
        }
        (settings, error)
    }

    pub fn directory() -> PathBuf {
        std::env::var_os("RUSTSCRIBE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("Library/Application Support/Rustscribe")
            })
    }

    pub fn path(&self, model: Model) -> Option<&Path> {
        match model {
            Model::Whisper => self.whisper.as_deref(),
            Model::Cohere => self.cohere.as_deref(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to(&Self::directory())
    }

    fn save_to(&self, directory: &Path) -> io::Result<()> {
        std::fs::create_dir_all(directory)?;
        let mut file = tempfile::NamedTempFile::new_in(directory)?;
        serde_json::to_writer_pretty(&mut file, self)?;
        file.write_all(b"\n")?;
        file.flush()?;
        file.persist(directory.join("settings.json"))
            .map_err(|error| error.error)?;
        Ok(())
    }
}
