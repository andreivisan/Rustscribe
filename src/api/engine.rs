use std::{path::Path, sync::Once, time::Instant};

use transcribe_rs::{
    SpeechModel, TranscribeError, TranscribeOptions, TranscriptionResult,
    onnx::{Quantization, cohere::CohereModel},
    transcriber::{EnergyAdaptiveChunked, EnergyAdaptiveConfig, Transcriber},
    whisper_cpp::WhisperEngine,
};

use super::{ApiResult, EngineKind, Transcript, audio::extract_audio};

#[cfg(test)]
mod tests;

static CONFIGURE_ACCELERATORS: Once = Once::new();

/// Owns one loaded model and reuses it across transcription jobs.
///
/// Loading and transcription are blocking operations. Desktop callers should
/// move the engine to a background worker rather than use it on the UI thread.
pub struct TranscriptionEngine {
    kind: EngineKind,
    model: Box<dyn SpeechModel>,
}

impl TranscriptionEngine {
    /// Load Whisper with Metal or Cohere with the CPU backend.
    pub fn load(kind: EngineKind, model_path: &Path) -> ApiResult<Self> {
        // transcribe-rs accelerator preferences are process-wide. Configure
        // them once before loading either engine, including on worker threads.
        CONFIGURE_ACCELERATORS.call_once(|| {
            transcribe_rs::set_whisper_accelerator(transcribe_rs::WhisperAccelerator::Gpu);
            transcribe_rs::set_ort_accelerator(transcribe_rs::OrtAccelerator::CpuOnly);
        });

        let model = load_speech_model(kind, model_path)?;
        Ok(Self { kind, model })
    }

    /// Extract audio and transcribe it in bounded chunks using the loaded model.
    pub fn transcribe(&mut self, input_path: &Path) -> ApiResult<Transcript> {
        let tempdir = tempfile::tempdir()?;
        let output_file_path = tempdir.path().join("audio.wav");
        let start_audio_extraction = Instant::now();
        extract_audio(input_path, &output_file_path)?;
        let extraction_duration = start_audio_extraction.elapsed();

        let options = TranscribeOptions {
            language: Some("en".to_owned()),
            ..Default::default()
        };

        let start_transcription = Instant::now();
        let transcription =
            transcribe_wav_chunked(self.model.as_mut(), &output_file_path, options)?;
        let transcription_duration = start_transcription.elapsed();

        Ok(Transcript {
            source: input_path.to_path_buf(),
            engine: self.kind,
            text: transcription.text,
            extraction_duration,
            transcription_duration,
        })
    }
}

fn load_speech_model(
    engine: EngineKind,
    model_path: &Path,
) -> Result<Box<dyn SpeechModel>, TranscribeError> {
    match engine {
        EngineKind::Whisper => {
            let model = WhisperEngine::load(model_path)?;
            Ok(Box::new(model))
        }
        EngineKind::Cohere => {
            let model = CohereModel::load(model_path, &Quantization::Int8)?;
            Ok(Box::new(model))
        }
    }
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
