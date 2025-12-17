use super::binary_parser::BinaryParser;
use super::text_parser::TextParser;
use super::{ParseError, ParsedContent, Parser};
use mimetype_detector::{detect, kind::MimeKind};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::{debug, warn};

/// Global parser dispatcher
pub static PARSER_DISPATCHER: Lazy<ParserDispatcher> = Lazy::new(ParserDispatcher::new);

/// Format MimeKind flags for readable logging
fn format_mime_kind(kind: MimeKind) -> String {
    let mut kinds = Vec::new();
    if kind.contains(MimeKind::TEXT) {
        kinds.push("TEXT");
    }
    if kind.contains(MimeKind::APPLICATION) {
        kinds.push("APPLICATION");
    }
    if kind.contains(MimeKind::IMAGE) {
        kinds.push("IMAGE");
    }
    if kind.contains(MimeKind::AUDIO) {
        kinds.push("AUDIO");
    }
    if kind.contains(MimeKind::VIDEO) {
        kinds.push("VIDEO");
    }
    if kind.contains(MimeKind::DOCUMENT) {
        kinds.push("DOCUMENT");
    }
    if kind.contains(MimeKind::ARCHIVE) {
        kinds.push("ARCHIVE");
    }
    if kind.contains(MimeKind::FONT) {
        kinds.push("FONT");
    }
    if kind.contains(MimeKind::EXECUTABLE) {
        kinds.push("EXECUTABLE");
    }
    if kind.contains(MimeKind::UNKNOWN) {
        kinds.push("UNKNOWN");
    }

    if kinds.is_empty() { "UNKNOWN".to_string() } else { kinds.join("|") }
}

/// Dispatcher for managing file parsers
pub struct ParserDispatcher {
    parsers: Vec<Arc<dyn Parser>>,
}

impl ParserDispatcher {
    /// Create a new dispatcher with default parsers
    pub fn new() -> Self {
        let mut parsers: Vec<Arc<dyn Parser>> = vec![
            // Register built-in parsers
            // Binary parser first - rejects known binary extensions
            Arc::new(BinaryParser::new()),
            // Text parser handles text/code files
            Arc::new(TextParser::new()),
            // Future: Feature-gated parsers
            // #[cfg(feature = "pdf-parser")]
            // Arc::new(PdfParser::new()),
            // #[cfg(feature = "docx-parser")]
            // Arc::new(DocxParser::new()),
        ];

        // Sort by priority (highest first)
        parsers.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        Self { parsers }
    }

    /// Find a parser that can handle the given content
    ///
    /// # Arguments
    /// * `content` - Raw file bytes (used for MIME detection)
    /// * `extension` - File extension (fallback if MIME detection fails)
    ///
    /// # Returns
    /// * `Some(Arc<dyn Parser>)` - Parser that can handle this content
    /// * `None` - No parser found
    pub fn find_parser(&self, content: &[u8], extension: Option<&str>) -> Option<Arc<dyn Parser>> {
        // 1. Try MIME detection first
        let mime_type = detect(content);
        let kind = mime_type.kind();

        // Check if we detected a meaningful kind (not UNKNOWN)
        if kind != MimeKind::UNKNOWN {
            debug!("Detected MIME kind: {:?} for extension: {:?}", kind, extension);

            for parser in &self.parsers {
                if parser.supported_kinds().iter().any(|k| kind.contains(*k)) {
                    debug!("Selected parser '{}' based on MIME kind {:?}", parser.name(), kind);
                    return Some(parser.clone());
                }
            }
        }

        // 2. Fallback to extension-based matching
        if let Some(ext) = extension {
            let ext_lower = ext.to_lowercase();
            for parser in &self.parsers {
                if parser.supported_extensions().iter().any(|e| e.to_lowercase() == ext_lower) {
                    debug!("Selected parser '{}' based on extension '{}'", parser.name(), ext);
                    return Some(parser.clone());
                }
            }
        }

        // 3. Ultimate fallback: for files without extension (like LICENSE, README),
        // try TextParser to see if it can parse as text
        if extension.is_none() {
            // Find TextParser by name
            if let Some(text_parser) = self.parsers.iter().find(|p| p.name() == "text") {
                debug!("No extension found, attempting TextParser as fallback for potential text file");
                return Some(text_parser.clone());
            }
        }

        // No parser found
        debug!(
            "No parser found for extension: {:?}, content length: {}",
            extension,
            content.len()
        );
        None
    }

    /// Parse content using the appropriate parser
    ///
    /// # Arguments
    /// * `content` - Raw file bytes
    /// * `file_path` - Path to the file (for logging/error messages)
    /// * `extension` - Optional file extension
    ///
    /// # Returns
    /// * `Ok(ParsedContent)` - Successfully parsed content
    /// * `Err(ParseError)` - Failed to parse or no parser found
    pub fn parse(&self, content: &[u8], file_path: &str, extension: Option<&str>) -> Result<ParsedContent, ParseError> {
        match self.find_parser(content, extension) {
            Some(parser) => parser.parse(content, file_path),
            None => {
                // Log with file_path for debugging
                let mime_type = detect(content);
                warn!(
                    "No parser found for file '{}' - extension: {:?}, MIME: {}, kind: {}, size: {} bytes",
                    file_path,
                    extension,
                    mime_type.mime(),
                    format_mime_kind(mime_type.kind()),
                    content.len()
                );
                Err(ParseError::UnsupportedType(extension.unwrap_or("unknown").to_string()))
            }
        }
    }

    /// Check if any parser supports this content
    ///
    /// # Arguments
    /// * `content` - Raw file bytes
    /// * `extension` - Optional file extension
    ///
    /// # Returns
    /// * `true` - At least one parser can PARSE this content (extract text)
    /// * `false` - No text parser found (binary files return false)
    #[allow(dead_code)] // Used in tests, may be used in future
    pub fn is_supported(&self, content: &[u8], extension: Option<&str>) -> bool {
        // Only consider parseable (non-binary) parsers
        // BinaryParser (priority 100) rejects; TextParser (priority -10) accepts text
        if let Some(parser) = self.find_parser(content, extension) {
            // Check if it's the binary parser (returns BinaryFile errors)
            parser.name() != "binary"
        } else {
            false
        }
    }

    /// Get all supported MIME kinds across all parsers
    #[allow(dead_code)] // Will be used in the future
    pub fn all_supported_kinds(&self) -> Vec<MimeKind> {
        self.parsers.iter().flat_map(|p| p.supported_kinds().iter().copied()).collect()
    }

    /// Get all text/parseable extensions (excluding binary parser extensions)
    /// Use this for file filtering to accept known text files
    pub fn all_text_extensions(&self) -> Vec<&'static str> {
        self.parsers
            .iter()
            .filter(|p| p.name() != "binary")
            .flat_map(|p| p.supported_extensions().iter().copied())
            .collect()
    }

    /// Get all binary extensions (from binary parser only)
    /// Use this for file filtering to reject known binary files
    pub fn all_binary_extensions(&self) -> Vec<&'static str> {
        self.parsers
            .iter()
            .filter(|p| p.name() == "binary")
            .flat_map(|p| p.supported_extensions().iter().copied())
            .collect()
    }
}

impl Default for ParserDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_finds_text_parser_for_rs() {
        let dispatcher = ParserDispatcher::new();
        let content = b"let x = 42;";
        let parser = dispatcher.find_parser(content, Some("rs"));
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().name(), "text");
    }

    #[test]
    fn test_dispatcher_parse_rust_file() {
        let dispatcher = ParserDispatcher::new();
        let content = b"fn main() { println!(\"Hello\"); }";
        let result = dispatcher.parse(content, "main.rs", Some("rs"));
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.text, "fn main() { println!(\"Hello\"); }");
    }

    #[test]
    fn test_dispatcher_rejects_binary_files() {
        let dispatcher = ParserDispatcher::new();
        // JPG is recognized as binary and rejected by BinaryParser
        let content = b"\xFF\xD8\xFF\xE0"; // JPG header
        let result = dispatcher.parse(content, "image.jpg", Some("jpg"));
        assert!(result.is_err());
        // BinaryParser rejects with BinaryFile error
        assert!(matches!(result, Err(ParseError::BinaryFile(_))));
    }

    #[test]
    fn test_dispatcher_is_supported() {
        let dispatcher = ParserDispatcher::new();
        let content = b"let x = 42;";
        assert!(dispatcher.is_supported(content, Some("rs")));
        assert!(dispatcher.is_supported(content, Some("py")));
        // PDF is not supported by default (no PDF parser registered yet)
        // But mimetype-detector will detect it, so we need a different unsupported type
        assert!(!dispatcher.is_supported(b"\x89PNG\r\n\x1a\n", Some("png"))); // PNG not supported
    }

    #[test]
    fn test_dispatcher_all_text_extensions() {
        let dispatcher = ParserDispatcher::new();
        let extensions = dispatcher.all_text_extensions();
        assert!(extensions.contains(&"rs"));
        assert!(extensions.contains(&"py"));
        assert!(extensions.contains(&"js"));
        // Binary extensions should NOT be in text extensions
        assert!(!extensions.contains(&"png"));
        assert!(!extensions.contains(&"jpg"));
    }

    #[test]
    fn test_dispatcher_all_binary_extensions() {
        let dispatcher = ParserDispatcher::new();
        let extensions = dispatcher.all_binary_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"exe"));
        // Text extensions should NOT be in binary extensions
        assert!(!extensions.contains(&"rs"));
        assert!(!extensions.contains(&"py"));
    }
}
