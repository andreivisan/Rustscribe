# Markdown export exercise

The module refactor is complete. Implement export in `src/api/markdown.rs`, then call it from `src/cli.rs`. The renderer and writer must also be usable by the future desktop UI.

## Public functions

Implement these functions:

```rust
pub fn render_markdown(transcript: &Transcript) -> String

pub fn write_markdown(
    transcript: &Transcript,
    output_path: &Path,
) -> std::io::Result<()>
```

Import `Transcript` from `super` and `Path` from `std::path`. Re-export both functions from `src/api/mod.rs`:

```rust
pub use markdown::{render_markdown, write_markdown};
```

The renderer builds text without touching the filesystem. The writer saves it and returns errors without printing. Keep engine loading and transcription outside both functions.

## 1. Render a readable document

Target format:

```markdown
# quantum boy story

**Source:** quantum-boy-story.webm
**Engine:** Cohere

## Transcript

I know this guy who had a pretty crazy life. ...

The next paragraph continues here. ...
```

Requirements:

- Derive the title from `transcript.source.file_stem()`, replacing underscores and hyphens with spaces. Fall back to `Transcript` if the result is empty or missing.
- Show the source filename, not the entire filesystem path.
- Show a human-readable engine name. Implement `std::fmt::Display` for `EngineKind` in `types.rs`, returning `Whisper` or `Cohere`.
- Preserve the spoken words and punctuation. Do not summarize, rewrite, or remove repeated phrases.
- Preserve existing paragraph boundaries. For a long paragraph, insert a blank line at the first sentence-ending word after approximately 120 words. A simple initial heuristic is a word ending in `.`, `!`, or `?`, optionally followed by closing quotation marks or brackets. This is formatting, not linguistic sentence detection.
- Do not split an unpunctuated passage just to hit the word target. Keep its remaining text together.
- Escape literal Markdown characters in generated titles, source names, and transcript content so characters such as `*`, backticks, `<`, and `#` display as text. One simple approach is to backslash-escape ASCII punctuation in user-supplied text; keep the Markdown syntax you generate separately unescaped.
- Finish the document with a newline. An empty transcript should produce a valid document without inventing spoken content.

Pseudocode:

```text
derive title and source filename
append escaped title and metadata
append the Transcript heading

for each existing paragraph:
    reset word count
    for each word:
        append the word as escaped literal text
        increment word count
        if the word closes a sentence and count >= 120:
            append a paragraph break
            reset word count
    terminate the paragraph

return the finished document
```

Whitespace normalization is fine; preserve the order and spelling of words and punctuation. Count and inspect the original words before Markdown escaping.

## 2. Save without overwriting an existing file

Use `std` and the already-installed `tempfile` crate. No additional dependency is needed.

Pseudocode:

```text
render the document
resolve the output parent directory (use "." for a bare filename)
create any missing parent directories
create a NamedTempFile in that same directory
write all document bytes
flush the temporary file
publish it using persist_noclobber(output_path)
return success
```

Hints:

- `Path::parent()` can return an empty path for a bare filename. Normalize that to `Path::new(".")`.
- `std::io::Write` supplies `write_all` and `flush`.
- `NamedTempFile::persist_noclobber` refuses to replace an existing destination. Convert its `PersistError` to the underlying `std::io::Error` through the `error` field.
- Preparing the contents in a temporary file avoids exposing an incompletely written Markdown document as the final output. Temporary files clean themselves up on ordinary error returns.
- Do not implement an overwrite flag in this exercise.

## 3. Connect the CLI

Add this field to `Cli`:

```rust
#[arg(short, long)]
output: Option<PathBuf>,
```

After transcription:

```text
if --output was provided:
    use that path
otherwise:
    use input_path.with_extension("md")

write_markdown(&transcript, &output_path)?
print the transcript to stdout as before
print the saved path and existing timing information to stderr
```

Use `unwrap_or_else` to construct the default path only when needed. Borrow paths and `Transcript` when calling the writer; there is no need to clone the transcript text.

These commands should work once the exercise is implemented:

```bash
cargo run --release -- cohere models/cohere-int8 \
  "/Users/andreivisan/YouTube/intresting_videos/quantum-boy-story.webm" \
  --output transcripts/quantum-boy-story.md

cargo run --release -- whisper models/ggml-large-v3-turbo.bin \
  "/Users/andreivisan/YouTube/intresting_videos/quantum-boy-story.webm" \
  --output transcripts/quantum-boy-story-whisper.md
```

## Done when

- Both commands produce readable Markdown, including paragraphs for long text.
- Omitting `--output` places the Markdown beside the input video.
- A second export to the same path returns an error and preserves the original file.
- Filenames containing spaces and Markdown punctuation display correctly.
- Empty text and text without sentence-ending punctuation are handled without panics or lost words.
- Rendering and writing can be checked with a manually constructed `Transcript`; no model is required for these checks.
- `cargo fmt --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, and any export tests pass.

This finishes the current CLI/core milestone. The `desktop` feature and `src/gpui/mod.rs` are placeholders for the subsequent GUI work.
