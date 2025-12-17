use super::{ParseError, ParsedContent, Parser};
use mimetype_detector::kind::MimeKind;

/// Parser for binary files - rejects all attempts to parse
/// Used for filtering out known binary file extensions during indexing
pub struct BinaryParser {
    supported_extensions: Vec<&'static str>,
}

impl BinaryParser {
    pub fn new() -> Self {
        Self {
            // List of known binary file extensions to skip during indexing
            supported_extensions: vec![
                // Images
                "jpg",
                "jpeg",
                "png",
                "gif",
                "bmp",
                "svg",
                "ico",
                "tiff",
                "webp",
                "heic",
                "psd",
                // Audio
                "mp3",
                "wav",
                "flac",
                "aac",
                "opus",
                "m4a",
                "ogg",
                "wma",
                // Video
                "mp4",
                "mkv",
                "webm",
                "avi",
                "mov",
                "flv",
                "wmv",
                "m3u8",
                "ts",
                // Archives
                "zip",
                "rar",
                "7z",
                "tar",
                "gz",
                "bz2",
                "xz",
                "iso",
                "dmg",
                "tgz",
                // Executables
                "exe",
                "dll",
                "so",
                "dylib",
                "msi",
                "apk",
                "ipa",
                "app",
                // Compiled objects
                "o",
                "obj",
                "pyc",
                "pyo",
                "class",
                "jar",
                "wasm",
                // Fonts
                "ttf",
                "otf",
                "woff",
                "woff2",
                "eot",
                // Databases
                "db",
                "sqlite",
                "sqlite3",
                "mdb",
                "accdb",
                // translation
                "mo",
                // Version control
                "git",
                "gitattributes",
                // Build artifacts
                "o",
                "a",
                "lib",
                "rlib",
                // Other binary
                "bin",
                "dat",
                "pkl",
                "pickle",
                "h5",
                "hdf5",
                // Documents (TODO later : extract them into a new doc parser)
                // Note: PDF, DOCX and other document formats are included here temporarily
                // because we plan to create dedicated parsers for them later
                "pdf",
                "docx",
                "doc",
                "pptx",
                "ppt",
            ],
        }
    }
}

impl Default for BinaryParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for BinaryParser {
    fn name(&self) -> &'static str {
        "binary"
    }

    fn supported_kinds(&self) -> &[MimeKind] {
        // Binary parser doesn't support any MIME kinds during parsing
        // It's used for filtering during file discovery, not actual parsing
        &[]
    }

    fn supported_extensions(&self) -> &[&'static str] {
        &self.supported_extensions
    }

    fn parse(&self, _content: &[u8], file_path: &str) -> Result<ParsedContent, ParseError> {
        // Always reject - binary files should not be parsed
        Err(ParseError::BinaryFile(file_path.to_string()))
    }

    fn priority(&self) -> i32 {
        // High priority - reject binary files before text parser
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_parser_rejects_images() {
        let parser = BinaryParser::new();
        let image_content = b"\x89PNG\r\n\x1a\n"; // PNG header
        let result = parser.parse(image_content, "test.png");
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::BinaryFile(_))));
    }

    #[test]
    fn test_binary_parser_name() {
        let parser = BinaryParser::new();
        assert_eq!(parser.name(), "binary");
    }

    #[test]
    fn test_binary_parser_supports_extensions() {
        let parser = BinaryParser::new();
        let extensions = parser.supported_extensions();
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"zip"));
        assert!(extensions.contains(&"exe"));
    }

    #[test]
    fn test_binary_parser_priority() {
        let parser = BinaryParser::new();
        // Higher priority than text parser to reject binaries first
        assert!(parser.priority() > 0);
    }
}
