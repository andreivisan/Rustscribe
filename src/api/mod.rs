//! Local transcription shared by the CLI, desktop app, and other consumers.

mod audio;
mod engine;
mod markdown;
mod types;

pub use engine::TranscriptionEngine;
pub use markdown::render_markdown;
pub use types::{EngineKind, Transcript};

/// An API error that can cross a background-worker boundary.
pub type ApiError = Box<dyn std::error::Error + Send + Sync>;
pub type ApiResult<T> = Result<T, ApiError>;
