use clap::{Parser, ValueEnum};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use transcribe_rs::{
    SpeechModel, TranscribeError, TranscribeOptions,
    audio::read_wav_samples,
    onnx::{Quantization, cohere::CohereModel},
    whisper_cpp::WhisperEngine,
};

use crate::EngineKind::{Cohere, Whisper};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum EngineKind {
    Whisper,
    Cohere,
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_enum)]
    engine: EngineKind,
    model_path: PathBuf,
    audio_path: PathBuf,
}

fn load_speech_model(
    engine: EngineKind,
    model_path: &Path,
) -> Result<Box<dyn SpeechModel>, TranscribeError> {
    match engine {
        Whisper => {
            let model = WhisperEngine::load(model_path)?;
            Ok(Box::new(model))
        }
        Cohere => {
            let model = CohereModel::load(model_path, &Quantization::Int8)?;
            Ok(Box::new(model))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let samples = read_wav_samples(&cli.audio_path)?;
    if samples.is_empty() {
        return Err(Box::new(TranscribeError::Audio(
            "Failed to read the audio sample".to_owned(),
        )));
    }

    match cli.engine {
        Whisper => {
            transcribe_rs::set_whisper_accelerator(transcribe_rs::WhisperAccelerator::Gpu);
        }
        Cohere => {
            transcribe_rs::set_ort_accelerator(transcribe_rs::OrtAccelerator::CpuOnly);
        }
    };
    let start = Instant::now();
    let mut model = load_speech_model(cli.engine, &cli.model_path)?;
    let duration = start.elapsed();

    let options = TranscribeOptions {
        language: Some("en".to_owned()),
        ..Default::default()
    };

    let start_transcription = Instant::now();
    let transcription = model.transcribe(&samples, &options)?;
    let transcription_duration = start_transcription.elapsed();

    println!("{}", transcription.text);
    eprintln!(
        "{:?} | {:.2?} | {:.2?}",
        cli.engine, duration, transcription_duration
    );

    Ok(())
}
