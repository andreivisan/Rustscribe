use super::*;
use crate::api::EngineKind;

fn transcript(source: &str, text: &str) -> Transcript {
    Transcript {
        source: source.into(),
        engine: EngineKind::Whisper,
        text: text.into(),
        extraction_duration: Default::default(),
        transcription_duration: Default::default(),
    }
}

fn body(markdown: &str) -> &str {
    markdown.split_once("## Transcript\n\n").unwrap().1
}

#[test]
fn uses_readable_title_and_filename_without_exposing_parent_directories() {
    let markdown = render_markdown(&transcript(
        "/private/lectures/Rust_for-beginners.webm",
        "Hello.",
    ));
    assert!(markdown.starts_with("# Rust for beginners\n\n"));
    assert!(markdown.contains("- **Source:** Rust\\_for\\-beginners\\.webm\n"));
    assert!(markdown.contains("- **Engine:** Whisper\n"));
    assert!(!markdown.contains("/private/lectures"));
    assert_eq!(body(&markdown), "Hello\\.\n");
}

#[test]
fn missing_or_blank_title_has_a_useful_fallback() {
    for source in ["", "__--.webm"] {
        assert!(render_markdown(&transcript(source, "")).starts_with("# Transcript\n"));
    }
    assert!(render_markdown(&transcript("", "")).contains("**Source:** Unknown source"));
}

#[test]
fn escapes_markdown_and_html_in_user_content_and_preserves_unicode() {
    let markdown = render_markdown(&transcript(
        "lecție_[notes].webm",
        "# Heading *bold* [link](url) <script> `code` \\ café 世界",
    ));
    assert!(markdown.starts_with("# lecție \\[notes\\]\n"));
    assert_eq!(
        body(&markdown),
        "\\# Heading \\*bold\\* \\[link\\]\\(url\\) \\<script\\> \\`code\\` \\\\ café 世界\n"
    );
}

#[test]
fn normalizes_whitespace_but_preserves_explicit_paragraphs() {
    let markdown = render_markdown(&transcript(
        "talk.wav",
        "  First\tline\r\ncontinues.\r\n\r\n \t\r\nSecond   paragraph.  ",
    ));
    assert_eq!(
        body(&markdown),
        "First line continues\\.\n\nSecond paragraph\\.\n"
    );
}

#[test]
fn paragraph_break_waits_for_a_sentence_end_after_the_word_target() {
    let first = "word ".repeat(120);
    let markdown = render_markdown(&transcript(
        "talk.wav",
        &format!("{first}still talking. Next sentence."),
    ));
    assert_eq!(
        body(&markdown),
        format!("{first}still talking\\.\n\nNext sentence\\.\n")
    );
}

#[test]
fn closing_quotes_do_not_prevent_a_paragraph_break() {
    let first = "word ".repeat(119);
    let markdown = render_markdown(&transcript("talk.wav", &format!("{first}finished!” Next.")));
    assert_eq!(body(&markdown), format!("{first}finished\\!”\n\nNext\\.\n"));
}

#[test]
fn empty_text_still_produces_a_valid_document_for_either_engine() {
    let mut input = transcript("talk.wav", " \n\t");
    for engine in [EngineKind::Whisper, EngineKind::Cohere] {
        input.engine = engine;
        let markdown = render_markdown(&input);
        assert!(markdown.contains(&format!("**Engine:** {engine}\n")));
        assert_eq!(body(&markdown), "");
        assert!(markdown.ends_with('\n'));
    }
}
