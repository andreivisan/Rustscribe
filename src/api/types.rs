use std::{path::PathBuf, time::Duration};

use crate::api::EngineKind::{Cohere, Whisper};

#[derive(Debug, Clone, Copy)]
pub enum EngineKind {
    Whisper,
    Cohere,
}

/// An owned transcription result, independent of the CLI and GUI.
#[derive(Debug)]
pub struct Transcript {
    pub source: PathBuf,
    pub engine: EngineKind,
    pub text: String,
    pub extraction_duration: Duration,
    pub transcription_duration: Duration,
}

impl std::fmt::Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Whisper => f.write_str("Whisper"),
            Cohere => f.write_str("Cohere"),
        }
    }
}
