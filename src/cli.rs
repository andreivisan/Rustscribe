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
