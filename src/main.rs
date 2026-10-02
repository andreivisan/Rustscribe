use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use transcribe_rs::SpeechModel;

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
//
// fn parse_args() ->
//
// fn load_speech_model() -> Box<dyn SpeechModel> {
//
// }

fn main() {
    let parser = Cli::parse();
    println!("{:?}", parser);
}
