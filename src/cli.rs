use std::{path::PathBuf, time::Instant};

use clap::{Parser, ValueEnum};
use rustscribe::api::{ApiResult, EngineKind, TranscriptionEngine, write_markdown};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliEngine {
    Whisper,
    Cohere,
}

impl From<CliEngine> for EngineKind {
    fn from(engine: CliEngine) -> Self {
        match engine {
            CliEngine::Whisper => Self::Whisper,
            CliEngine::Cohere => Self::Cohere,
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_enum)]
    engine: CliEngine,
    model_path: PathBuf,
    input_path: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub(crate) fn run() -> ApiResult<()> {
    let cli = Cli::parse();
    let output_path = cli
        .output
        .unwrap_or_else(|| cli.input_path.with_extension("md"));
    let start = Instant::now();
    let mut engine = TranscriptionEngine::load(cli.engine.into(), &cli.model_path)?;
    let load_duration = start.elapsed();

    let transcript = engine.transcribe(&cli.input_path)?;
    write_markdown(&transcript, &output_path)?;
    eprintln!("Saved {}", output_path.display());

    println!("{}", transcript.text);
    eprintln!(
        "{:?} | {:.2?} | {:.2?} | {:.2?}",
        transcript.engine,
        transcript.extraction_duration,
        load_duration,
        transcript.transcription_duration,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_engines_and_paths_with_spaces() {
        for (name, kind) in [
            ("whisper", EngineKind::Whisper),
            ("cohere", EngineKind::Cohere),
        ] {
            let cli = Cli::try_parse_from([
                "rustscribe",
                name,
                "models/my model",
                "videos/my lesson.webm",
            ])
            .unwrap();
            assert_eq!(EngineKind::from(cli.engine).to_string(), kind.to_string());
            assert_eq!(cli.model_path, PathBuf::from("models/my model"));
            assert_eq!(cli.input_path, PathBuf::from("videos/my lesson.webm"));
            assert!(cli.output.is_none());
        }
    }

    #[test]
    fn accepts_short_and_long_output_flags() {
        for flag in ["-o", "--output"] {
            let cli = Cli::try_parse_from([
                "rustscribe",
                "whisper",
                "model.bin",
                "video.webm",
                flag,
                "notes/my lesson.md",
            ])
            .unwrap();
            assert_eq!(cli.output, Some(PathBuf::from("notes/my lesson.md")));
        }
    }
}
