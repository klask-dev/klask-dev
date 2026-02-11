use super::{BinaryParser, DocxParser, FileParser, PdfParser, PlainTextParser, UnknownParser};

/// Dispatcher that routes files to appropriate parsers based on their extension
pub struct ParserDispatcher;

impl ParserDispatcher {
    pub fn new() -> Self {
        Self
    }

    /// Get the appropriate parser for a file extension
    pub fn get_parser(extension: &str) -> FileParser {
        let ext_lower = extension.to_lowercase();

        // Try each parser in order
        let parsers: Vec<FileParser> = vec![
            FileParser::PlainText(PlainTextParser),
            FileParser::Pdf(PdfParser),
            FileParser::Docx(DocxParser),
            FileParser::Binary(BinaryParser),
        ];

        for parser in parsers {
            if parser.can_parse(&ext_lower) {
                return parser;
            }
        }

        // UnknownParser is the fallback
        FileParser::Unknown(UnknownParser)
    }

    /// Check if a file should be filtered (not parsed)
    /// Returns true if the file should be skipped (only binary files are filtered)
    pub fn should_filter(extension: &str) -> bool {
        // Check if it's a binary file parser
        let binary_parser = FileParser::Binary(BinaryParser);
        binary_parser.can_parse(extension)
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
    fn test_get_parser_for_text_files() {
        let parser = ParserDispatcher::get_parser("rs");
        matches!(parser, FileParser::PlainText(_));

        let parser = ParserDispatcher::get_parser("py");
        matches!(parser, FileParser::PlainText(_));

        let parser = ParserDispatcher::get_parser("txt");
        matches!(parser, FileParser::PlainText(_));
    }

    #[test]
    fn test_get_parser_for_pdf() {
        let parser = ParserDispatcher::get_parser("pdf");
        matches!(parser, FileParser::Pdf(_));
    }

    #[test]
    fn test_get_parser_for_docx() {
        let parser = ParserDispatcher::get_parser("docx");
        matches!(parser, FileParser::Docx(_));
    }

    #[test]
    fn test_get_parser_for_binary() {
        let parser = ParserDispatcher::get_parser("png");
        matches!(parser, FileParser::Binary(_));

        let parser = ParserDispatcher::get_parser("exe");
        matches!(parser, FileParser::Binary(_));

        let parser = ParserDispatcher::get_parser("mp4");
        matches!(parser, FileParser::Binary(_));
    }

    #[test]
    fn test_get_parser_for_unknown() {
        let parser = ParserDispatcher::get_parser("xyz");
        matches!(parser, FileParser::Unknown(_));

        let parser = ParserDispatcher::get_parser("random");
        matches!(parser, FileParser::Unknown(_));
    }

    #[test]
    fn test_should_filter_binary_files() {
        assert!(ParserDispatcher::should_filter("png"));
        assert!(ParserDispatcher::should_filter("jpg"));
        assert!(ParserDispatcher::should_filter("exe"));
        assert!(ParserDispatcher::should_filter("zip"));
    }

    #[test]
    fn test_should_not_filter_text_files() {
        assert!(!ParserDispatcher::should_filter("rs"));
        assert!(!ParserDispatcher::should_filter("py"));
        assert!(!ParserDispatcher::should_filter("txt"));
        assert!(!ParserDispatcher::should_filter("md"));
    }

    #[test]
    fn test_should_not_filter_unknown() {
        assert!(!ParserDispatcher::should_filter("xyz"));
        assert!(!ParserDispatcher::should_filter("random"));
    }
}
