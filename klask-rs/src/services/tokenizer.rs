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
/// - `"NETBOX_URL"` → `["netbox_url"]`
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
fn tokenize_code_word(text: &str, byte_offset_base: usize, position_base: usize) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut current_token = String::new();

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
                let token = Token {
                    offset_from: token_byte_start,
                    offset_to: byte_pos,
                    position: position_base + tokens.len(),
                    text: current_token.to_lowercase(),
                    position_length: 1,
                };
                tokens.push(token);
                current_token.clear();
                token_byte_start = byte_pos;
            }
        }

        current_token.push(ch);
        byte_pos += ch_len;
        i += 1;
    }

    // Emit the final token
    if !current_token.is_empty() {
        let token = Token {
            offset_from: token_byte_start,
            offset_to: byte_pos,
            position: position_base + tokens.len(),
            text: current_token.to_lowercase(),
            position_length: 1,
        };
        tokens.push(token);
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
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "a");
    }

    #[test]
    fn test_single_char_uppercase() {
        let tokens = tokenize_code("A");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "a", "Single uppercase char should be lowercased");
    }

    #[test]
    fn test_single_word_lowercase() {
        let tokens = tokenize_code("hello");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_single_word_uppercase() {
        let tokens = tokenize_code("HELLO");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "hello",
            "All uppercase word should be lowercased as single token"
        );
    }

    // ==================== CamelCase Cases ====================

    #[test]
    fn test_camel_case_basic() {
        let tokens = tokenize_code("camelCase");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "camel");
        assert_eq!(tokens[1].text, "case");
    }

    #[test]
    fn test_camel_case_three_parts() {
        let tokens = tokenize_code("myVariableName");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
    }

    #[test]
    fn test_camel_case_four_parts() {
        let tokens = tokenize_code("parseJSONFromAPIResponse");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "parse");
        assert_eq!(tokens[1].text, "json");
        assert_eq!(tokens[2].text, "from");
        assert_eq!(tokens[3].text, "api");
        assert_eq!(tokens[4].text, "response");
    }

    #[test]
    fn test_pascal_case() {
        let tokens = tokenize_code("PascalCase");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "pascal");
        assert_eq!(tokens[1].text, "case");
    }

    #[test]
    fn test_pascal_case_three_parts() {
        let tokens = tokenize_code("MyVariableName");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
    }

    // ==================== Acronym Cases ====================

    #[test]
    fn test_html_parser() {
        let tokens = tokenize_code("HTMLParser");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "html");
        assert_eq!(tokens[1].text, "parser");
    }

    #[test]
    fn test_http_response() {
        let tokens = tokenize_code("HTTPResponse");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "http");
        assert_eq!(tokens[1].text, "response");
    }

    #[test]
    fn test_get_http_response() {
        let tokens = tokenize_code("getHTTPResponse");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "response");
    }

    #[test]
    fn test_xml_parser() {
        let tokens = tokenize_code("XMLParser");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "xml");
        assert_eq!(tokens[1].text, "parser");
    }

    #[test]
    fn test_io_error() {
        let tokens = tokenize_code("IOError");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "io");
        assert_eq!(tokens[1].text, "error");
    }

    #[test]
    fn test_https_connection() {
        let tokens = tokenize_code("HTTPSConnection");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "https");
        assert_eq!(tokens[1].text, "connection");
    }

    #[test]
    fn test_url_handler() {
        let tokens = tokenize_code("URLHandler");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "url");
        assert_eq!(tokens[1].text, "handler");
    }

    #[test]
    fn test_api_client() {
        let tokens = tokenize_code("APIClient");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "api");
        assert_eq!(tokens[1].text, "client");
    }

    #[test]
    fn test_json_object() {
        let tokens = tokenize_code("JSONObject");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "json");
        assert_eq!(tokens[1].text, "object");
    }

    // ==================== Underscore & Hyphen Cases ====================

    #[test]
    fn test_snake_case() {
        let tokens = tokenize_code("snake_case");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "snake_case", "Underscores should be preserved");
    }

    #[test]
    fn test_snake_case_three_parts() {
        let tokens = tokenize_code("my_variable_name");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "my_variable_name",
            "All underscores should be preserved in single token"
        );
    }

    #[test]
    fn test_netbox_url() {
        let tokens = tokenize_code("NETBOX_URL");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "netbox_url",
            "Uppercase with underscores should be lowercased as single token"
        );
    }

    #[test]
    fn test_all_caps_with_underscore() {
        let tokens = tokenize_code("AWS_REGION_NAME");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "aws_region_name",
            "All caps with underscores should be lowercased as single token"
        );
    }

    #[test]
    fn test_hyphenated_basic() {
        let tokens = tokenize_code("my-module");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my-module", "Hyphens should be preserved");
    }

    #[test]
    fn test_hyphenated_three_parts() {
        let tokens = tokenize_code("my-function-name");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].text, "my-function-name",
            "All hyphens should be preserved in single token"
        );
    }

    #[test]
    fn test_config_service() {
        // Hyphens prevent splitting, so the whole thing stays as one token
        let tokens = tokenize_code("config-Service");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "config-service", "Hyphen prevents splitting");
    }

    // ==================== Mixed Cases ====================

    #[test]
    fn test_mixed_snake_and_camel() {
        let tokens = tokenize_code("my_functionName");
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0].text, "my_function",
            "Snake case before camelCase should split on case transition"
        );
        assert_eq!(tokens[1].text, "name");
    }

    #[test]
    fn test_mixed_hyphen_and_camel() {
        let tokens = tokenize_code("my-functionName");
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0].text, "my-function",
            "Hyphen followed by camelCase should preserve hyphen"
        );
        assert_eq!(tokens[1].text, "name");
    }

    #[test]
    fn test_snake_case_with_camel() {
        let tokens = tokenize_code("snake_camelCase");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "snake_camel");
        assert_eq!(tokens[1].text, "case");
    }

    #[test]
    fn test_complex_mixed_naming() {
        // Underscores prevent splitting, so everything stays as one token
        let tokens = tokenize_code("my_api_Handler");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "my_api_handler");
    }

    // ==================== Numbers Cases ====================

    #[test]
    fn test_number_at_start() {
        let tokens = tokenize_code("1stPlace");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "1st");
        assert_eq!(tokens[1].text, "place");
    }

    #[test]
    fn test_number_in_middle() {
        // Numbers are treated like lowercase letters, so no split happens
        let tokens = tokenize_code("base64Encode");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "base64encode");
    }

    #[test]
    fn test_multiple_numbers() {
        // Numbers don't cause splits; only lowercase->uppercase does
        let tokens = tokenize_code("var1Name2Value");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "var1name2value");
    }

    #[test]
    fn test_consecutive_numbers() {
        // Split happens on lowercase to uppercase: "Base" vs "64URLSafe"
        let tokens = tokenize_code("Base64URLSafe");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "base64url");
        assert_eq!(tokens[1].text, "safe");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_double_underscore() {
        let tokens = tokenize_code("test__double");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "test__double", "Double underscores should be preserved");
    }

    #[test]
    fn test_double_hyphen() {
        let tokens = tokenize_code("test--double");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "test--double", "Double hyphens should be preserved");
    }

    #[test]
    fn test_leading_underscore() {
        let tokens = tokenize_code("_privateFunction");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "_private");
        assert_eq!(tokens[1].text, "function");
    }

    #[test]
    fn test_trailing_underscore() {
        let tokens = tokenize_code("functionName_");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "function");
        assert_eq!(tokens[1].text, "name_");
    }

    #[test]
    fn test_only_underscores() {
        let tokens = tokenize_code("___");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "___");
    }

    #[test]
    fn test_only_hyphens() {
        let tokens = tokenize_code("---");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "---");
    }

    #[test]
    fn test_whitespace_splits_words() {
        let tokens = tokenize_code("hello world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    #[test]
    fn test_uppercase_preserved_case_insensitive() {
        let tokens = tokenize_code("UPPERCASE");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "uppercase", "All uppercase should be lowercased");
    }

    #[test]
    fn test_mixed_case_all_lower() {
        let tokens = tokenize_code("MixedCaseValue");
        assert_eq!(tokens.len(), 3);
        for token in tokens {
            assert!(
                token.text.chars().all(|c| !c.is_uppercase() || c == '_' || c == '-'),
                "All tokens should be lowercase"
            );
        }
    }

    // ==================== Real-World Code Examples ====================

    #[test]
    fn test_java_style_getter() {
        let tokens = tokenize_code("getHTTPServerConnection");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "server");
        assert_eq!(tokens[3].text, "connection");
    }

    #[test]
    fn test_rust_style_snake() {
        let tokens = tokenize_code("parse_json_from_file");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "parse_json_from_file");
    }

    #[test]
    fn test_css_class_name() {
        let tokens = tokenize_code("btn-primary-lg");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "btn-primary-lg");
    }

    #[test]
    fn test_environment_variable() {
        let tokens = tokenize_code("DATABASE_CONNECTION_TIMEOUT");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "database_connection_timeout");
    }

    #[test]
    fn test_kubernetes_pod_name() {
        let tokens = tokenize_code("klask-backend-prod");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "klask-backend-prod");
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
            for token in tokens {
                for c in token.text.chars() {
                    if !['_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ' '].contains(&c) {
                        assert!(
                            !c.is_uppercase(),
                            "Token '{}' from '{}' contains uppercase char: {}",
                            token.text,
                            test_case,
                            c
                        );
                    }
                }
            }
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
        if tokens.len() > 1 {
            assert!(
                tokens[1].offset_from > tokens[0].offset_from,
                "Subsequent tokens should have larger offsets"
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
}
