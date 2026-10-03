//! Reusable local transcription API and optional desktop interface.

pub mod api;

#[cfg(feature = "desktop")]
pub mod gpui;
