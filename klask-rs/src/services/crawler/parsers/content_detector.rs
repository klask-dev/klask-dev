use super::constants::{ANALYSIS_SIZE, TEXT_CONFIDENCE_THRESHOLD};
/// Content type detection - determines if a file is text or binary
/// by analyzing the first 128 bytes for text-like characters
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// File is likely text (ASCII, UTF-8, or other text encodings)
    Text,
    /// File is likely binary data
    Binary,
    /// Unable to determine (too few bytes analyzed)
    #[allow(dead_code)]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub content_type: ContentType,
    /// Percentage of characters that appear to be text (0-100)
    pub text_confidence: u8,
    /// Number of bytes analyzed
    #[allow(dead_code)]
    pub bytes_analyzed: usize,
    /// Detailed statistics
    pub stats: CharacterStats,
}

#[derive(Debug, Clone, Default)]
pub struct CharacterStats {
    /// Common ASCII/text characters (printable ASCII 32-126, plus common whitespace)
    pub text_chars: usize,
    /// Control characters (0-31, excluding common whitespace)
    pub control_chars: usize,
    /// Non-ASCII bytes (128-255)
    pub non_ascii_bytes: usize,
    /// Null bytes
    pub null_bytes: usize,
}

impl ContentAnalysis {
    #[allow(dead_code)]
    pub fn confidence_percentage(&self) -> u8 {
        self.text_confidence
    }

    #[allow(dead_code)]
    pub fn is_likely_text(&self) -> bool {
        self.content_type == ContentType::Text
    }

    pub fn is_likely_binary(&self) -> bool {
        self.content_type == ContentType::Binary
    }
}

/// Detect if content is text or binary by analyzing the first N bytes
///
/// This function:
/// 1. Extracts the first ANALYSIS_SIZE bytes for analysis
/// 2. Counts text vs control characters
/// 3. Counts non-ASCII bytes (UTF-8 continuations are not penalized heavily)
/// 4. Calculates a confidence percentage
/// 5. Returns ContentType based on heuristics
///
/// Heuristics:
/// - Presence of null bytes strongly indicates binary
/// - Excessive control characters indicate binary
/// - Text: >= 70% of characters are text-like (ASCII + valid UTF-8 sequences)
/// - Binary: < 70% text-like characters
pub fn detect_content_type(content: &[u8]) -> ContentAnalysis {
    let bytes_to_analyze = std::cmp::min(content.len(), ANALYSIS_SIZE);
    let sample = &content[..bytes_to_analyze];

    let mut stats = CharacterStats::default();
    let mut i = 0;

    while i < sample.len() {
        let byte = sample[i];

        match byte {
            // Null byte - strong indicator of binary
            0 => {
                stats.null_bytes += 1;
                i += 1;
            }
            // Common whitespace and control chars
            9 | 10 | 13 => {
                stats.text_chars += 1; // \t, \n, \r
                i += 1;
            }
            // Printable ASCII (space to ~)
            32..=126 => {
                stats.text_chars += 1;
                i += 1;
            }
            // Other control characters (bad sign for text)
            1..=8 | 11..=12 | 14..=31 => {
                stats.control_chars += 1;
                i += 1;
            }
            // UTF-8 multi-byte sequences (starting byte)
            192..=247 => {
                // Valid UTF-8 start byte, count the entire sequence as text
                // UTF-8 structure: 110xxxxx (2 bytes), 1110xxxx (3 bytes), 11110xxx (4 bytes)
                stats.text_chars += 1;

                // Skip continuation bytes (10xxxxxx)
                let continuation_count = if byte < 224 {
                    2 // 2-byte sequence
                } else if byte < 240 {
                    3 // 3-byte sequence
                } else {
                    4 // 4-byte sequence
                };

                // Consume continuation bytes
                for _ in 1..continuation_count {
                    if i + 1 < sample.len() && (sample[i + 1] & 0xC0) == 0x80 {
                        stats.non_ascii_bytes += 1;
                        i += 1;
                    } else {
                        // Invalid UTF-8 sequence
                        break;
                    }
                }
                i += 1;
            }
            // Orphaned continuation bytes or invalid sequences (128-191)
            128..=191 => {
                // These are either invalid UTF-8 starts or orphaned continuations
                stats.non_ascii_bytes += 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Calculate text confidence percentage
    // Penalize control chars more than regular characters
    let text_indicators = stats.text_chars as f64;
    let penalty = stats.control_chars as f64 * 2.0; // Control chars count double against us
    let text_score = (text_indicators - penalty).max(0.0);
    let total_analyzed = bytes_to_analyze as f64;

    let text_confidence = if total_analyzed > 0.0 {
        ((text_score / total_analyzed) * 100.0) as u8
    } else {
        0
    };

    // Determine content type
    let content_type = if stats.null_bytes > 0 {
        // Null bytes strongly indicate binary
        ContentType::Binary
    } else if text_confidence >= TEXT_CONFIDENCE_THRESHOLD {
        ContentType::Text
    } else {
        ContentType::Binary
    };

    debug!(
        "Content analysis: type={:?}, confidence={}%, text_chars={}, control_chars={}, non_ascii={}, null_bytes={}",
        content_type, text_confidence, stats.text_chars, stats.control_chars, stats.non_ascii_bytes, stats.null_bytes
    );

    ContentAnalysis { content_type, text_confidence, bytes_analyzed: bytes_to_analyze, stats }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8_text() {
        let content = b"Hello, World! This is some UTF-8 text.";
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Text);
        assert!(analysis.text_confidence >= TEXT_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn test_detect_multiline_text() {
        let content = b"Line 1\nLine 2\nLine 3\nSome text here";
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Text);
        assert!(analysis.text_confidence >= TEXT_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn test_detect_with_tabs_and_newlines() {
        let content = b"fn main() {\n\tprintln!(\"Hello\");\n}";
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Text);
        assert!(analysis.text_confidence >= TEXT_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn test_detect_null_bytes_binary() {
        let content = b"text\0binary\0data";
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Binary);
        assert!(analysis.stats.null_bytes > 0);
    }

    #[test]
    fn test_detect_pure_binary() {
        let content = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Binary);
    }

    #[test]
    fn test_detect_utf8_with_non_ascii() {
        // UTF-8 encoded text: "Café" (with accent)
        let content = "Café résumé naïve".as_bytes();
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Text);
        assert!(analysis.text_confidence >= TEXT_CONFIDENCE_THRESHOLD);
        assert!(analysis.stats.non_ascii_bytes > 0);
    }

    #[test]
    fn test_detect_iso_8859_1_text() {
        // ISO-8859-1 encoded text with extended characters
        let content = &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xE9]; // "Hello é"
        let analysis = detect_content_type(content);
        assert_eq!(analysis.content_type, ContentType::Text);
        assert!(analysis.text_confidence >= TEXT_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn test_detect_mostly_binary_with_some_text() {
        let mut content = vec![0xFF, 0xFE, 0xFD];
        content.extend_from_slice(b"text");
        let analysis = detect_content_type(&content);
        assert_eq!(analysis.content_type, ContentType::Binary);
    }

    #[test]
    fn test_detect_empty_content() {
        let content = &[];
        let analysis = detect_content_type(content);
        assert_eq!(analysis.bytes_analyzed, 0);
    }

    #[test]
    fn test_small_sample_size() {
        let content = b"hi";
        let analysis = detect_content_type(content);
        assert_eq!(analysis.bytes_analyzed, 2);
        assert_eq!(analysis.content_type, ContentType::Text);
    }

    #[test]
    fn test_large_file_only_first_256_bytes_analyzed() {
        let mut content = Vec::new();
        content.extend_from_slice(b"This is text at the beginning");
        // Add 100+ bytes to exceed ANALYSIS_SIZE
        for _ in 0..20 {
            content.extend_from_slice(b"extra content ");
        }
        // Add binary data later (should not affect analysis)
        content.extend_from_slice(&[0xFF, 0xFE, 0xFD]);

        let analysis = detect_content_type(&content);
        assert_eq!(analysis.bytes_analyzed, 256);
        // Since first 256 bytes are mostly text, should detect as text
        assert_eq!(analysis.content_type, ContentType::Text);
    }

    #[test]
    fn test_control_characters_not_confused_with_text() {
        // Mostly control characters should be detected as binary
        let mut content = vec![];
        for i in 0..30 {
            content.push(if i < 20 { i } else { b'a' }); // Mix control chars with some text
        }
        let analysis = detect_content_type(&content);
        // Should be borderline or binary due to high control char ratio
        assert!(analysis.text_confidence < 80);
    }
}
