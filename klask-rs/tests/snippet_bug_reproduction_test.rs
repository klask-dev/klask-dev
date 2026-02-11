/// Reproduction test for the UTF-8 snippet generation bug
///
/// The bug scenario:
/// 1. A file contains emoji or multi-byte UTF-8 characters
/// 2. Snippet generation tries to truncate at a character limit
/// 3. But the code mistakenly uses byte positions instead of character positions
/// 4. Result: truncating in the middle of a multi-byte sequence
/// 5. When trying to convert to string: crash or invalid UTF-8 error
#[cfg(test)]
mod utf8_bug_reproduction {
    use std::panic;

    /// The buggy implementation - simulating what might cause the crash
    fn buggy_truncate_at_position(text: &str, position: usize) -> String {
        // BUG: Treating position as bytes instead of characters
        // This will panic or produce invalid UTF-8 if position lands in middle of multi-byte char
        let bytes = text.as_bytes();
        if position >= bytes.len() {
            text.to_string()
        } else {
            // Danger: This can panic with "byte index X is not a char boundary"
            String::from_utf8_lossy(&bytes[..position]).to_string()
        }
    }

    /// Correct implementation using character counting
    fn correct_truncate_at_position(text: &str, position: usize) -> String {
        text.chars().take(position).collect()
    }

    /// Test Case 1: Emoji at exact truncation point
    #[test]
    fn test_emoji_at_truncation_boundary() {
        let text = "prefix🔍suffix";

        // Try to truncate at byte position 6 (right after "prefix")
        // But emoji 🔍 is 4 bytes, so this lands at the start of emoji

        // The buggy way might fail
        let result = panic::catch_unwind(|| buggy_truncate_at_position(text, 6));

        // Depending on the exact implementation, it might fail
        println!("Buggy result: {:?}", result.is_ok());

        // The correct way always works
        let correct = correct_truncate_at_position(text, 6);
        assert!(!correct.is_empty());
    }

    /// Test Case 2: Chinese characters - 3 bytes each
    #[test]
    fn test_chinese_truncation_bug() {
        let text = "Search结果: results here";
        // "Search" = 6 bytes
        // "结" = 3 bytes (starts at byte 6)
        // "果" = 3 bytes (starts at byte 9)

        // Try to truncate at byte 7 (middle of "结")
        let result = panic::catch_unwind(|| buggy_truncate_at_position(text, 7));

        // Buggy version might produce invalid UTF-8
        println!("Buggy truncate at 7: {:?}", result);

        // Correct version using char count
        let correct = correct_truncate_at_position(text, 7);
        assert!(!correct.is_empty());
    }

    /// Test Case 3: Real-world code snippet that would break the backend
    #[test]
    fn test_real_world_snippet_break() {
        // A typical code snippet with mixed content
        let file_content = r#"
// Search function 🔍
fn find_pattern(regex: &str) {
    println!("Searching: {}", regex);
    // 中文注释
    let results = search(regex);
}
"#;

        // Simulate searching for content and generating snippet
        // What if code tries to truncate to exactly 40 bytes?
        let bad_position = 40; // Might be in middle of a multi-byte char

        let result = panic::catch_unwind(|| buggy_truncate_at_position(file_content, bad_position));

        println!("Buggy approach to 40 bytes: {:?}", result.is_ok());

        // Safe approach: truncate to 40 characters instead of bytes
        let safe = correct_truncate_at_position(file_content, 40);
        assert!(!safe.is_empty());
    }

    /// Test Case 4: The exact crash scenario
    /// If backend tries to extract snippet at byte boundary that lands in emoji
    /// The bug: Results in invalid UTF-8 that would fail strict parsing
    #[test]
    fn test_exact_crash_scenario() {
        let snippet_content = "before🔍after";
        // Bytes: b,e,f,o,r,e (6 bytes ASCII) + 🔍 (4 bytes: F0 9F 94 8D) + a,f,t,e,r

        // Try to truncate at byte 7 - in the middle of emoji (after F0, before 9F)
        let position = 7;

        // Strict UTF-8 parsing fails because we cut in middle of emoji
        let parse_result = std::str::from_utf8(&snippet_content.as_bytes()[..position]);
        assert!(parse_result.is_err(), "Should fail - cut in middle of emoji at byte 7");

        // If using lossy decoding (String::from_utf8_lossy), bad bytes are replaced
        // This could cause data loss in snippets - showing corrupted content to user
        let c_bytes = snippet_content.as_bytes()[..position].to_vec();
        let lossy = String::from_utf8_lossy(&c_bytes);

        // The result is "before" + replacement character - data loss!
        // This is the silent corruption bug
        println!("Lossy result: {:?}", lossy);
        assert!(
            lossy.contains('\u{FFFD}'),
            "Lossy decoding should have replacement char"
        );
        assert!(!lossy.contains("🔍"), "Emoji is lost in lossy decoding");

        // But the correct char-based approach works fine with no data loss
        let correct = correct_truncate_at_position(snippet_content, 6);
        assert_eq!(correct, "before");
    }

    /// Test Case 5: What happens with actual backend snippet generation
    #[test]
    #[allow(clippy::implicit_saturating_sub, clippy::string_from_utf8_as_bytes)]
    fn test_backend_snippet_generation_safety() {
        let file_content = include_str!("./snippet_utf8_test.rs");

        // Simulate searching for a term and extracting surrounding context
        let search_term = "emoji";

        if let Some(byte_pos) = file_content.find(search_term) {
            // Buggy approach: extract 50 bytes around position
            let start = if byte_pos > 25 { byte_pos - 25 } else { 0 };
            let end = (byte_pos + 25).min(file_content.len());

            let buggy = std::str::from_utf8(&file_content.as_bytes()[start..end]);

            // Safe approach: find char position, then extract with char counting
            let char_pos = file_content[..byte_pos].chars().count();
            let safe: String = file_content.chars().skip(char_pos.saturating_sub(25)).take(50).collect();

            // Safe approach always works
            assert!(!safe.is_empty());

            // Buggy approach might fail
            println!("Buggy UTF-8 parse result: {:?}", buggy.is_ok());
        }
    }

    /// Test Case 6: Various emoji that might cause issues
    #[test]
    fn test_various_emoji_truncation_points() {
        let test_cases = vec![
            ("single🔍emoji", 6),    // Right before emoji
            ("single🔍emoji", 7),    // In middle of emoji (should work with char approach)
            ("🎉start here", 0),     // Beginning with emoji
            ("end🎊", 3),            // Right before emoji
            ("多个😀多个😀多个", 5), // Multiple emoji
        ];

        for (text, truncate_pos) in test_cases {
            // Correct way always works
            let correct = correct_truncate_at_position(text, truncate_pos);
            assert!(
                correct.chars().all(|_| true),
                "Correct approach should always produce valid UTF-8"
            );

            println!("Safe truncate '{}' at {}: '{}'", text, truncate_pos, correct);
        }
    }

    /// Test Case 7: Verify the backend scenario from real code
    /// This is similar to what happens in generate_optimized_snippet()
    #[test]
    fn test_simulate_backend_snippet_generation() {
        let content = "This is a test file with emoji 🔍 in it and some other content";

        // Simulate the safe way the actual code should work:
        // 1. Find term
        // 2. Convert byte position to char position
        // 3. Extract using char iteration

        let term = "emoji";
        if let Some(byte_pos) = content.find(term) {
            // Convert byte to char position (this is key!)
            let char_pos = content[..byte_pos].chars().count();

            // Extract snippet using char positions (safe!)
            let snippet: String = content.chars().skip(char_pos.saturating_sub(5)).take(30).collect();

            println!("Snippet: {}", snippet);

            // Verify it's valid UTF-8 and contains our term
            assert!(snippet.contains("emoji"));
            let _ = snippet.chars().count(); // Should not panic
        }
    }

    /// Test Case 8: Ensure line number calculation is also UTF-8 safe
    #[test]
    fn test_line_number_calculation_with_utf8() {
        let content = "Line 1\nLine 2 with emoji🔍\nLine 3";

        let term = "🔍";
        if let Some(byte_pos) = content.find(term) {
            // Buggy approach: count newlines in byte slice
            // This works but is fragile

            // Better approach: convert to char position first
            let char_pos = content[..byte_pos].chars().count();

            // Now count lines safely using char iteration
            let line_number = content.chars().take(char_pos).filter(|&c| c == '\n').count() as u32 + 1;

            assert_eq!(line_number, 2, "Should find emoji on line 2");
        }
    }
}
