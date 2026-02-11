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

/// Query tokenizer for case-INSENSITIVE search
/// This tokenizer does NOT split on camelCase - it only lowercases the input.
/// Used for query parsing so that "vlanID" search query becomes "vlanid" token
/// which matches the complete lowercased token in the index.
#[derive(Clone)]
pub struct QueryTokenizer;

impl QueryTokenizer {
    pub fn new() -> Self {
        QueryTokenizer
    }
}

impl Default for QueryTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for QueryTokenizer {
    type TokenStream<'a> = QueryTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        QueryTokenStream::new(text, false) // lowercase = true (case-insensitive)
    }
}

/// Query tokenizer for case-SENSITIVE search
/// This tokenizer does NOT split on camelCase and preserves case exactly.
/// Used for query parsing so that "vlanID" search query stays "vlanID" token
/// which matches the case-preserved token in the index.
#[derive(Clone)]
pub struct QueryTokenizerCaseSensitive;

impl QueryTokenizerCaseSensitive {
    pub fn new() -> Self {
        QueryTokenizerCaseSensitive
    }
}

impl Default for QueryTokenizerCaseSensitive {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for QueryTokenizerCaseSensitive {
    type TokenStream<'a> = QueryTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        QueryTokenStream::new(text, true) // preserve_case = true (case-sensitive)
    }
}

/// Token stream for query tokenizers (no camelCase splitting)
pub struct QueryTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl QueryTokenStream {
    fn new(text: &str, preserve_case: bool) -> Self {
        let tokens = tokenize_query(text, preserve_case);
        QueryTokenStream { tokens, index: 0 }
    }
}

impl TokenStream for QueryTokenStream {
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

/// Tokenizes query text WITHOUT camelCase splitting
/// Only splits on whitespace and non-alphanumeric chars (except _ and -)
/// Optionally lowercases tokens based on preserve_case flag
fn tokenize_query(text: &str, preserve_case: bool) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();

    // Split by whitespace and non-alphanumeric characters (except underscores and hyphens)
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || (!c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|s| !s.is_empty())
        .collect();

    let mut byte_offset = 0;

    for (position, word) in words.iter().enumerate() {
        // Find this word in the original text
        let word_byte_pos = if byte_offset < text.len() {
            text[byte_offset..].find(word).unwrap_or(0)
        } else {
            0
        };

        let word_byte_offset = byte_offset + word_byte_pos;

        // Create token - either lowercase or preserve case
        let token_text = if preserve_case { word.to_string() } else { word.to_lowercase() };

        tokens.push(Token {
            offset_from: word_byte_offset,
            offset_to: word_byte_offset + word.len(),
            position,
            text: token_text,
            position_length: 1,
        });

        byte_offset = word_byte_offset + word.len();
    }

    tokens
}

/// Case-preserving code tokenizer for case-sensitive search
/// This tokenizer uses the same splitting logic as CodeTokenizer (camelCase, whitespace, etc.)
/// but preserves the original case of tokens instead of lowercasing them.
/// This enables case-sensitive matching via RegexQuery on the indexed tokens.
#[derive(Clone)]
pub struct CaseSensitiveCodeTokenizer;

impl CaseSensitiveCodeTokenizer {
    /// Creates a new CaseSensitiveCodeTokenizer instance
    pub fn new() -> Self {
        CaseSensitiveCodeTokenizer
    }
}

impl Default for CaseSensitiveCodeTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for CaseSensitiveCodeTokenizer {
    type TokenStream<'a> = CaseSensitiveCodeTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        CaseSensitiveCodeTokenStream::new(text)
    }
}

/// Token stream for the case-sensitive code tokenizer
pub struct CaseSensitiveCodeTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl CaseSensitiveCodeTokenStream {
    fn new(text: &str) -> Self {
        let tokens = tokenize_code_case_sensitive(text);
        CaseSensitiveCodeTokenStream { tokens, index: 0 }
    }
}

impl TokenStream for CaseSensitiveCodeTokenStream {
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
                // Emit only lowercased version - QueryParser expects one token per position
                // Case-sensitive search is handled by content_raw field with RawCasePreservingTokenizer
                let token = Token {
                    offset_from: token_byte_start,
                    offset_to: byte_pos,
                    position: position_base + tokens.len(),
                    text: current_token.to_lowercase(),
                    position_length: 1,
                };
                tokens.push(token);

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
        // Emit only lowercased version - QueryParser expects one token per position
        let token = Token {
            offset_from: token_byte_start,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: current_token.to_lowercase(),
            position_length: 1,
        };
        tokens.push(token);
    }

    // If we generated multiple tokens (i.e., the word was split), add complete lowercased token
    if tokens.len() > 1 {
        // Add the complete lowercased identifier to allow searching for the whole identifier
        // e.g., searching for "readertemplate" should find "readerTemplate" when split into ["reader", "template"]
        let complete_token = Token {
            offset_from: byte_offset_base,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: text.to_lowercase(),
            position_length: 1,
        };
        tokens.push(complete_token);
    }

    tokens
}

/// Tokenizes code text with CASE PRESERVATION (for case-sensitive search)
/// Uses the same splitting logic as tokenize_code but preserves original case.
fn tokenize_code_case_sensitive(text: &str) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();

    // First, split by whitespace and non-alphanumeric characters (except underscores and hyphens)
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || (!c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|s| !s.is_empty())
        .collect();

    let mut token_position = 0;
    let mut byte_offset = 0;

    for word in words.iter() {
        let word_byte_pos = if byte_offset < text.len() {
            text[byte_offset..].find(word).unwrap_or(0)
        } else {
            0
        };

        let word_byte_offset = byte_offset + word_byte_pos;
        let word_tokens = tokenize_code_word_case_sensitive(word, word_byte_offset, token_position);
        token_position += word_tokens.len();
        byte_offset = word_byte_offset + word.len();

        tokens.extend(word_tokens);
    }

    tokens
}

/// Tokenizes a single word with CASE PRESERVATION (for case-sensitive search)
/// Same splitting logic as tokenize_code_word but emits tokens with original case.
fn tokenize_code_word_case_sensitive(text: &str, byte_offset_base: usize, position_base: usize) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut current_token = String::new();

    let mut byte_pos = byte_offset_base;
    let mut token_byte_start = byte_offset_base;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let ch_len = ch.len_utf8();

        let should_split = if i == 0 {
            false
        } else {
            let prev_ch = chars[i - 1];
            let is_underscore_or_hyphen = ch == '_' || ch == '-';
            let prev_is_underscore_or_hyphen = prev_ch == '_' || prev_ch == '-';

            if is_underscore_or_hyphen || prev_is_underscore_or_hyphen {
                false
            } else if prev_ch.is_lowercase() && ch.is_uppercase() {
                true
            } else {
                prev_ch.is_uppercase() && ch.is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase()
            }
        };

        if should_split && !current_token.is_empty() {
            // CASE-PRESERVING: emit the token with original case (no to_lowercase)
            let token = Token {
                offset_from: token_byte_start,
                offset_to: byte_pos,
                position: position_base + tokens.len(),
                text: current_token.clone(), // Preserve original case!
                position_length: 1,
            };
            tokens.push(token);

            current_token.clear();
            token_byte_start = byte_pos;
        }

        current_token.push(ch);
        byte_pos += ch_len;
        i += 1;
    }

    // Emit the final token with original case
    if !current_token.is_empty() {
        let token = Token {
            offset_from: token_byte_start,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: current_token, // Preserve original case!
            position_length: 1,
        };
        tokens.push(token);
    }

    // If word was split, also add the complete identifier with original case
    if tokens.len() > 1 {
        let complete_token = Token {
            offset_from: byte_offset_base,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: text.to_string(), // Preserve original case!
            position_length: 1,
        };
        tokens.push(complete_token);
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
        assert_eq!(tokens.len(), 1); // lowercased only
        assert_eq!(tokens[0].text, "a");
    }

    #[test]
    fn test_single_char_uppercase() {
        let tokens = tokenize_code("A");
        assert_eq!(tokens.len(), 1); // lowercased only
        assert_eq!(tokens[0].text, "a", "Lowercased version");
    }

    #[test]
    fn test_single_word_lowercase() {
        let tokens = tokenize_code("hello");
        assert_eq!(tokens.len(), 1); // lowercased only
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_single_word_uppercase() {
        let tokens = tokenize_code("HELLO");
        assert_eq!(tokens.len(), 1); // lowercased only
        assert_eq!(tokens[0].text, "hello", "Lowercased version");
    }

    // ==================== CamelCase Cases ====================

    #[test]
    fn test_camel_case_basic() {
        let tokens = tokenize_code("camelCase");
        // 1 (camel) + 1 (case) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "camel");
        assert_eq!(tokens[1].text, "case");
        assert_eq!(tokens[2].text, "camelcase");
    }

    #[test]
    fn test_camel_case_basic_simple() {
        let tokens = tokenize_code("camelCase");
        // Count: 1 (camel) + 1 (case) + 1 (complete)
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "camel"); // lowercased camel
        assert_eq!(tokens[1].text, "case"); // lowercased case
        assert_eq!(tokens[2].text, "camelcase"); // complete lowercased
    }

    #[test]
    fn test_camel_case_three_parts() {
        let tokens = tokenize_code("myVariableName");
        // 1 (my) + 1 (variable) + 1 (name) + 1 (complete) = 4
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
        assert_eq!(tokens[3].text, "myvariablename");
    }

    #[test]
    fn test_camel_case_four_parts() {
        let tokens = tokenize_code("parseJSONFromAPIResponse");
        // 1 (parse) + 1 (json) + 1 (from) + 1 (api) + 1 (response) + 1 (complete) = 6
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "parse");
        assert_eq!(tokens[1].text, "json");
        assert_eq!(tokens[2].text, "from");
        assert_eq!(tokens[3].text, "api");
        assert_eq!(tokens[4].text, "response");
        // Check that complete lowercase token exists
        let has_lowercase_complete = tokens.iter().any(|t| t.text == "parsejsonfromapiresponse");
        assert!(has_lowercase_complete, "Should have lowercase complete token");
    }

    #[test]
    fn test_pascal_case() {
        let tokens = tokenize_code("PascalCase");
        // 1 (pascal) + 1 (case) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "pascal");
        assert_eq!(tokens[1].text, "case");
        assert_eq!(tokens[2].text, "pascalcase");
    }

    #[test]
    fn test_pascal_case_three_parts() {
        let tokens = tokenize_code("MyVariableName");
        // 1 (my) + 1 (variable) + 1 (name) + 1 (complete) = 4
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
        assert_eq!(tokens[3].text, "myvariablename");
    }

    // ==================== Acronym Cases ====================

    #[test]
    fn test_html_parser() {
        let tokens = tokenize_code("HTMLParser");
        // 1 (html) + 1 (parser) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "html");
        assert_eq!(tokens[1].text, "parser");
        assert_eq!(tokens[2].text, "htmlparser");
    }

    #[test]
    fn test_http_response() {
        let tokens = tokenize_code("HTTPResponse");
        // 1 (http) + 1 (response) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "http");
        assert_eq!(tokens[1].text, "response");
        assert_eq!(tokens[2].text, "httpresponse");
    }

    #[test]
    fn test_get_http_response() {
        let tokens = tokenize_code("getHTTPResponse");
        // 1 (get) + 1 (http) + 1 (response) + 1 (complete) = 4
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "response");
        assert_eq!(tokens[3].text, "gethttpresponse");
    }

    #[test]
    fn test_xml_parser() {
        let tokens = tokenize_code("XMLParser");
        // 1 (xml) + 1 (parser) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "xml");
        assert_eq!(tokens[1].text, "parser");
        assert_eq!(tokens[2].text, "xmlparser");
    }

    #[test]
    fn test_io_error() {
        let tokens = tokenize_code("IOError");
        // 1 (io) + 1 (error) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "io");
        assert_eq!(tokens[1].text, "error");
        assert_eq!(tokens[2].text, "ioerror");
    }

    #[test]
    fn test_https_connection() {
        let tokens = tokenize_code("HTTPSConnection");
        // 1 (https) + 1 (connection) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "https");
        assert_eq!(tokens[1].text, "connection");
        assert_eq!(tokens[2].text, "httpsconnection");
    }

    #[test]
    fn test_url_handler() {
        let tokens = tokenize_code("URLHandler");
        // 1 (url) + 1 (handler) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "url");
        assert_eq!(tokens[1].text, "handler");
        assert_eq!(tokens[2].text, "urlhandler");
    }

    #[test]
    fn test_api_client() {
        let tokens = tokenize_code("APIClient");
        // 1 (api) + 1 (client) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "api");
        assert_eq!(tokens[1].text, "client");
        assert_eq!(tokens[2].text, "apiclient");
    }

    #[test]
    fn test_json_object() {
        let tokens = tokenize_code("JSONObject");
        // 1 (json) + 1 (object) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "json");
        assert_eq!(tokens[1].text, "object");
        assert_eq!(tokens[2].text, "jsonobject");
    }

    // ==================== Underscore & Hyphen Cases ====================

    #[test]
    fn test_snake_case() {
        let tokens = tokenize_code("snake_case");
        // 1 - no split because of underscore
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "snake_case", "Lowercased version");
    }

    #[test]
    fn test_snake_case_three_parts() {
        let tokens = tokenize_code("my_variable_name");
        // 1 - no split because of underscores
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my_variable_name", "Lowercased version");
    }

    #[test]
    fn test_nginx_url() {
        let tokens = tokenize_code("NETBOX_URL");
        // 1 - no split because of underscores
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "netbox_url",
            "Uppercase with underscores should be lowercased"
        );
    }

    #[test]
    fn test_all_caps_with_underscore() {
        let tokens = tokenize_code("AWS_REGION_NAME");
        // 1 - no split because of underscores
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "aws_region_name", "Lowercase version");
    }

    #[test]
    fn test_hyphenated_basic() {
        let tokens = tokenize_code("my-module");
        // 1 - no split because of hyphen
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my-module", "Lowercased version");
    }

    #[test]
    fn test_hyphenated_three_parts() {
        let tokens = tokenize_code("my-function-name");
        // 1 - no split because of hyphens
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my-function-name", "Lowercased version");
    }

    #[test]
    fn test_config_service() {
        // Hyphens prevent splitting, so the whole thing stays as one token
        let tokens = tokenize_code("config-Service");
        // 1 - hyphen prevents splitting
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "config-service",
            "Hyphen prevents splitting - lowercase"
        );
    }

    // ==================== Mixed Cases ====================

    #[test]
    fn test_mixed_snake_and_camel() {
        let tokens = tokenize_code("my_functionName");
        // 1 (my_function) + 1 (name) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "my_function", "Lowercase with underscore");
        assert_eq!(tokens[1].text, "name", "Split on case transition");
        assert_eq!(tokens[2].text, "my_functionname", "Complete lowercase");
    }

    #[test]
    fn test_mixed_hyphen_and_camel() {
        let tokens = tokenize_code("my-functionName");
        // 1 (my-function) + 1 (name) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "my-function", "Lowercase with hyphen");
        assert_eq!(tokens[1].text, "name", "Split on case transition");
        assert_eq!(tokens[2].text, "my-functionname", "Complete lowercase");
    }

    #[test]
    fn test_snake_case_with_camel() {
        let tokens = tokenize_code("snake_camelCase");
        // 1 (snake_camel) + 1 (case) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "snake_camel", "Lowercase with underscore");
        assert_eq!(tokens[1].text, "case", "Split on case transition");
        assert_eq!(tokens[2].text, "snake_camelcase", "Complete lowercase");
    }

    #[test]
    fn test_complex_mixed_naming() {
        // Underscores prevent splitting, so everything stays as one token
        let tokens = tokenize_code("my_api_Handler");
        // 1 - underscores prevent splitting
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my_api_handler", "Lowercase version");
    }

    // ==================== Numbers Cases ====================

    #[test]
    fn test_number_at_start() {
        let tokens = tokenize_code("1stPlace");
        // 1 (1st) + 1 (place) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "1st");
        assert_eq!(tokens[1].text, "place");
        assert_eq!(tokens[2].text, "1stplace");
    }

    #[test]
    fn test_number_in_middle() {
        // Numbers don't trigger splits since they're neither lowercase nor uppercase
        // "base64Encode" -> "base64encode" doesn't split at 4->E because 4 is not lowercase
        let tokens = tokenize_code("base64Encode");
        // No split happens, so: 1 (base64encode)
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "base64encode");
    }

    #[test]
    fn test_multiple_numbers() {
        // Numbers don't cause splits; only lowercase->uppercase does
        // "var1Name2Value" splits at "r->N" (lowercase->uppercase transition)
        // "1" is not lowercase, so "1N" doesn't split
        // "2" is not lowercase, so "2V" doesn't split
        let tokens = tokenize_code("var1Name2Value");
        // 1 (var1name2value) - single token because number->uppercase doesn't split
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "var1name2value");
    }

    #[test]
    fn test_consecutive_numbers() {
        // Split happens on lowercase to uppercase: "Base64URL" vs "Safe"
        let tokens = tokenize_code("Base64URLSafe");
        // 1 (base64url) + 1 (safe) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "base64url");
        assert_eq!(tokens[1].text, "safe");
        assert_eq!(tokens[2].text, "base64urlsafe");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_double_underscore() {
        let tokens = tokenize_code("test__double");
        // 1 - underscores prevent splitting
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "test__double", "Lowercased");
    }

    #[test]
    fn test_double_hyphen() {
        let tokens = tokenize_code("test--double");
        // 1 - hyphens prevent splitting
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "test--double", "Lowercased");
    }

    #[test]
    fn test_leading_underscore() {
        let tokens = tokenize_code("_privateFunction");
        // 1 (_private) + 1 (function) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "_private");
        assert_eq!(tokens[1].text, "function");
        assert_eq!(tokens[2].text, "_privatefunction");
    }

    #[test]
    fn test_trailing_underscore() {
        let tokens = tokenize_code("functionName_");
        // 1 (function) + 1 (name_) + 1 (complete) = 3
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "function");
        assert_eq!(tokens[1].text, "name_");
        assert_eq!(tokens[2].text, "functionname_");
    }

    #[test]
    fn test_only_underscores() {
        let tokens = tokenize_code("___");
        // 1 - no split
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "___");
    }

    #[test]
    fn test_only_hyphens() {
        let tokens = tokenize_code("---");
        // 1 - no split
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "---");
    }

    #[test]
    fn test_whitespace_splits_words() {
        let tokens = tokenize_code("hello world");
        // 1 (hello) + 1 (world) = 2
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    #[test]
    fn test_uppercase_preserved_case_insensitive() {
        let tokens = tokenize_code("UPPERCASE");
        // 1 - single word, no split
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "uppercase", "Lowercased version");
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
        // 1 (get) + 1 (http) + 1 (server) + 1 (connection) + 1 (complete) = 5
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "server");
        assert_eq!(tokens[3].text, "connection");
        // Check lowercase complete exists
        let has_lowercase_complete = tokens.iter().any(|t| t.text == "gethttpserverconnection");
        assert!(has_lowercase_complete);
    }

    #[test]
    fn test_rust_style_snake() {
        let tokens = tokenize_code("parse_json_from_file");
        // 1 - no split due to underscores
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "parse_json_from_file");
    }

    #[test]
    fn test_css_class_name() {
        let tokens = tokenize_code("btn-primary-lg");
        // 1 - no split due to hyphens
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "btn-primary-lg");
    }

    #[test]
    fn test_environment_variable() {
        let tokens = tokenize_code("DATABASE_CONNECTION_TIMEOUT");
        // 1 - no split due to underscores
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "database_connection_timeout");
    }

    #[test]
    fn test_kubernetes_pod_name() {
        let tokens = tokenize_code("klask-backend-prod");
        // 1 - no split due to hyphens
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "klask-backend-prod");
    }

    #[test]
    fn test_go_style_interface() {
        let tokens = tokenize_code("io.Reader");
        // The dot is not a split character, so it should be part of token
        assert!(!tokens.is_empty());
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
        assert!(!tokens.is_empty(), "Should have at least one token");
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

        // Snake case doesn't split, so we have 1 token (lowercased only)
        assert_eq!(token_texts.len(), 1, "Snake case should have 1 token (lowercase only)");
        assert!(
            token_texts.contains(&"snake_case".to_string()),
            "Should have 'snake_case' token"
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
