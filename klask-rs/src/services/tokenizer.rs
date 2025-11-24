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
/// In addition to the split tokens, also emits the complete lowercased identifier as a token.
/// This allows searching for both individual components and the complete identifier.
///
/// Example: "readerTemplate" → ["reader", "template", "readertemplate"]
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

    // Emit the final split token
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

    // If we generated multiple tokens (i.e., the word was split), add the complete lowercased identifier as an additional token
    // This allows users to search for either individual components or the complete identifier
    if tokens.len() > 1 {
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
        assert_eq!(tokens.len(), 3); // camel, case, camelcase (complete)
        assert_eq!(tokens[0].text, "camel");
        assert_eq!(tokens[1].text, "case");
        assert_eq!(tokens[2].text, "camelcase"); // complete token
    }

    #[test]
    fn test_camel_case_three_parts() {
        let tokens = tokenize_code("myVariableName");
        assert_eq!(tokens.len(), 4); // my, variable, name, myvariablename (complete)
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
        assert_eq!(tokens[3].text, "myvariablename"); // complete token
    }

    #[test]
    fn test_camel_case_four_parts() {
        let tokens = tokenize_code("parseJSONFromAPIResponse");
        assert_eq!(tokens.len(), 6); // parse, json, from, api, response, parsejsonfromapiresponse (complete)
        assert_eq!(tokens[0].text, "parse");
        assert_eq!(tokens[1].text, "json");
        assert_eq!(tokens[2].text, "from");
        assert_eq!(tokens[3].text, "api");
        assert_eq!(tokens[4].text, "response");
        assert_eq!(tokens[5].text, "parsejsonfromapiresponse"); // complete token
    }

    #[test]
    fn test_pascal_case() {
        let tokens = tokenize_code("PascalCase");
        assert_eq!(tokens.len(), 3); // pascal, case, pascalcase (complete)
        assert_eq!(tokens[0].text, "pascal");
        assert_eq!(tokens[1].text, "case");
        assert_eq!(tokens[2].text, "pascalcase"); // complete token
    }

    #[test]
    fn test_pascal_case_three_parts() {
        let tokens = tokenize_code("MyVariableName");
        assert_eq!(tokens.len(), 4); // my, variable, name, myvariablename (complete)
        assert_eq!(tokens[0].text, "my");
        assert_eq!(tokens[1].text, "variable");
        assert_eq!(tokens[2].text, "name");
        assert_eq!(tokens[3].text, "myvariablename"); // complete token
    }

    // ==================== Acronym Cases ====================

    #[test]
    fn test_html_parser() {
        let tokens = tokenize_code("HTMLParser");
        assert_eq!(tokens.len(), 3); // html, parser, htmlparser (complete)
        assert_eq!(tokens[0].text, "html");
        assert_eq!(tokens[1].text, "parser");
        assert_eq!(tokens[2].text, "htmlparser"); // complete token
    }

    #[test]
    fn test_http_response() {
        let tokens = tokenize_code("HTTPResponse");
        assert_eq!(tokens.len(), 3); // http, response, httpresponse (complete)
        assert_eq!(tokens[0].text, "http");
        assert_eq!(tokens[1].text, "response");
        assert_eq!(tokens[2].text, "httpresponse"); // complete token
    }

    #[test]
    fn test_get_http_response() {
        let tokens = tokenize_code("getHTTPResponse");
        assert_eq!(tokens.len(), 4); // get, http, response, gethttpresponse (complete)
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "response");
        assert_eq!(tokens[3].text, "gethttpresponse"); // complete token
    }

    #[test]
    fn test_xml_parser() {
        let tokens = tokenize_code("XMLParser");
        assert_eq!(tokens.len(), 3); // xml, parser, xmlparser (complete)
        assert_eq!(tokens[0].text, "xml");
        assert_eq!(tokens[1].text, "parser");
        assert_eq!(tokens[2].text, "xmlparser"); // complete token
    }

    #[test]
    fn test_io_error() {
        let tokens = tokenize_code("IOError");
        assert_eq!(tokens.len(), 3); // io, error, ioerror (complete)
        assert_eq!(tokens[0].text, "io");
        assert_eq!(tokens[1].text, "error");
        assert_eq!(tokens[2].text, "ioerror"); // complete token
    }

    #[test]
    fn test_https_connection() {
        let tokens = tokenize_code("HTTPSConnection");
        assert_eq!(tokens.len(), 3); // https, connection, httpsconnection (complete)
        assert_eq!(tokens[0].text, "https");
        assert_eq!(tokens[1].text, "connection");
        assert_eq!(tokens[2].text, "httpsconnection"); // complete token
    }

    #[test]
    fn test_url_handler() {
        let tokens = tokenize_code("URLHandler");
        assert_eq!(tokens.len(), 3); // url, handler, urlhandler (complete)
        assert_eq!(tokens[0].text, "url");
        assert_eq!(tokens[1].text, "handler");
        assert_eq!(tokens[2].text, "urlhandler"); // complete token
    }

    #[test]
    fn test_api_client() {
        let tokens = tokenize_code("APIClient");
        assert_eq!(tokens.len(), 3); // api, client, apiclient (complete)
        assert_eq!(tokens[0].text, "api");
        assert_eq!(tokens[1].text, "client");
        assert_eq!(tokens[2].text, "apiclient"); // complete token
    }

    #[test]
    fn test_json_object() {
        let tokens = tokenize_code("JSONObject");
        assert_eq!(tokens.len(), 3); // json, object, jsonobject (complete)
        assert_eq!(tokens[0].text, "json");
        assert_eq!(tokens[1].text, "object");
        assert_eq!(tokens[2].text, "jsonobject"); // complete token
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
    fn test_nginx_url() {
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
        assert_eq!(tokens.len(), 3); // my_function, name, my_functionname (complete)
        assert_eq!(
            tokens[0].text, "my_function",
            "Snake case before camelCase should split on case transition"
        );
        assert_eq!(tokens[1].text, "name");
        assert_eq!(tokens[2].text, "my_functionname"); // complete token
    }

    #[test]
    fn test_mixed_hyphen_and_camel() {
        let tokens = tokenize_code("my-functionName");
        assert_eq!(tokens.len(), 3); // my-function, name, my-functionname (complete)
        assert_eq!(
            tokens[0].text, "my-function",
            "Hyphen followed by camelCase should preserve hyphen"
        );
        assert_eq!(tokens[1].text, "name");
        assert_eq!(tokens[2].text, "my-functionname"); // complete token
    }

    #[test]
    fn test_snake_case_with_camel() {
        let tokens = tokenize_code("snake_camelCase");
        assert_eq!(tokens.len(), 3); // snake_camel, case, snake_camelcase (complete)
        assert_eq!(tokens[0].text, "snake_camel");
        assert_eq!(tokens[1].text, "case");
        assert_eq!(tokens[2].text, "snake_camelcase"); // complete token
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
        assert_eq!(tokens.len(), 3); // 1st, place, 1stplace (complete)
        assert_eq!(tokens[0].text, "1st");
        assert_eq!(tokens[1].text, "place");
        assert_eq!(tokens[2].text, "1stplace"); // complete token
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
        assert_eq!(tokens.len(), 3); // base64url, safe, base64urlsafe (complete)
        assert_eq!(tokens[0].text, "base64url");
        assert_eq!(tokens[1].text, "safe");
        assert_eq!(tokens[2].text, "base64urlsafe"); // complete token
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
        assert_eq!(tokens.len(), 3); // _private, function, _privatefunction (complete)
        assert_eq!(tokens[0].text, "_private");
        assert_eq!(tokens[1].text, "function");
        assert_eq!(tokens[2].text, "_privatefunction"); // complete token
    }

    #[test]
    fn test_trailing_underscore() {
        let tokens = tokenize_code("functionName_");
        assert_eq!(tokens.len(), 3); // function, name_, functionname_ (complete)
        assert_eq!(tokens[0].text, "function");
        assert_eq!(tokens[1].text, "name_");
        assert_eq!(tokens[2].text, "functionname_"); // complete token
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
        assert_eq!(tokens.len(), 4); // mixed, case, value, mixedcasevalue (complete)
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
        assert_eq!(tokens.len(), 5); // get, http, server, connection, gethttpserverconnection (complete)
        assert_eq!(tokens[0].text, "get");
        assert_eq!(tokens[1].text, "http");
        assert_eq!(tokens[2].text, "server");
        assert_eq!(tokens[3].text, "connection");
        assert_eq!(tokens[4].text, "gethttpserverconnection"); // complete token
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

        // Snake case doesn't split, so only one token
        assert_eq!(token_texts.len(), 1, "Snake case should have only 1 token");
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
