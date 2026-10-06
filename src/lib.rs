//! Reusable local transcription API and optional desktop interface.

pub mod api;

#[cfg(test)]
mod test_support;

#[cfg(feature = "desktop")]
pub mod gpui;
