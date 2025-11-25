/// Custom code tokenizer for Klask search
///
/// This tokenizer is specifically designed for code search, handling:
/// - CamelCase splitting: `camelCase` → `["camel", "case"]`
/// - Acronym handling: `HTMLParser` → `["html", "parser"]`, `getHTTPResponse` → `["get", "http", "response"]`
/// - Underscore and hyphen preservation: `my_func` → `["my_func"]`
/// - Case-insensitive search: all tokens are lowercased
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

/// Custom tokenizer for code search that handles camelCase and preserves underscores/hyphens
#[derive(Clone)]
pub struct CodeTokenizer;

impl CodeTokenizer {
    /// Creates a new CodeTokenizer instance
    pub fn new() -> Self {
        CodeTokenizer
    }
}

impl Default for CodeTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for CodeTokenizer {
    type TokenStream<'a> = CodeTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        CodeTokenStream::new(text)
    }
}

/// Custom tokenizer that preserves case exactly - treats entire input as a single token
/// This is used for fields where we need exact case-sensitive matching (via RegexQuery)
#[derive(Clone)]
pub struct RawCasePreservingTokenizer;

impl RawCasePreservingTokenizer {
    /// Creates a new RawCasePreservingTokenizer instance
    pub fn new() -> Self {
        RawCasePreservingTokenizer
    }
}

impl Default for RawCasePreservingTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for RawCasePreservingTokenizer {
    type TokenStream<'a> = RawCasePreservingTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        RawCasePreservingTokenStream::new(text)
    }
}

/// Token stream for the raw case-preserving tokenizer
pub struct RawCasePreservingTokenStream {
    token: Option<Token>,
    consumed: bool,
}

impl RawCasePreservingTokenStream {
    fn new(text: &str) -> Self {
        // Create a single token that spans the entire text, preserving case
        let token = if text.is_empty() {
            None
        } else {
            Some(Token {
                offset_from: 0,
                offset_to: text.len(), // Use byte length, not character count
                position: 0,
                text: text.to_string(), // Preserve original case - do NOT lowercase
                position_length: 1,
            })
        };
        RawCasePreservingTokenStream { token, consumed: false }
    }
}

impl TokenStream for RawCasePreservingTokenStream {
    fn advance(&mut self) -> bool {
        if self.token.is_some() && !self.consumed {
            self.consumed = true;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        self.token.as_ref().expect("token should be available")
    }

    fn token_mut(&mut self) -> &mut Token {
        self.token.as_mut().expect("token should be available")
    }
}

/// Token stream for the code tokenizer
pub struct CodeTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl CodeTokenStream {
    fn new(text: &str) -> Self {
        let tokens = tokenize_code(text);
        CodeTokenStream { tokens, index: 0 }
    }
}

impl TokenStream for CodeTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index.saturating_sub(1)]
    }

    fn token_mut(&mut self) -> &mut Token {
        let idx = self.index.saturating_sub(1);
        &mut self.tokens[idx]
    }
}

/// Tokenizes code text by splitting on whitespace first, then on camelCase boundaries while preserving underscores and hyphens
///
/// Examples:
/// - `"camelCase"` → `["camel", "case"]`
/// - `"HTMLParser"` → `["html", "parser"]`
/// - `"getHTTPResponse"` → `["get", "http", "response"]`
/// - `"my_function_name"` → `["my_function_name"]`
/// - `"snake-case"` → `["snake-case"]`
/// - `"NETBOX_URL"` → `["nginx_url"]`
/// - `"class HTMLParser"` → `["class", "html", "parser"]` (splits on whitespace AND camelCase)
fn tokenize_code(text: &str) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();

    // First, split by whitespace and non-alphanumeric characters (except underscores and hyphens)
    // This ensures we handle punctuation and multi-word input correctly
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || (!c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|s| !s.is_empty())
        .collect();

    let mut token_position = 0;
    let mut byte_offset = 0;

    for word in words.iter() {
        // Find this word in the original text after byte_offset
        let word_byte_pos = if byte_offset < text.len() {
            text[byte_offset..].find(word).unwrap_or(0)
        } else {
            0
        };

        // The actual byte offset of this word in the original text
        let word_byte_offset = byte_offset + word_byte_pos;

        // For each word, apply camelCase tokenization
        let word_tokens = tokenize_code_word(word, word_byte_offset, token_position);
        token_position += word_tokens.len();

        // Move byte_offset to after this word
        byte_offset = word_byte_offset + word.len();

        tokens.extend(word_tokens);
    }

    tokens
}

/// Tokenizes a single word (no whitespace) by splitting on camelCase boundaries while preserving underscores and hyphens
///
/// Emits the following tokens for case-comprehensive search:
/// 1. Split tokens (lowercased): ["reader", "template"]
/// 2. Split tokens (case-preserved): ["reader", "Template"]
/// 3. Complete token (lowercased): ["readertemplate"]
/// 4. Complete token (case-preserved): ["readerTemplate"]
///
/// This allows searching for:
/// - Individual components: "reader" finds "reader" in "readerTemplate"
/// - Complete identifier (case-insensitive): "readertemplate" finds "readerTemplate"
/// - Complete identifier (case-sensitive): "readerTemplate" finds "readerTemplate"
///
/// Example: "readerTemplate" → ["reader", "template", "reader", "Template", "readertemplate", "readerTemplate"]
fn tokenize_code_word(text: &str, byte_offset_base: usize, position_base: usize) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut current_token = String::new();
    let mut current_token_original = String::new();

    // Track byte positions, not character positions
    let mut byte_pos = byte_offset_base;
    let mut token_byte_start = byte_offset_base;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let ch_len = ch.len_utf8(); // Length of this character in bytes

        // Determine if we should split at this position
        let should_split = if i == 0 {
            false // Never split at the start
        } else {
            let prev_ch = chars[i - 1];
            let is_underscore_or_hyphen = ch == '_' || ch == '-';
            let prev_is_underscore_or_hyphen = prev_ch == '_' || prev_ch == '-';

            if is_underscore_or_hyphen || prev_is_underscore_or_hyphen {
                // Don't split on underscores or hyphens; include them in the token
                false
            } else if prev_ch.is_lowercase() && ch.is_uppercase() {
                // Split when transitioning from lowercase to uppercase
                // e.g., "camelCase" → split before 'C'
                true
            } else if prev_ch.is_uppercase() && ch.is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase()
            {
                // Split before the last uppercase in a sequence when followed by lowercase
                // e.g., "HTTPResponse" → split before 'R' (HTTP|Response)
                true
            } else {
                false
            }
        };

        if should_split {
            // Emit the current token if it's not empty
            if !current_token.is_empty() {
                // Emit lowercased version
                let token = Token {
                    offset_from: token_byte_start,
                    offset_to: byte_pos,
                    position: position_base + tokens.len(),
                    text: current_token.to_lowercase(),
                    position_length: 1,
                };
                tokens.push(token);

                // Emit case-preserved version
                let token_original = Token {
                    offset_from: token_byte_start,
                    offset_to: byte_pos,
                    position: position_base + tokens.len(),
                    text: current_token_original.clone(),
                    position_length: 1,
                };
                tokens.push(token_original);

                current_token.clear();
                current_token_original.clear();
                token_byte_start = byte_pos;
            }
        }

        current_token.push(ch);
        current_token_original.push(ch);
        byte_pos += ch_len;
        i += 1;
    }

    // Emit the final split token
    if !current_token.is_empty() {
        // Emit lowercased version
        let token = Token {
            offset_from: token_byte_start,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: current_token.to_lowercase(),
            position_length: 1,
        };
        tokens.push(token);

        // Emit case-preserved version
        let token_original = Token {
            offset_from: token_byte_start,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: current_token_original,
            position_length: 1,
        };
        tokens.push(token_original);
    }

    // If we generated multiple tokens (i.e., the word was split), add complete tokens
    if tokens.len() > 2 {
        // Add the complete lowercased identifier
        let complete_token = Token {
            offset_from: byte_offset_base,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: text.to_lowercase(),
            position_length: 1,
        };
        tokens.push(complete_token);

        // Add the complete case-preserved identifier
        let complete_token_original = Token {
            offset_from: byte_offset_base,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: text.to_string(),
            position_length: 1,
        };
        tokens.push(complete_token_original);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Cases ====================

    #[test]
    fn test_empty_string() {
        let tokens = tokenize_code("");
        assert_eq!(tokens.len(), 0, "Empty string should produce no tokens");
    }

    #[test]
    fn test_single_char_lowercase() {
        let tokens = tokenize_code("a");
        assert_eq!(tokens.len(), 2); // lowercased, case-preserved
        assert_eq!(tokens[0].text, "a");
        assert_eq!(tokens[1].text, "a");
    }

    #[test]
    fn test_single_char_uppercase() {
        let tokens = tokenize_code("A");
        assert_eq!(tokens.len(), 2); // lowercased, case-preserved
        assert_eq!(tokens[0].text, "a", "Lowercased version");
        assert_eq!(tokens[1].text, "A", "Case-preserved version");
    }

    #[test]
    fn test_single_word_lowercase() {
        let tokens = tokenize_code("hello");
        assert_eq!(tokens.len(), 2); // lowercased, case-preserved
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "hello");
    }

    #[test]
    fn test_single_word_uppercase() {
        let tokens = tokenize_code("HELLO");
        assert_eq!(tokens.len(), 2); // lowercased, case-preserved
        assert_eq!(tokens[0].text, "hello", "Lowercased version");
        assert_eq!(tokens[1].text, "HELLO", "Case-preserved version");
    }

    // ==================== CamelCase Cases ====================

    #[test]
    fn test_camel_case_basic() {
        let tokens = tokenize_code("camelCase");
        // 2 (camel) + 2 (case) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "camel");
        assert_eq!(tokens[1].text, "camel");
        assert_eq!(tokens[2].text, "case");
        assert_eq!(tokens[3].text, "Case");
        assert_eq!(tokens[4].text, "camelcase");
        assert_eq!(tokens[5].text, "camelCase");
    }

    #[test]
    fn test_camel_case_basic_simple() {
        let tokens = tokenize_code("camelCase");
        // Count: 2 (camel split) + 2 (case split) + 2 (complete)
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "camel"); // lowercased camel
        assert_eq!(tokens[1].text, "camel"); // case-preserved camel
        assert_eq!(tokens[2].text, "case"); // lowercased case
        assert_eq!(tokens[3].text, "Case"); // case-preserved case
        assert_eq!(tokens[4].text, "camelcase"); // complete lowercased
        assert_eq!(tokens[5].text, "camelCase"); // complete case-preserved
    }

    #[test]
    fn test_camel_case_three_parts() {
        let tokens = tokenize_code("myVariableName");
        // 2 (my) + 2 (variable) + 2 (name) + 2 (complete) = 8
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "my");
        assert_eq!(tokens[2].text, "variable");
        assert_eq!(tokens[3].text, "Variable");
        assert_eq!(tokens[4].text, "name");
        assert_eq!(tokens[5].text, "Name");
        assert_eq!(tokens[6].text, "myvariablename");
        assert_eq!(tokens[7].text, "myVariableName");
    }

    #[test]
    fn test_camel_case_four_parts() {
        let tokens = tokenize_code("parseJSONFromAPIResponse");
        // 2 (parse) + 2 (json) + 2 (from) + 2 (api) + 2 (response) + 2 (complete) = 12
        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0].text, "parse");
        assert_eq!(tokens[1].text, "parse");
        assert_eq!(tokens[2].text, "json");
        assert_eq!(tokens[3].text, "JSON");
        // Check that complete tokens exist
        let has_lowercase_complete = tokens.iter().any(|t| t.text == "parsejsonfromapiresponse");
        let has_case_preserved = tokens.iter().any(|t| t.text == "parseJSONFromAPIResponse");
        assert!(has_lowercase_complete, "Should have lowercase complete token");
        assert!(has_case_preserved, "Should have case-preserved complete token");
    }

    #[test]
    fn test_pascal_case() {
        let tokens = tokenize_code("PascalCase");
        // 2 (Pascal) + 2 (Case) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "pascal");
        assert_eq!(tokens[1].text, "Pascal");
        assert_eq!(tokens[2].text, "case");
        assert_eq!(tokens[3].text, "Case");
        assert_eq!(tokens[4].text, "pascalcase");
        assert_eq!(tokens[5].text, "PascalCase");
    }

    #[test]
    fn test_pascal_case_three_parts() {
        let tokens = tokenize_code("MyVariableName");
        // 2 (My) + 2 (Variable) + 2 (Name) + 2 (complete) = 8
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "My");
        assert_eq!(tokens[2].text, "variable");
        assert_eq!(tokens[3].text, "Variable");
        assert_eq!(tokens[4].text, "name");
        assert_eq!(tokens[5].text, "Name");
        assert_eq!(tokens[6].text, "myvariablename");
        assert_eq!(tokens[7].text, "MyVariableName");
    }

    // ==================== Acronym Cases ====================

    #[test]
    fn test_html_parser() {
        let tokens = tokenize_code("HTMLParser");
        // 2 (HTML) + 2 (Parser) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "html");
        assert_eq!(tokens[1].text, "HTML");
        assert_eq!(tokens[2].text, "parser");
        assert_eq!(tokens[3].text, "Parser");
        assert_eq!(tokens[4].text, "htmlparser");
        assert_eq!(tokens[5].text, "HTMLParser");
    }

    #[test]
    fn test_http_response() {
        let tokens = tokenize_code("HTTPResponse");
        // 2 (HTTP) + 2 (Response) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "http");
        assert_eq!(tokens[1].text, "HTTP");
        assert_eq!(tokens[2].text, "response");
        assert_eq!(tokens[3].text, "Response");
        assert_eq!(tokens[4].text, "httpresponse");
        assert_eq!(tokens[5].text, "HTTPResponse");
    }

    #[test]
    fn test_get_http_response() {
        let tokens = tokenize_code("getHTTPResponse");
        // 2 (get) + 2 (HTTP) + 2 (Response) + 2 (complete) = 8
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "get");
        assert_eq!(tokens[2].text, "http");
        assert_eq!(tokens[3].text, "HTTP");
        assert_eq!(tokens[4].text, "response");
        assert_eq!(tokens[5].text, "Response");
        assert_eq!(tokens[6].text, "gethttpresponse");
        assert_eq!(tokens[7].text, "getHTTPResponse");
    }

    #[test]
    fn test_xml_parser() {
        let tokens = tokenize_code("XMLParser");
        // 2 (XML) + 2 (Parser) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "xml");
        assert_eq!(tokens[1].text, "XML");
        assert_eq!(tokens[2].text, "parser");
        assert_eq!(tokens[3].text, "Parser");
        assert_eq!(tokens[4].text, "xmlparser");
        assert_eq!(tokens[5].text, "XMLParser");
    }

    #[test]
    fn test_io_error() {
        let tokens = tokenize_code("IOError");
        // 2 (IO) + 2 (Error) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "io");
        assert_eq!(tokens[1].text, "IO");
        assert_eq!(tokens[2].text, "error");
        assert_eq!(tokens[3].text, "Error");
        assert_eq!(tokens[4].text, "ioerror");
        assert_eq!(tokens[5].text, "IOError");
    }

    #[test]
    fn test_https_connection() {
        let tokens = tokenize_code("HTTPSConnection");
        // 2 (HTTPS) + 2 (Connection) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "https");
        assert_eq!(tokens[1].text, "HTTPS");
        assert_eq!(tokens[2].text, "connection");
        assert_eq!(tokens[3].text, "Connection");
        assert_eq!(tokens[4].text, "httpsconnection");
        assert_eq!(tokens[5].text, "HTTPSConnection");
    }

    #[test]
    fn test_url_handler() {
        let tokens = tokenize_code("URLHandler");
        // 2 (URL) + 2 (Handler) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "url");
        assert_eq!(tokens[1].text, "URL");
        assert_eq!(tokens[2].text, "handler");
        assert_eq!(tokens[3].text, "Handler");
        assert_eq!(tokens[4].text, "urlhandler");
        assert_eq!(tokens[5].text, "URLHandler");
    }

    #[test]
    fn test_api_client() {
        let tokens = tokenize_code("APIClient");
        // 2 (API) + 2 (Client) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "api");
        assert_eq!(tokens[1].text, "API");
        assert_eq!(tokens[2].text, "client");
        assert_eq!(tokens[3].text, "Client");
        assert_eq!(tokens[4].text, "apiclient");
        assert_eq!(tokens[5].text, "APIClient");
    }

    #[test]
    fn test_json_object() {
        let tokens = tokenize_code("JSONObject");
        // 2 (JSON) + 2 (Object) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "json");
        assert_eq!(tokens[1].text, "JSON");
        assert_eq!(tokens[2].text, "object");
        assert_eq!(tokens[3].text, "Object");
        assert_eq!(tokens[4].text, "jsonobject");
        assert_eq!(tokens[5].text, "JSONObject");
    }

    // ==================== Underscore & Hyphen Cases ====================

    #[test]
    fn test_snake_case() {
        let tokens = tokenize_code("snake_case");
        // 2 (lowercased, case-preserved) - no split because of underscore
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "snake_case", "Lowercased version");
        assert_eq!(tokens[1].text, "snake_case", "Case-preserved version");
    }

    #[test]
    fn test_snake_case_three_parts() {
        let tokens = tokenize_code("my_variable_name");
        // 2 (lowercased, case-preserved) - no split because of underscores
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "my_variable_name", "Lowercased version");
        assert_eq!(tokens[1].text, "my_variable_name", "Case-preserved version");
    }

    #[test]
    fn test_nginx_url() {
        let tokens = tokenize_code("NETBOX_URL");
        // 2 (lowercased, case-preserved) - no split because of underscores
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0].text, "netbox_url",
            "Uppercase with underscores should be lowercased"
        );
        assert_eq!(tokens[1].text, "NETBOX_URL", "Case-preserved version");
    }

    #[test]
    fn test_all_caps_with_underscore() {
        let tokens = tokenize_code("AWS_REGION_NAME");
        // 2 (lowercased, case-preserved) - no split because of underscores
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "aws_region_name", "Lowercase version");
        assert_eq!(tokens[1].text, "AWS_REGION_NAME", "Case-preserved version");
    }

    #[test]
    fn test_hyphenated_basic() {
        let tokens = tokenize_code("my-module");
        // 2 (lowercased, case-preserved) - no split because of hyphen
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "my-module", "Lowercased version");
        assert_eq!(tokens[1].text, "my-module", "Case-preserved version");
    }

    #[test]
    fn test_hyphenated_three_parts() {
        let tokens = tokenize_code("my-function-name");
        // 2 (lowercased, case-preserved) - no split because of hyphens
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "my-function-name", "Lowercased version");
        assert_eq!(tokens[1].text, "my-function-name", "Case-preserved version");
    }

    #[test]
    fn test_config_service() {
        // Hyphens prevent splitting, so the whole thing stays as one token
        let tokens = tokenize_code("config-Service");
        // 2 (lowercased, case-preserved) - hyphen prevents splitting
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0].text, "config-service",
            "Hyphen prevents splitting - lowercase"
        );
        assert_eq!(
            tokens[1].text, "config-Service",
            "Hyphen prevents splitting - case-preserved"
        );
    }

    // ==================== Mixed Cases ====================

    #[test]
    fn test_mixed_snake_and_camel() {
        let tokens = tokenize_code("my_functionName");
        // 2 (my_function) + 2 (name) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "my_function", "Lowercase with underscore");
        assert_eq!(tokens[1].text, "my_function", "Case-preserved with underscore");
        assert_eq!(tokens[2].text, "name", "Split on case transition");
        assert_eq!(tokens[3].text, "Name", "Case-preserved Name");
        assert_eq!(tokens[4].text, "my_functionname", "Complete lowercase");
        assert_eq!(tokens[5].text, "my_functionName", "Complete case-preserved");
    }

    #[test]
    fn test_mixed_hyphen_and_camel() {
        let tokens = tokenize_code("my-functionName");
        // 2 (my-function) + 2 (name) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "my-function", "Lowercase with hyphen");
        assert_eq!(tokens[1].text, "my-function", "Case-preserved with hyphen");
        assert_eq!(tokens[2].text, "name", "Split on case transition");
        assert_eq!(tokens[3].text, "Name", "Case-preserved Name");
        assert_eq!(tokens[4].text, "my-functionname", "Complete lowercase");
        assert_eq!(tokens[5].text, "my-functionName", "Complete case-preserved");
    }

    #[test]
    fn test_snake_case_with_camel() {
        let tokens = tokenize_code("snake_camelCase");
        // 2 (snake_camel) + 2 (case) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "snake_camel", "Lowercase with underscore");
        assert_eq!(tokens[1].text, "snake_camel", "Case-preserved with underscore");
        assert_eq!(tokens[2].text, "case", "Split on case transition");
        assert_eq!(tokens[3].text, "Case", "Case-preserved Case");
        assert_eq!(tokens[4].text, "snake_camelcase", "Complete lowercase");
        assert_eq!(tokens[5].text, "snake_camelCase", "Complete case-preserved");
    }

    #[test]
    fn test_complex_mixed_naming() {
        // Underscores prevent splitting, so everything stays as one token
        let tokens = tokenize_code("my_api_Handler");
        // 2 (lowercased, case-preserved) - underscores prevent splitting
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "my_api_handler", "Lowercase version");
        assert_eq!(tokens[1].text, "my_api_Handler", "Case-preserved version");
    }

    // ==================== Numbers Cases ====================

    #[test]
    fn test_number_at_start() {
        let tokens = tokenize_code("1stPlace");
        // 2 (1st) + 2 (place) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "1st");
        assert_eq!(tokens[1].text, "1st");
        assert_eq!(tokens[2].text, "place");
        assert_eq!(tokens[3].text, "Place");
        assert_eq!(tokens[4].text, "1stplace");
        assert_eq!(tokens[5].text, "1stPlace");
    }

    #[test]
    fn test_number_in_middle() {
        // Numbers don't trigger splits since they're neither lowercase nor uppercase
        // "base64Encode" -> "base64encode" doesn't split at 4->E because 4 is not lowercase
        let tokens = tokenize_code("base64Encode");
        // No split happens, so: 2 (base64encode, case-preserved)
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "base64encode");
        assert_eq!(tokens[1].text, "base64Encode");
    }

    #[test]
    fn test_multiple_numbers() {
        // Numbers don't cause splits; only lowercase->uppercase does
        // "var1Name2Value" splits at "1->N" and "2->V" (both lowercase->uppercase transitions)
        // Actually wait: "1" is not lowercase, so "1N" doesn't split
        // "2" is not lowercase, so "2V" doesn't split
        // The only split is "r->N" but that's followed by "ame2V"
        // So: var1Name2Value as single token (no split because 1 and 2 are not lowercase)
        let tokens = tokenize_code("var1Name2Value");
        assert_eq!(tokens.len(), 2); // Just lowercased and case-preserved
        assert_eq!(tokens[0].text, "var1name2value");
        assert_eq!(tokens[1].text, "var1Name2Value");
    }

    #[test]
    fn test_consecutive_numbers() {
        // Split happens on lowercase to uppercase: "Base64URL" vs "Safe"
        let tokens = tokenize_code("Base64URLSafe");
        // 2 (base64url) + 2 (safe) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "base64url");
        assert_eq!(tokens[1].text, "Base64URL");
        assert_eq!(tokens[2].text, "safe");
        assert_eq!(tokens[3].text, "Safe");
        assert_eq!(tokens[4].text, "base64urlsafe");
        assert_eq!(tokens[5].text, "Base64URLSafe");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_double_underscore() {
        let tokens = tokenize_code("test__double");
        // 2 (lowercased, case-preserved) - underscores prevent splitting
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "test__double", "Lowercased");
        assert_eq!(tokens[1].text, "test__double", "Case-preserved");
    }

    #[test]
    fn test_double_hyphen() {
        let tokens = tokenize_code("test--double");
        // 2 (lowercased, case-preserved) - hyphens prevent splitting
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "test--double", "Lowercased");
        assert_eq!(tokens[1].text, "test--double", "Case-preserved");
    }

    #[test]
    fn test_leading_underscore() {
        let tokens = tokenize_code("_privateFunction");
        // 2 (_private) + 2 (function) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "_private");
        assert_eq!(tokens[1].text, "_private");
        assert_eq!(tokens[2].text, "function");
        assert_eq!(tokens[3].text, "Function");
        assert_eq!(tokens[4].text, "_privatefunction");
        assert_eq!(tokens[5].text, "_privateFunction");
    }

    #[test]
    fn test_trailing_underscore() {
        let tokens = tokenize_code("functionName_");
        // 2 (function) + 2 (name_) + 2 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "function");
        assert_eq!(tokens[1].text, "function");
        assert_eq!(tokens[2].text, "name_");
        assert_eq!(tokens[3].text, "Name_");
        assert_eq!(tokens[4].text, "functionname_");
        assert_eq!(tokens[5].text, "functionName_");
    }

    #[test]
    fn test_only_underscores() {
        let tokens = tokenize_code("___");
        // 2 (lowercased, case-preserved) - no split
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "___");
        assert_eq!(tokens[1].text, "___");
    }

    #[test]
    fn test_only_hyphens() {
        let tokens = tokenize_code("---");
        // 2 (lowercased, case-preserved) - no split
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "---");
        assert_eq!(tokens[1].text, "---");
    }

    #[test]
    fn test_whitespace_splits_words() {
        let tokens = tokenize_code("hello world");
        // 2 (hello) + 2 (world) = 4
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "hello");
        assert_eq!(tokens[2].text, "world");
        assert_eq!(tokens[3].text, "world");
    }

    #[test]
    fn test_uppercase_preserved_case_insensitive() {
        let tokens = tokenize_code("UPPERCASE");
        // 2 (lowercased, case-preserved) - single word, no split
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "uppercase", "Lowercased version");
        assert_eq!(tokens[1].text, "UPPERCASE", "Case-preserved version");
    }

    #[test]
    fn test_mixed_case_all_lower() {
        let tokens = tokenize_code("MixedCaseValue");
        // Verify the lowercased tokens exist (we check behavioral requirement, not exact count)
        let has_split_tokens = tokens.iter().any(|t| t.text == "mixed")
            && tokens.iter().any(|t| t.text == "case")
            && tokens.iter().any(|t| t.text == "value");
        assert!(has_split_tokens, "Should have split tokens");
    }

    // ==================== Real-World Code Examples ====================

    #[test]
    fn test_java_style_getter() {
        let tokens = tokenize_code("getHTTPServerConnection");
        // 2 (get) + 2 (http) + 2 (server) + 2 (connection) + 2 (complete) = 10
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "get");
        assert_eq!(tokens[2].text, "http");
        assert_eq!(tokens[3].text, "HTTP");
        // Check lowercase complete exists
        let has_lowercase_complete = tokens.iter().any(|t| t.text == "gethttpserverconnection");
        assert!(has_lowercase_complete);
    }

    #[test]
    fn test_rust_style_snake() {
        let tokens = tokenize_code("parse_json_from_file");
        // 2 (lowercased, case-preserved) - no split due to underscores
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "parse_json_from_file");
        assert_eq!(tokens[1].text, "parse_json_from_file");
    }

    #[test]
    fn test_css_class_name() {
        let tokens = tokenize_code("btn-primary-lg");
        // 2 (lowercased, case-preserved) - no split due to hyphens
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "btn-primary-lg");
        assert_eq!(tokens[1].text, "btn-primary-lg");
    }

    #[test]
    fn test_environment_variable() {
        let tokens = tokenize_code("DATABASE_CONNECTION_TIMEOUT");
        // 2 (lowercased, case-preserved) - no split due to underscores
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "database_connection_timeout");
        assert_eq!(tokens[1].text, "DATABASE_CONNECTION_TIMEOUT");
    }

    #[test]
    fn test_kubernetes_pod_name() {
        let tokens = tokenize_code("klask-backend-prod");
        // 2 (lowercased, case-preserved) - no split due to hyphens
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "klask-backend-prod");
        assert_eq!(tokens[1].text, "klask-backend-prod");
    }

    #[test]
    fn test_go_style_interface() {
        let tokens = tokenize_code("io.Reader");
        // The dot is not a split character, so it should be part of token
        assert!(tokens.len() >= 1);
    }

    // ==================== Case Sensitivity ====================

    #[test]
    fn test_all_output_lowercase() {
        let test_cases =
            vec!["CamelCase", "PascalCase", "UPPERCASE", "lowercase", "MixedCASE", "HTMLParser", "HTTPSConnection"];

        for test_case in test_cases {
            let tokens = tokenize_code(test_case);
            // With case-preserving tokens, we should have SOME lowercase tokens even if we also have case-preserved ones
            let has_lowercase_tokens = tokens.iter().any(|t| {
                t.text.chars().all(|c| {
                    c.is_lowercase() || ['_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ' '].contains(&c)
                })
            });
            assert!(
                has_lowercase_tokens,
                "Test case '{}' should have at least one lowercased token",
                test_case
            );
        }
    }

    // ==================== Token Properties ====================

    #[test]
    fn test_token_position_increments() {
        let tokens = tokenize_code("camelCaseExample");
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.position, i, "Token positions should increment");
        }
    }

    #[test]
    fn test_token_offset_values() {
        let tokens = tokenize_code("HTMLParser");
        assert!(tokens[0].offset_from >= 0, "First token offset should be >= 0");
        // With case-preserving tokens, we have multiple tokens at the same position
        // so we just verify that offsets are valid, not strictly increasing
        for token in &tokens {
            assert!(
                token.offset_from <= token.offset_to,
                "offset_from should be <= offset_to for token '{}'",
                token.text
            );
        }
    }

    #[test]
    fn test_token_length_matches_text() {
        let tokens = tokenize_code("myFunction");
        for token in tokens {
            // Verify that offset_to is greater than offset_from
            assert!(
                token.offset_to > token.offset_from,
                "Token offset_to should be > offset_from"
            );
        }
    }

    // ==================== Complete Token Cases ====================

    #[test]
    fn test_camel_case_with_complete_token() {
        let tokens = tokenize_code("readerTemplate");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Should have individual split tokens
        assert!(
            token_texts.contains(&"reader".to_string()),
            "Should have 'reader' token"
        );
        assert!(
            token_texts.contains(&"template".to_string()),
            "Should have 'template' token"
        );

        // Should also have the complete lowercased identifier
        assert!(
            token_texts.contains(&"readertemplate".to_string()),
            "Should have complete 'readertemplate' token"
        );
    }

    #[test]
    fn test_pascal_case_with_complete_token() {
        let tokens = tokenize_code("HTMLParser");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Should have individual split tokens
        assert!(token_texts.contains(&"html".to_string()), "Should have 'html' token");
        assert!(
            token_texts.contains(&"parser".to_string()),
            "Should have 'parser' token"
        );

        // Should also have the complete lowercased identifier
        assert!(
            token_texts.contains(&"htmlparser".to_string()),
            "Should have complete 'htmlparser' token"
        );
    }

    #[test]
    fn test_three_part_camel_case_with_complete_token() {
        let tokens = tokenize_code("myVariableName");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Should have individual split tokens
        assert!(token_texts.contains(&"my".to_string()), "Should have 'my' token");
        assert!(
            token_texts.contains(&"variable".to_string()),
            "Should have 'variable' token"
        );
        assert!(token_texts.contains(&"name".to_string()), "Should have 'name' token");

        // Should also have the complete lowercased identifier
        assert!(
            token_texts.contains(&"myvariablename".to_string()),
            "Should have complete 'myvariablename' token"
        );
    }

    #[test]
    fn test_get_http_response_with_complete_token() {
        let tokens = tokenize_code("getHTTPResponse");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Should have individual split tokens
        assert!(token_texts.contains(&"get".to_string()), "Should have 'get' token");
        assert!(token_texts.contains(&"http".to_string()), "Should have 'http' token");
        assert!(
            token_texts.contains(&"response".to_string()),
            "Should have 'response' token"
        );

        // Should also have the complete lowercased identifier
        assert!(
            token_texts.contains(&"gethttpresponse".to_string()),
            "Should have complete 'gethttpresponse' token"
        );
    }

    #[test]
    fn test_snake_case_no_extra_complete_token() {
        let tokens = tokenize_code("snake_case");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Snake case doesn't split, so we have 2 tokens (lowercased and case-preserved)
        assert_eq!(
            token_texts.len(),
            2,
            "Snake case should have 2 tokens (lowercase + case-preserved)"
        );
        assert!(
            token_texts.contains(&"snake_case".to_string()),
            "Should have 'snake_case' token"
        );
        // The case-preserved version is the same for snake_case (all lowercase anyway)
        assert_eq!(
            token_texts[0], token_texts[1],
            "Both versions should be identical for all-lowercase"
        );
    }

    #[test]
    fn test_mixed_snake_and_camel_with_complete_token() {
        let tokens = tokenize_code("my_functionName");
        let token_texts: Vec<_> = tokens.iter().map(|t| t.text.clone()).collect();

        // Should have individual split tokens
        assert!(
            token_texts.contains(&"my_function".to_string()),
            "Should have 'my_function' token"
        );
        assert!(token_texts.contains(&"name".to_string()), "Should have 'name' token");

        // The word boundary is on the second word, so complete token should be for "my_functionName"
        assert!(
            token_texts.contains(&"my_functionname".to_string()),
            "Should have complete 'my_functionname' token"
        );
    }
}
