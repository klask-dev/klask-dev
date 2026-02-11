pub mod binary;
pub mod constants;
pub mod content_detector;
pub mod dispatcher;
pub mod docx;
pub mod pdf;
pub mod plain_text;
pub mod unknown;

use anyhow::Result;
use binary::BinaryParser;
use docx::DocxParser;
use pdf::PdfParser;
use plain_text::PlainTextParser;
use unknown::UnknownParser;

/// Enum representing all available file parsers
pub enum FileParser {
    PlainText(PlainTextParser),
    Pdf(PdfParser),
    Docx(DocxParser),
    Binary(BinaryParser),
    Unknown(UnknownParser),
}

impl FileParser {
    /// Check if this parser can handle the given file extension
    pub fn can_parse(&self, extension: &str) -> bool {
        match self {
            FileParser::PlainText(p) => p.can_parse(extension),
            FileParser::Pdf(p) => p.can_parse(extension),
            FileParser::Docx(p) => p.can_parse(extension),
            FileParser::Binary(p) => p.can_parse(extension),
            FileParser::Unknown(p) => p.can_parse(extension),
        }
    }

    /// Parse file content and return the extracted text
    /// Optional `filename` parameter is used for logging context
    pub async fn parse(&self, content: &[u8], filename: Option<&str>) -> Result<String> {
        match self {
            FileParser::PlainText(p) => p.parse(content, filename).await,
            FileParser::Pdf(p) => p.parse(content, filename).await,
            FileParser::Docx(p) => p.parse(content, filename).await,
            FileParser::Binary(p) => p.parse(content, filename).await,
            FileParser::Unknown(p) => p.parse(content, filename).await,
        }
    }
}

pub use dispatcher::ParserDispatcher;
