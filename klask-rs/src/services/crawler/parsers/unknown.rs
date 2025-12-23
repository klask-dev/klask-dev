use super::constants::TEXT_CONFIDENCE_THRESHOLD;
use super::content_detector::detect_content_type;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Parser for unknown/unrecognized file extensions
#[derive(Clone, Copy)]
pub struct UnknownParser;

impl UnknownParser {
    pub fn can_parse(&self, _extension: &str) -> bool {
        // This parser always accepts files (it's the fallback)
        // but it should only be used if other parsers don't match
        true
    }

    pub async fn parse(&self, content: &[u8], filename: Option<&str>) -> Result<String> {
        // First, analyze the content to determine if it's text or binary
        let analysis = detect_content_type(content);

        let filename_str = filename.unwrap_or("unknown");
        if analysis.stats.null_bytes == 0 && analysis.text_confidence < TEXT_CONFIDENCE_THRESHOLD {
            info!(
                "Unknown parser: analyzing '{}' ({} bytes), content_type={:?}, text_confidence={}%",
                filename_str,
                content.len(),
                analysis.content_type,
                analysis.text_confidence
            );
        }

        // If detected as binary, reject immediately
        if analysis.is_likely_binary() {
            if analysis.stats.null_bytes > 0 {
                info!(
                    "Unknown parser: rejected '{}' - contains {} null bytes (binary indicator)",
                    filename_str, analysis.stats.null_bytes
                );
            } else {
                warn!(
                    "Unknown parser: rejected '{}' - appears to be binary ({} control chars, {} non-ASCII bytes, only {}% text chars)",
                    filename_str,
                    analysis.stats.control_chars,
                    analysis.stats.non_ascii_bytes,
                    analysis.text_confidence
                );
            }
            return Err(anyhow::anyhow!(
                "Unable to parse unknown file type '{}': appears to be binary data ({} text confidence)",
                filename_str,
                analysis.text_confidence
            ));
        }

        // Try to decode as UTF-8 first (most common case)
        match String::from_utf8(content.to_vec()) {
            Ok(text) => {
                debug!(
                    "Unknown parser: successfully parsed '{}' as UTF-8 text ({} bytes, {} text confidence)",
                    filename_str,
                    text.len(),
                    analysis.text_confidence
                );
                Ok(text)
            }
            Err(e) => {
                // UTF-8 failed, but content looks like text
                // Try to decode as lossy UTF-8 (replace invalid sequences)
                warn!(
                    "Unknown parser: UTF-8 decode failed for '{}', attempting lossy decode: {}",
                    filename_str, e
                );
                let text = String::from_utf8_lossy(content).into_owned();
                if text.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Unable to parse unknown file type '{}': cannot decode as UTF-8 text (failed: {})",
                        filename_str,
                        e
                    ));
                }

                info!(
                    "Unknown parser: successfully parsed '{}' as lossy UTF-8 ({} bytes, replaced invalid sequences)",
                    filename_str,
                    text.len()
                );
                Ok(text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_anything() {
        let parser = UnknownParser;
        assert!(parser.can_parse(""));
        assert!(parser.can_parse("random"));
        assert!(parser.can_parse("xyz"));
        assert!(parser.can_parse("unknown"));
    }

    #[tokio::test]
    async fn test_parse_valid_utf8() {
        let parser = UnknownParser;
        let content = b"Some unknown file content";
        let result = parser.parse(content, Some("test.txt")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Some unknown file content");
    }

    #[tokio::test]
    async fn test_parse_with_null_bytes() {
        let parser = UnknownParser;
        let content = b"text\0binary";
        let result = parser.parse(content, Some("binary.dat")).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("binary data"));
    }

    #[tokio::test]
    async fn test_parse_pure_binary() {
        let parser = UnknownParser;
        let content = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
        let result = parser.parse(content, Some("image.bin")).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("binary data"));
    }

    #[tokio::test]
    async fn test_parse_utf8_with_non_ascii() {
        let parser = UnknownParser;
        // UTF-8 encoded: "Café résumé"
        let content = "Café résumé".as_bytes();
        let result = parser.parse(content, Some("french.txt")).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Caf"));
    }

    #[tokio::test]
    async fn test_parse_lossy_utf8() {
        let parser = UnknownParser;
        // ISO-8859-1 or invalid UTF-8 sequences mixed with valid ASCII
        let mut content = b"Hello world ".to_vec();
        content.push(0xFF); // Invalid UTF-8
        content.extend_from_slice(b" more text");

        let result = parser.parse(&content, Some("mixed.bin")).await;
        // Should succeed with lossy decoding (replacement characters)
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("more text"));
    }

    #[tokio::test]
    async fn test_parse_multiline_text() {
        let parser = UnknownParser;
        let content = b"Line 1\nLine 2\nLine 3";
        let result = parser.parse(content, Some("script.unknown")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Line 1\nLine 2\nLine 3");
    }
}
