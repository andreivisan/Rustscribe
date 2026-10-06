use super::*;
use hound::{SampleFormat, WavSpec, WavWriter};
use transcribe_rs::ModelCapabilities;

// Record the model boundary instead of downloading weights or depending on GPU output.
#[derive(Default)]
struct RecordingModel {
    chunks: Vec<Vec<f32>>,
    options: Vec<TranscribeOptions>,
    fail_on_call: Option<usize>,
}

impl SpeechModel for RecordingModel {
    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            name: "Test model",
            engine_id: "test",
            sample_rate: 16_000,
            languages: &["en"],
            supports_timestamps: false,
            supports_translation: false,
            supports_streaming: false,
        }
    }

    fn transcribe_raw(
        &mut self,
        samples: &[f32],
        options: &TranscribeOptions,
    ) -> Result<TranscriptionResult, TranscribeError> {
        self.chunks.push(samples.to_vec());
        self.options.push(options.clone());
        let call = self.chunks.len();
        if self.fail_on_call == Some(call) {
            return Err(TranscribeError::Inference("test inference failure".into()));
        }
        Ok(TranscriptionResult {
            text: format!("Chunk {call}."),
            segments: None,
        })
    }
}

fn pcm_spec() -> WavSpec {
    WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    }
}

fn write_wav(path: &Path, spec: WavSpec, samples: &[i16]) {
    let mut writer = WavWriter::create(path, spec).unwrap();
    for &sample in samples {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}

#[test]
fn short_audio_keeps_every_sample_and_normalizes_pcm() {
    let directory = tempfile::tempdir().unwrap();
    let wav = directory.path().join("short.wav");
    write_wav(&wav, pcm_spec(), &[i16::MIN, -16_384, 0, 16_384, i16::MAX]);
    let mut model = RecordingModel::default();
    let options = TranscribeOptions {
        language: Some("en".into()),
        ..Default::default()
    };

    let result = transcribe_wav_chunked(&mut model, &wav, options).unwrap();

    assert_eq!(result.text, "Chunk 1.");
    assert_eq!(
        model.chunks,
        [vec![-1.0, -0.5, 0.0, 0.5, 32767.0 / 32768.0]]
    );
    assert_eq!(model.options[0].language.as_deref(), Some("en"));
    assert!(!model.options[0].translate);
}

#[test]
fn long_audio_is_bounded_ordered_and_keeps_the_final_partial_buffer() {
    let directory = tempfile::tempdir().unwrap();
    let wav = directory.path().join("long.wav");
    // More than two chunks, with a tail that is shorter than a one-second read buffer.
    let samples: Vec<i16> = (0..16_000 * 62 + 137)
        .map(|index| (index % 30_000) as i16)
        .collect();
    write_wav(&wav, pcm_spec(), &samples);
    let mut model = RecordingModel::default();

    let result = transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).unwrap();

    assert!(model.chunks.len() >= 3);
    assert!(
        model
            .chunks
            .iter()
            .all(|chunk| !chunk.is_empty() && chunk.len() <= 27 * 16_000)
    );
    let actual: Vec<f32> = model.chunks.iter().flatten().copied().collect();
    let expected: Vec<f32> = samples
        .iter()
        .map(|&sample| f32::from(sample) / 32768.0)
        .collect();
    assert_eq!(
        actual, expected,
        "chunk boundaries must not drop or duplicate samples"
    );
    let expected_text = (1..=model.chunks.len())
        .map(|index| format!("Chunk {index}."))
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(result.text, expected_text);
}

#[test]
fn incompatible_wav_formats_are_rejected_before_inference() {
    let directory = tempfile::tempdir().unwrap();
    let wav = directory.path().join("wrong-format.wav");
    let specs = [
        WavSpec {
            channels: 2,
            ..pcm_spec()
        },
        WavSpec {
            sample_rate: 44_100,
            ..pcm_spec()
        },
        WavSpec {
            bits_per_sample: 24,
            ..pcm_spec()
        },
        WavSpec {
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
            ..pcm_spec()
        },
    ];

    for spec in specs {
        let mut writer = WavWriter::create(&wav, spec).unwrap();
        for _ in 0..spec.channels {
            if spec.sample_format == SampleFormat::Float {
                writer.write_sample(0.5_f32).unwrap();
            } else {
                writer.write_sample(123_i32).unwrap();
            }
        }
        writer.finalize().unwrap();
        let mut model = RecordingModel::default();
        let error =
            transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).unwrap_err();
        assert!(
            matches!(error, TranscribeError::Audio(_)),
            "{spec:?}: {error}"
        );
        assert!(model.chunks.is_empty());
    }
}

#[test]
fn empty_missing_and_corrupt_audio_do_not_reach_the_model() {
    let directory = tempfile::tempdir().unwrap();
    let wav = directory.path().join("audio.wav");
    let mut model = RecordingModel::default();
    assert!(transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).is_err());

    std::fs::write(&wav, b"not a WAV file").unwrap();
    assert!(transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).is_err());

    write_wav(&wav, pcm_spec(), &[]);
    let error = transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).unwrap_err();
    assert!(matches!(error, TranscribeError::Audio(message) if message.contains("no samples")));
    assert!(model.chunks.is_empty());
}

#[test]
fn inference_failure_aborts_instead_of_returning_a_partial_transcript() {
    let directory = tempfile::tempdir().unwrap();
    let wav = directory.path().join("audio.wav");
    write_wav(&wav, pcm_spec(), &vec![100_i16; 16_000 * 70]);
    let mut model = RecordingModel {
        fail_on_call: Some(2),
        ..Default::default()
    };

    let error = transcribe_wav_chunked(&mut model, &wav, TranscribeOptions::default()).unwrap_err();

    assert!(
        matches!(error, TranscribeError::Inference(message) if message == "test inference failure")
    );
    assert_eq!(model.chunks.len(), 2);
}

#[test]
fn extraction_and_chunking_return_source_and_engine_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("lecture with spaces.wav");
    write_wav(
        &source,
        WavSpec {
            channels: 2,
            sample_rate: 44_100,
            ..pcm_spec()
        },
        &vec![1000; 44_100 * 2],
    );
    let mut engine = TranscriptionEngine {
        kind: EngineKind::Cohere,
        model: Box::<RecordingModel>::default(),
    };

    let transcript = engine.transcribe(&source).unwrap();

    assert_eq!(transcript.source, source);
    assert!(matches!(transcript.engine, EngineKind::Cohere));
    assert_eq!(transcript.text, "Chunk 1.");
}
