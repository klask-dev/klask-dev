use anyhow::Result;

/// DOCX document parser (Microsoft Word format)
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct DocxParser;

impl DocxParser {
    pub fn can_parse(&self, extension: &str) -> bool {
        extension.to_lowercase() == "docx"
    }

    pub async fn parse(&self, _content: &[u8], _filename: Option<&str>) -> Result<String> {
        // TODO: Implement DOCX parsing using a library like docx or zipfile
        // This should:
        // 1. Extract the DOCX (which is a ZIP archive)
        // 2. Parse the document.xml file
        // 3. Extract text content from paragraphs and runs
        // 4. Handle formatting and embedded text
        // 5. Return extracted text as String
        Err(anyhow::anyhow!("DOCX parsing is not yet implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_docx() {
        let parser = DocxParser;
        assert!(parser.can_parse("docx"));
        assert!(parser.can_parse("DOCX"));
    }

    #[test]
    fn test_cannot_parse_other_extensions() {
        let parser = DocxParser;
        assert!(!parser.can_parse("txt"));
        assert!(!parser.can_parse("pdf"));
        assert!(!parser.can_parse("doc"));
    }

    #[tokio::test]
    async fn test_parse_not_implemented() {
        let parser = DocxParser;
        let content = b"PK\x03\x04"; // ZIP magic bytes (DOCX is a ZIP)
        let result = parser.parse(content, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not yet implemented"));
    }
}
