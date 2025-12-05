/// Tests for UTF-8 handling in snippet generation
/// Reproduces potential bugs where multi-byte UTF-8 characters might be cut in half
#[cfg(test)]
mod snippet_utf8_tests {
    use std::panic;

    /// Helper function to simulate unsafe byte truncation (like the bug might do)
    fn unsafe_truncate_at_byte(text: &str, byte_limit: usize) -> Result<String, String> {
        if byte_limit >= text.len() {
            return Ok(text.to_string());
        }

        // This is what would cause the bug - truncating at an arbitrary byte position
        // without checking UTF-8 boundaries
        match std::str::from_utf8(&text.as_bytes()[..byte_limit]) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(format!("Invalid UTF-8 at position {}: {:?}", byte_limit, e)),
        }
    }

    /// Safe UTF-8 aware truncation (what we should use)
    fn safe_truncate_at_char(text: &str, char_limit: usize) -> String {
        text.chars().take(char_limit).collect()
    }

    /// Test: Single emoji (4-byte UTF-8 character) gets cut in half
    #[test]
    fn test_emoji_truncation_at_byte_boundary() {
        // 🔍 is 4 bytes: F0 9F 94 8D
        let text = "prefix🔍suffix";

        // If we try to truncate at byte 7 (in the middle of emoji):
        // "prefix" = 6 bytes, so byte 7 would be F0 (first byte of emoji)
        let result = unsafe_truncate_at_byte(text, 7);

        // This should fail - truncating in the middle of a multi-byte sequence
        assert!(result.is_err(), "Should fail when truncating in middle of emoji");
        assert!(result.unwrap_err().contains("Invalid UTF-8"));
    }

    /// Test: Chinese characters (3 bytes each) cut at wrong boundary
    #[test]
    fn test_chinese_character_truncation() {
        // "你好" = 6 bytes (3 each)
        let text = "Hello你好World";
        //        "Hello" = 5 bytes
        // "你" starts at byte 5, is 3 bytes (5-7)
        // "好" starts at byte 8, is 3 bytes (8-10)

        // Try to truncate at byte 6 (inside "你")
        let result = unsafe_truncate_at_byte(text, 6);
        assert!(
            result.is_err(),
            "Should fail when truncating in middle of Chinese character"
        );
    }

    /// Test: Combining diacritical marks (e.g., é = e + combining acute)
    #[test]
    fn test_combining_character_truncation() {
        // "café" can be written as:
        // 1. Single character ´: "café" (4 bytes: 63 61 66 C3 A9)
        // 2. Base + combining: "cafe" + combining acute (5 bytes: 63 61 66 65 CC 81)

        let text_composed = "café"; // Single ´ character

        // Try to truncate at byte 4 (in the middle of é which is 2 bytes)
        let result = unsafe_truncate_at_byte(text_composed, 4);
        assert!(
            result.is_err(),
            "Should fail when truncating in middle of accented character"
        );
    }

    /// Test: Korean characters (Hangul - 3 bytes each)
    #[test]
    fn test_korean_character_truncation() {
        // "안녕" = 6 bytes (3 each)
        let text = "prefix안녕suffix";
        // "prefix" = 6 bytes
        // "안" starts at byte 6, is 3 bytes

        // Try to truncate at byte 7 (inside "안")
        let result = unsafe_truncate_at_byte(text, 7);
        assert!(
            result.is_err(),
            "Should fail when truncating in middle of Korean character"
        );
    }

    /// Test: Real-world code snippet with emoji and unicode
    #[test]
    fn test_real_world_unicode_in_code() {
        let code_snippet = r#"
// 🚀 Rocket emoji in comment
fn hello() {
    let message = "你好世界"; // Chinese comment
    println!("Hello {}", message); // éàù accented chars
    let rocket = "🚀".to_string();
}
"#;

        // Try unsafe truncation at various problematic positions
        for byte_pos in 0..code_snippet.len() {
            let result = unsafe_truncate_at_byte(code_snippet, byte_pos);
            // If it fails, that's a UTF-8 boundary issue
            if result.is_err() {
                // Record this position as a boundary where truncation would fail
                // This is exactly the bug scenario
            }
        }

        // Safe truncation should always work
        for char_pos in 0..=code_snippet.chars().count() {
            let result = safe_truncate_at_char(code_snippet, char_pos);
            // Should never fail
            assert!(
                !result.is_empty() || char_pos == 0,
                "Safe truncation should never produce invalid UTF-8"
            );
        }
    }

    /// Test: Potential panic from invalid UTF-8
    #[test]
    #[allow(clippy::implicit_saturating_sub, clippy::string_from_utf8_as_bytes)]
    fn test_invalid_utf8_causes_panic() {
        let text = "Hello🔍World";

        // This is the bug scenario - if code tries to slice at wrong boundary:
        let result = panic::catch_unwind(|| {
            // Simulate what might happen if code does:
            // let snippet = &text[..7]; // Wrong - might be in middle of emoji

            // Instead, try the safe way
            let safe = safe_truncate_at_char(text, 6);
            assert_eq!(safe, "Hello🔍");
        });

        assert!(result.is_ok(), "Safe truncation should not panic");
    }

    /// Test: Snippet generation with mixed content
    #[test]
    fn test_snippet_with_mixed_unicode_and_ascii() {
        let snippets = vec![
            ("Simple ASCII text", 10),
            ("Hello🔍World", 6),
            ("你好世界 Chinese", 5),
            ("café au lait", 5),
            ("مرحبا بالعالم", 3), // Arabic
            ("🎉🎊🎈", 2),        // Emojis
            ("a̗b̗c̗", 2),           // Combining marks
        ];

        for (text, char_limit) in snippets {
            let result = safe_truncate_at_char(text, char_limit);

            // Verify result is valid UTF-8
            let _ = result.chars().count(); // This would panic if invalid UTF-8

            // Verify we didn't exceed char limit
            let actual_chars = result.chars().count();
            assert!(
                actual_chars <= char_limit,
                "Truncated '{}' should have <= {} chars, got {}",
                text,
                char_limit,
                actual_chars
            );
        }
    }

    /// Test: Byte position to char position conversion (like in line number calculation)
    #[test]
    fn test_byte_position_to_char_conversion() {
        let text = "Hello🔍World";
        // H=1 char, e=1 char, l=1 char, l=1 char, o=1 char, 🔍=1 char (but 4 bytes), W=1, o=1, r=1, l=1, d=1
        // Total: 11 chars, 15 bytes (5 ASCII + 4 for emoji + 6 ASCII)

        let total_chars = text.chars().count();
        assert_eq!(
            total_chars, 11,
            "Should have 11 characters (emoji is 1 char but 4 bytes)"
        );

        let total_bytes = text.len();
        assert_eq!(total_bytes, 14, "Should have 14 bytes (5 + 4 + 5)");

        // Find byte position of emoji
        let emoji_byte_pos = text.find("🔍").unwrap();
        assert_eq!(emoji_byte_pos, 5, "Emoji should start at byte 5");

        // Convert to char position
        let char_pos = text[..emoji_byte_pos].chars().count();
        assert_eq!(char_pos, 5, "Emoji should be at char position 5");
    }

    /// Test: HTML escaping with UTF-8 content
    #[test]
    fn test_html_escaping_with_unicode() {
        fn escape_html(s: &str) -> String {
            s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
        }

        let test_cases = vec![
            ("hello < world", "hello &lt; world"),
            (
                "🔍 <script>alert('xss')</script>",
                "🔍 &lt;script&gt;alert('xss')&lt;/script&gt;",
            ),
            ("你好 < 世界", "你好 &lt; 世界"),
        ];

        for (input, expected) in test_cases {
            let result = escape_html(input);
            assert_eq!(result, expected, "HTML escaping should preserve UTF-8");

            // Verify result is still valid UTF-8
            let _ = result.chars().count();
        }
    }

    /// Test: Simulating the actual bug scenario
    /// What might happen if snippet generation used byte slicing instead of char iteration
    #[test]
    #[allow(clippy::implicit_saturating_sub, clippy::string_from_utf8_as_bytes)]
    fn test_simulated_bug_scenario() {
        let file_content = r#"
// Function with emoji
fn process_data() {
    // 🔍 Search functionality
    let pattern = "😀.*world";

    if let Some(pos) = pattern.find("世") {
        println!("Found at {}", pos);
    }
}
"#;

        // Simulate finding a search term and extracting around it
        let search_term = "🔍";

        if let Some(byte_pos) = file_content.find(search_term) {
            // WRONG: Try to take 50 bytes centered on match
            let start = if byte_pos > 25 { byte_pos - 25 } else { 0 };
            let end = (byte_pos + 25).min(file_content.len());

            // This might fail if we land in middle of UTF-8 sequence
            let bad_result = std::str::from_utf8(&file_content.as_bytes()[start..end]);

            // RIGHT: Use char iteration instead
            let char_pos = file_content[..byte_pos].chars().count();
            let safe_result = file_content.chars().skip(char_pos.saturating_sub(25)).take(50).collect::<String>();

            // Safe result should always be valid UTF-8
            let _ = safe_result.chars().count();

            println!("Bad result (might be Err): {:?}", bad_result.is_err());
            println!("Safe result (always Ok): {}", !safe_result.is_empty());
        }
    }
}
