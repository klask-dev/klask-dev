use anyhow::Result;

/// Plain text parser for code files and text documents
#[derive(Clone, Copy)]
pub struct PlainTextParser;

impl PlainTextParser {
    /// List of text file extensions supported by this parser
    const TEXT_EXTENSIONS: &'static [&'static str] = &[
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
        // Data formats
        "sql",
        "json",
        "toml",
        "xml",
        // Markup
        "md",
        // Shell scripts
        "sh",
        "bash",
        "zsh",
        "fish",
        "ps1",
        "bat",
        "cmd",
        // Configuration
        "dockerfile",
        "yaml",
        "yml",
        "cfg",
        "conf",
        "ini",
        "properties",
        // Build systems
        "gradle",
        "maven",
        "pom",
        "sbt",
        "cmake",
        "makefile",
        // Other
        "r",
        "m",
        "perl",
        "pl",
        "lua",
        "txt",
        // Infrastructure as Code
        "tf",
        "hcl",
        // localization
        "po",
    ];

    pub fn can_parse(&self, extension: &str) -> bool {
        Self::TEXT_EXTENSIONS.contains(&extension.to_lowercase().as_str())
    }

    pub async fn parse(&self, content: &[u8], _filename: Option<&str>) -> Result<String> {
        // Convert bytes to string
        // If the file contains null bytes (binary data), return error
        if content.iter().any(|&b| b == 0) {
            return Err(anyhow::anyhow!("Binary content detected in file (contains null bytes)"));
        }

        match String::from_utf8(content.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(anyhow::anyhow!("Unable to decode file as UTF-8 text")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_text_extensions() {
        let parser = PlainTextParser;
        assert!(parser.can_parse("rs"));
        assert!(parser.can_parse("py"));
        assert!(parser.can_parse("js"));
        assert!(parser.can_parse("md"));
        assert!(parser.can_parse("txt"));
    }

    #[test]
    fn test_cannot_parse_binary_extensions() {
        let parser = PlainTextParser;
        assert!(!parser.can_parse("pdf"));
        assert!(!parser.can_parse("docx"));
        assert!(!parser.can_parse("png"));
        assert!(!parser.can_parse("jpg"));
    }

    #[tokio::test]
    async fn test_parse_valid_utf8() {
        let parser = PlainTextParser;
        let content = b"fn main() { println!(\"Hello\"); }";
        let result = parser.parse(content, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fn main() { println!(\"Hello\"); }");
    }

    #[tokio::test]
    async fn test_parse_with_null_bytes() {
        let parser = PlainTextParser;
        let content = b"fn main() {\0 null bytes";
        let result = parser.parse(content, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_invalid_utf8() {
        let parser = PlainTextParser;
        let content = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let result = parser.parse(content, None).await;
        assert!(result.is_err());
    }
}
