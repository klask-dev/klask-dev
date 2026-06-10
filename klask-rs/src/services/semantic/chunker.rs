// The binary target compiles this module tree without consuming the chunker
// yet (the indexing pipeline lands in plan phase 2); silence the resulting
// false-positive dead-code warnings until then.
#![allow(dead_code)]

//! Source-code chunking for embedding generation.
//!
//! Splits file content into overlapping line windows sized for the embedding
//! model context. Window boundaries prefer natural code seams (blank lines and
//! new top-level declarations) so a function is less likely to be cut in half.
//! Tree-sitter based structural chunking is planned as a follow-up (see
//! docs/SEMANTIC_SEARCH_PLAN.md §3.3); this line-window chunker is the
//! universal fallback it will degrade to for unsupported languages.

/// A contiguous slice of a file prepared for embedding.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// First line of the chunk, 1-based inclusive.
    pub start_line: usize,
    /// Last line of the chunk, 1-based inclusive.
    pub end_line: usize,
    /// Chunk text, prefixed with a context header line (file path). The header
    /// measurably improves code embedding quality by anchoring the snippet.
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ChunkOptions {
    /// Maximum number of file lines per chunk (excluding the context header).
    pub max_lines: usize,
    /// Number of lines repeated between consecutive chunks.
    pub overlap_lines: usize,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self { max_lines: 60, overlap_lines: 15 }
    }
}

/// Fraction of the window we are willing to give up to end on a natural seam.
const BOUNDARY_SEARCH_FRACTION: f64 = 0.25;

/// Split `content` into overlapping chunks ready for embedding.
///
/// Guarantees:
/// - every line of the file appears in at least one chunk;
/// - chunk windows never exceed `max_lines` lines;
/// - consecutive chunks overlap (so context spanning a boundary is not lost);
/// - always makes forward progress, even with degenerate options.
pub fn chunk_file(file_path: &str, content: &str, options: &ChunkOptions) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    if total_lines == 0 {
        return Vec::new();
    }

    let max_lines = options.max_lines.max(1);
    // Overlap must leave room to advance by at least one line per chunk
    let overlap = options.overlap_lines.min(max_lines - 1);

    let mut chunks = Vec::new();
    let mut start = 0usize; // 0-based inclusive index into `lines`

    loop {
        let hard_end = (start + max_lines).min(total_lines); // 0-based exclusive
        let end = if hard_end < total_lines {
            adjust_to_boundary(&lines, start, hard_end)
        } else {
            hard_end
        };

        chunks.push(Chunk {
            start_line: start + 1,
            end_line: end,
            text: format!("// file: {}\n{}", file_path, lines[start..end].join("\n")),
        });

        if end >= total_lines {
            break;
        }
        // Step back by the overlap, but always move forward
        start = (end - overlap).max(start + 1);
    }

    chunks
}

/// Walk backwards from the hard window end looking for a natural seam: a blank
/// line, or a line starting a new top-level declaration (non-whitespace at
/// column 0 right after a line that doesn't open a deeper block). Gives up
/// after `BOUNDARY_SEARCH_FRACTION` of the window and keeps the hard end.
fn adjust_to_boundary(lines: &[&str], start: usize, hard_end: usize) -> usize {
    let window = hard_end - start;
    let min_end = hard_end - ((window as f64 * BOUNDARY_SEARCH_FRACTION) as usize).min(window - 1);

    let mut end = hard_end;
    while end > min_end {
        let last_in_window = lines[end - 1];
        let first_outside = lines.get(end).copied().unwrap_or("");
        // Seam: window ends on a blank line, or the next line starts a new
        // top-level construct (column-0 non-whitespace character)
        if last_in_window.trim().is_empty()
            || first_outside.starts_with(|c: char| !c.is_whitespace() && c != '}' && c != ')')
        {
            return end;
        }
        end -= 1;
    }
    hard_end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_of(n: usize) -> String {
        (1..=n).map(|i| format!("    line {}", i)).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn test_empty_content_yields_no_chunks() {
        assert!(chunk_file("a.rs", "", &ChunkOptions::default()).is_empty());
    }

    #[test]
    fn test_short_file_is_a_single_chunk() {
        let content = "fn main() {\n    println!(\"hi\");\n}";
        let chunks = chunk_file("src/main.rs", content, &ChunkOptions::default());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 3);
        assert!(chunks[0].text.starts_with("// file: src/main.rs\n"));
        assert!(chunks[0].text.contains("println!"));
    }

    #[test]
    fn test_every_line_is_covered_and_chunks_overlap() {
        let total = 500;
        let content = lines_of(total);
        let options = ChunkOptions { max_lines: 60, overlap_lines: 15 };
        let chunks = chunk_file("big.rs", &content, &options);

        assert!(chunks.len() > 1);
        assert_eq!(chunks.first().unwrap().start_line, 1);
        assert_eq!(chunks.last().unwrap().end_line, total);

        for window in chunks.windows(2) {
            let (a, b) = (&window[0], &window[1]);
            assert!(b.start_line > a.start_line, "chunks must make forward progress");
            assert!(
                b.start_line <= a.end_line + 1,
                "no gap allowed between chunk ending at {} and chunk starting at {}",
                a.end_line,
                b.start_line
            );
        }
        for chunk in &chunks {
            assert!(chunk.end_line - chunk.start_line < options.max_lines);
        }
    }

    #[test]
    fn test_boundary_preference_cuts_at_blank_line() {
        // Indented body with a blank line at line 8; window of 9 forces the
        // boundary search to back up from the hard end (line 9) to the seam
        let content = "fn a() {\n    b\n    c\n    d\n    e\n    f\n    g\n\n    h\n    i";
        let options = ChunkOptions { max_lines: 9, overlap_lines: 2 };
        let chunks = chunk_file("x.rs", content, &options);
        // First chunk should stop at the blank line (line 8) rather than line 9
        assert_eq!(chunks[0].end_line, 8);
    }

    #[test]
    fn test_boundary_preference_cuts_before_new_top_level_declaration() {
        // fn a() spans lines 1-10, fn z() starts at line 11. A 12-line window
        // would cut inside fn z; the seam before the new declaration wins.
        let content =
            "fn a() {\n    b1\n    b2\n    b3\n    b4\n    b5\n    b6\n    b7\n    b8\n}\nfn z() {\n    y1\n    y2\n}";
        let options = ChunkOptions { max_lines: 12, overlap_lines: 2 };
        let chunks = chunk_file("x.rs", content, &options);
        assert_eq!(
            chunks[0].end_line, 10,
            "first chunk should end at fn a()'s closing brace"
        );
    }

    #[test]
    fn test_degenerate_options_still_progress() {
        let content = lines_of(10);
        // overlap >= max_lines would loop forever without the clamp
        let options = ChunkOptions { max_lines: 3, overlap_lines: 99 };
        let chunks = chunk_file("x.rs", &content, &options);
        assert_eq!(chunks.last().unwrap().end_line, 10);
        assert!(chunks.len() <= 10, "must terminate with bounded chunk count");
    }

    #[test]
    fn test_single_long_line() {
        let content = "x".repeat(10_000);
        let chunks = chunk_file("x.txt", &content, &ChunkOptions::default());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 1);
    }

    #[test]
    fn test_default_options_match_plan() {
        let options = ChunkOptions::default();
        assert_eq!(options.max_lines, 60);
        assert_eq!(options.overlap_lines, 15);
    }
}
