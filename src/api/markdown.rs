use super::Transcript;

pub fn render_markdown(transcript: &Transcript) -> String {
    let mut output = String::from("## Transcript\n\n");
    let mut words_in_paragraph = 0;

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
