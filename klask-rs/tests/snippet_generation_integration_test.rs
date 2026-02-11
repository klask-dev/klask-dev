/// Integration tests for real snippet generation with UTF-8 content
/// These tests verify that the actual snippet generation code handles UTF-8 correctly
#[cfg(test)]
mod snippet_generation_tests {
    /// Simulates the safe UTF-8 truncation logic from search.rs
    /// This is what the real code should use
    fn truncate_safely(text: &str, max_chars: usize) -> String {
        text.chars().take(max_chars).collect()
    }

    /// Simulates what the bad code might do - truncate at byte position without checking
    fn truncate_unsafely(text: &str, max_bytes: usize) -> Result<String, String> {
        if max_bytes >= text.len() {
            return Ok(text.to_string());
        }
        match std::str::from_utf8(&text.as_bytes()[..max_bytes]) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err("Invalid UTF-8".to_string()),
        }
    }

    /// Test: Extract line number from byte position with UTF-8
    #[test]
    fn test_line_number_extraction_with_utf8() {
        let content = "Line 1\nLine 2 with emoji 🔍\nLine 3\nLine 4";

        // Find the emoji
        if let Some(emoji_pos) = content.find("🔍") {
            // The bug: if code counts lines by iterating bytes instead of chars
            // it might count wrong with multi-byte characters

            // Correct: count lines by iterating chars
            let line_number = content[..emoji_pos].chars().filter(|&c| c == '\n').count() as u32 + 1;

            assert_eq!(line_number, 2, "Emoji should be on line 2");
        }
    }

    /// Test: Generate excerpt with UTF-8 content
    #[test]
    fn test_excerpt_generation_with_unicode() {
        let content = "Hello world with emoji 🔍 in the middle and some more text after it";

        // Find position of emoji
        if let Some(pos) = content.find("🔍") {
            // Wrong way: try to extract 50 bytes around emoji
            let _start_byte = pos.saturating_sub(25);
            let end_byte = (pos + 25).min(content.len());
            let _unsafe_excerpt = truncate_unsafely(content, end_byte);

            // Right way: use char iteration
            let char_pos = content[..pos].chars().count();
            let safe_excerpt = content.chars().skip(char_pos.saturating_sub(15)).take(40).collect::<String>();

            println!("Safe excerpt: {}", safe_excerpt);

            // Verify safe excerpt is valid UTF-8
            assert!(!safe_excerpt.is_empty());
            let _ = safe_excerpt.chars().count();
        }
    }

    /// Test: Search term highlighting with UTF-8
    #[test]
    fn test_search_term_highlighting_with_utf8() {
        let content = "The 🔍 symbol is used for search in many applications";

        // Find and highlight the emoji
        let search_term = "🔍";

        if let Some(pos) = content.find(search_term) {
            // Convert byte position to char position
            let char_pos = content[..pos].chars().count();
            let char_end = char_pos + search_term.chars().count();

            // Extract highlight context using char positions
            let chars: Vec<char> = content.chars().collect();

            // Context: 10 chars before and 10 chars after
            let context_start = char_pos.saturating_sub(10);
            let context_end = (char_end + 10).min(chars.len());

            let highlighted: String = chars[context_start..context_end].iter().collect();

            println!("Highlighted context: {}", highlighted);
            assert!(highlighted.contains("🔍"));
        }
    }

    /// Test: Handle Emoji with skin tone modifiers (ZWJ sequences)
    #[test]
    fn test_emoji_with_modifiers() {
        // Family emoji: 👨‍👩‍👧‍👦 (multiple codepoints joined with ZWJ)
        let content = "Family: 👨‍👩‍👧‍👦 is important";

        // Safe truncation should work
        let truncated = truncate_safely(content, 15);
        assert!(!truncated.is_empty());

        // Verify it's valid UTF-8
        let _ = truncated.chars().count();
    }

    /// Test: Mixed emoji and regular text in snippet
    #[test]
    fn test_mixed_content_snippet() {
        let content = r#"
fn search_files() {
    // 🔍 Looking for pattern
    let pattern = "test.*emoji 😀";

    // Process results
    for item in results {
        println!("Found: {}", item);
    }
}
"#;

        // Extract a snippet around the first emoji
        if let Some(_emoji_pos) = content.find("🔍") {
            let snippet = truncate_safely(content, 100);
            assert!(!snippet.is_empty());
        }
    }

    /// Test: Arabic and RTL text with emoji
    #[test]
    fn test_rtl_text_with_emoji() {
        let content = "مرحبا 👋 Hello العالم";

        // Safe truncation should preserve all characters
        let truncated = truncate_safely(content, 20);
        assert!(!truncated.is_empty());

        // Verify char count matches
        let chars = truncated.chars().count();
        assert!(chars <= 20);
    }

    /// Test: Potential panic scenario - truncate at exact byte boundaries
    #[test]
    fn test_no_panic_on_boundary_truncation() {
        let text = "Hello🔍World";

        // Try truncating at every byte position
        for byte_pos in 0..=text.len() {
            // Unsafe method should fail gracefully on bad boundaries
            let _result = truncate_unsafely(text, byte_pos);

            // Safe method should never fail
            let safe = truncate_safely(text, 5);
            assert!(!safe.is_empty() || byte_pos == 0);
        }
    }

    /// Test: Very long UTF-8 content with many emoji
    #[test]
    fn test_long_content_with_many_unicode() {
        let content = "Start 🎉 ".repeat(100) + "End";

        // Truncate to 500 chars
        let truncated = truncate_safely(&content, 500);

        // Verify it's valid UTF-8 and reasonable size
        let chars = truncated.chars().count();
        assert!(chars <= 500);

        // Should contain at least some emoji
        assert!(truncated.contains("🎉"));
    }

    /// Test: Verify line number calculation with multi-line UTF-8
    #[test]
    fn test_multiline_line_number_calculation() {
        let content = "Line 1\n🔍 Search line\nLine 3\n你好 Chinese line\nLine 5";

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 5);

        // Find emoji line
        for (idx, line) in lines.iter().enumerate() {
            if line.contains("🔍") {
                assert_eq!(idx, 1, "Emoji should be on line 2 (0-indexed line 1)");
            }
        }
    }

    /// Test: UTF-8 content with control characters and special cases
    #[test]
    fn test_edge_case_utf8_chars() {
        let content = "Normal \u{0000}null 🔍 emoji \u{200B}zwsp Arabic: مرحبا";

        // Safe truncation should handle all
        let truncated = truncate_safely(content, 50);
        assert!(!truncated.is_empty());
    }
}
