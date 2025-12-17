use super::{ParseError, ParsedContent, Parser};
use mimetype_detector::kind::MimeKind;

/// Default parser for text-based files (code, config, documentation)
pub struct TextParser {
    supported_extensions: Vec<&'static str>,
}

impl TextParser {
    pub fn new() -> Self {
        Self {
            // Consolidate the extension list from file_processing.rs
            supported_extensions: vec![
                // Programming languages
                "rs",
                "py",
                "js",
                "ts",
                "java",
                "c",
                "cpp",
                "h",
                "hpp",
                "go",
                "rb",
                "php",
                "cs",
                "swift",
                "kt",
                "scala",
                "clj",
                "hs",
                "ml",
                "fs",
                "elm",
                "dart",
                "vue",
                "jsx",
                "tsx",
                // Web
                "html",
                "css",
                "scss",
                "less",
                "sql",
                // Shell
                "sh",
                "bash",
                "zsh",
                "fish",
                "ps1",
                "bat",
                "cmd",
                // Config
                "yaml",
                "yml",
                "json",
                "toml",
                "xml",
                "cfg",
                "conf",
                "ini",
                "properties",
                "env",
                // Build
                "gradle",
                "maven",
                "pom",
                "sbt",
                "cmake",
                "makefile",
                // Other
                "md",
                "txt",
                "r",
                "m",
                "perl",
                "pl",
                "lua",
                "dockerfile",
            ],
        }
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for TextParser {
    fn name(&self) -> &'static str {
        "text"
    }

    fn supported_kinds(&self) -> &[MimeKind] {
        &[MimeKind::TEXT]
    }

    fn supported_extensions(&self) -> &[&'static str] {
        &self.supported_extensions
    }

    fn parse(&self, content: &[u8], file_path: &str) -> Result<ParsedContent, ParseError> {
        // Convert bytes to UTF-8 string
        let text = String::from_utf8(content.to_vec()).map_err(|_| ParseError::InvalidUtf8)?;

        // Additional binary check (null bytes in text)
        if text.contains('\0') {
            return Err(ParseError::BinaryFile(file_path.to_string()));
        }

        Ok(ParsedContent { text, metadata: None, mime_type: "text/plain".to_string() })
    }

    fn priority(&self) -> i32 {
        // Low priority - should be used as fallback
        -10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_parser_parses_utf8() {
        let parser = TextParser::new();
        let content = b"fn main() { println!(\"Hello\"); }";
        let result = parser.parse(content, "main.rs");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.text, "fn main() { println!(\"Hello\"); }");
    }

    #[test]
    fn test_text_parser_rejects_binary() {
        let parser = TextParser::new();
        let content = b"\x00\x01\x02\x03";
        let result = parser.parse(content, "binary.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_text_parser_rejects_invalid_utf8() {
        let parser = TextParser::new();
        let content = b"\xFF\xFE invalid utf8";
        let result = parser.parse(content, "invalid.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_text_parser_name() {
        let parser = TextParser::new();
        assert_eq!(parser.name(), "text");
    }

    #[test]
    fn test_text_parser_priority() {
        let parser = TextParser::new();
        assert_eq!(parser.priority(), -10);
    }
}
