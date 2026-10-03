use clap::{Parser, ValueEnum};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use transcribe_rs::{
    SpeechModel, TranscribeError, TranscribeOptions, TranscriptionResult,
    onnx::{Quantization, cohere::CohereModel},
    transcriber::{EnergyAdaptiveChunked, EnergyAdaptiveConfig, Transcriber},
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
    input_path: PathBuf,
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

fn extract_audio(input_path: &Path, output_path: &Path) -> std::io::Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-n")
        .arg("-i")
        .arg(input_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(output_path)
        .output()?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "FFmpeg failed ({}): {}",
            output.status,
            error_msg.trim(),
        )));
    }
    Ok(())
}

fn transcribe_wav_chunked(
    model: &mut dyn SpeechModel,
    wav_path: &Path,
    options: TranscribeOptions,
) -> Result<TranscriptionResult, TranscribeError> {
    let config = EnergyAdaptiveConfig {
        target_chunk_secs: 25.0,
        search_window_secs: 2.0,
        padding_secs: 0.0,
        min_chunk_secs: 0.0,
        ..Default::default()
    };
    let mut chunker = EnergyAdaptiveChunked::new(config, options);
    let mut reader = hound::WavReader::open(wav_path)?;
    let reader_spec = reader.spec();
    if reader_spec.sample_rate != 16_000
        || reader_spec.channels != 1
        || reader_spec.bits_per_sample != 16
        || reader_spec.sample_format != hound::SampleFormat::Int
    {
        return Err(TranscribeError::Audio(format!(
            "Expected 16 kHz mono 16-bit integer PCM WAV, found {reader_spec:?}",
        )));
    }
    if reader.duration() == 0_u32 {
        return Err(TranscribeError::Audio(
            "Audio contains no samples".to_owned(),
        ));
    }
    let mut buffer: Vec<f32> = Vec::with_capacity(16000);
    for sample in reader.samples::<i16>() {
        let sample_f32 = f32::from(sample?) / 32768.0;
        buffer.push(sample_f32);
        if buffer.len() == 16000 {
            chunker.feed(model, &buffer)?;
            buffer.clear();
        }
    }
    chunker.feed(model, &buffer)?;
    chunker.finish(model)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let tempdir = tempfile::tempdir()?;
    let output_file_path = tempdir.path().join("audio.wav");
    let start_audio_extraction = Instant::now();
    extract_audio(&cli.input_path, &output_file_path)?;
    let audio_extraction_duration = start_audio_extraction.elapsed();

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
    let transcription = transcribe_wav_chunked(model.as_mut(), &output_file_path, options)?;
    let transcription_duration = start_transcription.elapsed();

    println!("{}", transcription.text);
    eprintln!(
        "{:?} | {:.2?} | {:.2?} | {:.2?}",
        cli.engine, audio_extraction_duration, duration, transcription_duration
    );

    Ok(())
}
