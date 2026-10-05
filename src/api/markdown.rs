use std::{
    io::{Result, Write},
    path::Path,
};

use tempfile::NamedTempFile;

use super::Transcript;

pub fn render_markdown(transcript: &Transcript) -> String {
    let mut output = String::new();
    let mut words_in_paragraph = 0;

    let mut file_title = "Transcript".to_owned();

    if let Some(title) = transcript.source.file_stem() {
        let normalized = title.to_string_lossy().replace(['_', '-'], " ");

        let trimmed = normalized.trim();

        if !trimmed.is_empty() {
            file_title = trimmed.to_owned();
        }
    }

    let mut source_name = "Unknown source".to_owned();
    if let Some(source) = transcript.source.file_name() {
        source_name = source.to_string_lossy().into_owned();
    }

    output.push_str("# ");
    push_escaped(&mut output, &file_title);
    output.push_str("\n\n- **Source:** ");
    push_escaped(&mut output, &source_name);
    output.push_str("\n- **Engine:** ");
    output.push_str(&transcript.engine.to_string());
    output.push_str("\n\n## Transcript\n\n");

    for line in transcript.text.lines() {
        if line.trim().is_empty() {
            if words_in_paragraph > 0 {
                output.push_str("\n\n");
                words_in_paragraph = 0;
            }
            continue;
        }

        for word in line.split_whitespace() {
            if words_in_paragraph > 0 {
                output.push(' ');
            }

            push_escaped(&mut output, word);
            words_in_paragraph += 1;

            if words_in_paragraph >= 120 && ends_sentence(word) {
                output.push_str("\n\n");
                words_in_paragraph = 0;
            }
        }
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

pub fn write_markdown(transcript: &Transcript, output_path: &Path) -> Result<()> {
    let markdown = render_markdown(transcript);
    let directory = match output_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    std::fs::create_dir_all(directory)?;
    let mut temporary = NamedTempFile::new_in(directory)?;
    temporary.write_all(markdown.as_bytes())?;
    temporary.flush()?;
    match temporary.persist_noclobber(output_path) {
        Ok(_) => Ok(()),
        Err(problem) => Err(problem.error),
    }
}

fn push_escaped(output: &mut String, text: &str) {
    for character in text.chars() {
        if character.is_ascii_punctuation() {
            output.push('\\');
        }
        output.push(character);
    }
}

fn ends_sentence(word: &str) -> bool {
    let trimmed = word.trim_end_matches(['"', '\'', '”', '’', ')', ']', '}']);

    trimmed.ends_with(['.', '!', '?'])
}
