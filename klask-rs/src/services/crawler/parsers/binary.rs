use anyhow::Result;

/// Binary file parser that rejects all binary files
#[derive(Clone, Copy)]
pub struct BinaryParser;

impl BinaryParser {
    /// List of binary file extensions handled by this parser
    const BINARY_EXTENSIONS: &'static [&'static str] = &[
        // Images
        "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "svg", "ico", "rlib", "rmeta", "pak", "pack",
        "jar", // Archives
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "lz4", // Documents
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // Audio/Video
        "mp3", "mp4", "wav", "flac", "aac", "m4a", "ogg", "webm", "mkv", "avi", "mov", // Executables
        "exe", "dll", "so", "dylib", "a", "o", "class", "pyc", // Compiled
        "bin", "dat", "blob", "wasm", "rev", "mo", // Other binary formats
        "icns", "cur", "db", "sqlite", "iso", "dmg", "eot", "otf", "ttf", "woff2", "woff2", // fonts
    ];

    pub fn can_parse(&self, extension: &str) -> bool {
        Self::BINARY_EXTENSIONS.contains(&extension.to_lowercase().as_str())
    }

    pub async fn parse(&self, _content: &[u8], _filename: Option<&str>) -> Result<String> {
        Err(anyhow::anyhow!("Binary files cannot be parsed and indexed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_binary_extensions() {
        let parser = BinaryParser;
        assert!(parser.can_parse("png"));
        assert!(parser.can_parse("jpg"));
        assert!(parser.can_parse("exe"));
        assert!(parser.can_parse("zip"));
        assert!(parser.can_parse("mp4"));
    }

    #[test]
    fn test_cannot_parse_text_extensions() {
        let parser = BinaryParser;
        assert!(!parser.can_parse("txt"));
        assert!(!parser.can_parse("rs"));
        assert!(!parser.can_parse("py"));
        assert!(!parser.can_parse("js"));
    }

    #[tokio::test]
    async fn test_parse_returns_error() {
        let parser = BinaryParser;
        let content = b"\x89PNG\r\n\x1a\n"; // PNG magic bytes
        let result = parser.parse(content, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Binary files cannot be parsed"));
    }
}
