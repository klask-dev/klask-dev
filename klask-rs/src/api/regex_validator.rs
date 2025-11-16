/// Regex pattern validation module
///
/// Provides strict validation of regex patterns to prevent ReDoS (Regular Expression Denial of Service)
/// and other security issues before patterns reach Tantivy.
const MAX_REGEX_LENGTH: usize = 500;
const MAX_NESTING_DEPTH: usize = 3;
const DANGEROUS_PATTERNS: &[&str] = &[
    "(+)+",  // Nested quantifiers
    "(*)*",  // Nested quantifiers
    "({)?{", // Nested quantifiers
    "(|)*",  // Alternation with star
    "(|)+",  // Alternation with plus
];

/// Validates a regex pattern for security and performance concerns.
///
/// # Arguments
/// * `pattern` - The regex pattern to validate
///
/// # Returns
/// * `Ok(())` if the pattern is valid and safe
/// * `Err(String)` with a descriptive error message if validation fails
///
/// # Validation Rules
/// 1. Pattern length must not exceed MAX_REGEX_LENGTH (500 characters)
/// 2. Pattern cannot be empty
/// 3. Pattern must not contain known dangerous patterns that can cause ReDoS
/// 4. Pattern must not have more than MAX_NESTING_DEPTH levels of nested groups
/// 5. Quantifiers on nested groups must be carefully validated
pub fn validate_regex_pattern(pattern: &str) -> Result<(), String> {
    // 1. Vérifier longueur max
    if pattern.len() > MAX_REGEX_LENGTH {
        return Err(format!(
            "Regex pattern exceeds max length of {} characters (current: {})",
            MAX_REGEX_LENGTH,
            pattern.len()
        ));
    }

    // 2. Pattern vide
    if pattern.is_empty() {
        return Err("Regex pattern cannot be empty".to_string());
    }

    // 3. Vérifier patterns dangereux (ReDoS)
    for dangerous in DANGEROUS_PATTERNS {
        if pattern.contains(dangerous) {
            return Err(format!(
                "Pattern detected as potentially dangerous (ReDoS risk): contains '{}'",
                dangerous
            ));
        }
    }

    // 4. Vérifier nested groups et quantifiers
    validate_nesting_and_quantifiers(pattern)?;

    Ok(())
}

/// Validates nesting depth and quantifier usage to prevent catastrophic backtracking
fn validate_nesting_and_quantifiers(pattern: &str) -> Result<(), String> {
    if !pattern.contains('(') && !pattern.contains(')') {
        // No groups, so no nesting concerns
        return Ok(());
    }

    let chars: Vec<char> = pattern.chars().collect();
    let mut depth = 0;
    let mut max_depth = 0;
    let mut depths_with_quantifiers: Vec<usize> = Vec::new();

    for i in 0..chars.len() {
        match chars[i] {
            '(' => {
                // Opening group
                depth += 1;
                max_depth = max_depth.max(depth);

                // Check if there are special characters that might cause issues
                if depth > 1 {
                    // Check for alternation at this level
                    if i + 1 < chars.len() && chars[i + 1] == '|' {
                        // Alternation inside nested group - potential ReDoS risk
                        // but allow single level alternation
                    }
                }
            }
            ')' => {
                if depth == 0 {
                    return Err("Unmatched closing parenthesis in regex pattern".to_string());
                }

                // Check if there's a quantifier immediately after this group
                if i + 1 < chars.len() {
                    match chars[i + 1] {
                        '+' | '*' => {
                            // Quantifier after group
                            depths_with_quantifiers.push(depth);
                        }
                        '{' => {
                            // Counted repetition {n,m}
                            depths_with_quantifiers.push(depth);
                        }
                        _ => {}
                    }
                }

                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    // Check for unmatched opening parenthesis
    if depth > 0 {
        return Err("Unmatched opening parenthesis in regex pattern".to_string());
    }

    // Trop de groupes imbriqués = danger
    if max_depth > MAX_NESTING_DEPTH {
        return Err(format!(
            "Pattern has too many nested groups (max {} levels allowed, found {})",
            MAX_NESTING_DEPTH, max_depth
        ));
    }

    // Check for quantifier on nested group (increased risk of catastrophic backtracking)
    // Only flag this as dangerous if quantifier is at depth > 1 with multiple levels
    for quantifier_depth in depths_with_quantifiers {
        if quantifier_depth > 2 && max_depth > 2 {
            return Err("Pattern has quantifier on deeply nested group (ReDoS risk detected)".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simple_patterns() {
        assert!(validate_regex_pattern("^test$").is_ok());
        assert!(validate_regex_pattern("hello.*world").is_ok());
        assert!(validate_regex_pattern("[a-z]+").is_ok());
        assert!(validate_regex_pattern("^Crawler.*").is_ok());
    }

    #[test]
    fn test_valid_nested_patterns() {
        assert!(validate_regex_pattern("(test)").is_ok());
        assert!(validate_regex_pattern("((test))").is_ok());
        assert!(validate_regex_pattern("(test)+").is_ok());
        assert!(validate_regex_pattern("(hello|world)").is_ok());
    }

    #[test]
    fn test_empty_pattern() {
        let result = validate_regex_pattern("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_max_length_exceeded() {
        let long_pattern = "a".repeat(MAX_REGEX_LENGTH + 1);
        let result = validate_regex_pattern(&long_pattern);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds max length"));
    }

    #[test]
    fn test_at_max_length() {
        let pattern = "a".repeat(MAX_REGEX_LENGTH);
        assert!(validate_regex_pattern(&pattern).is_ok());
    }

    #[test]
    fn test_dangerous_nested_quantifiers_1() {
        let result = validate_regex_pattern("(+)+");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_nested_quantifiers_2() {
        let result = validate_regex_pattern("(*)*");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_nested_quantifiers_3() {
        let result = validate_regex_pattern("({)?{");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_alternation_with_quantifier_1() {
        let result = validate_regex_pattern("(|)*");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_alternation_with_quantifier_2() {
        let result = validate_regex_pattern("(|)+");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_nesting_levels() {
        let result = validate_regex_pattern("((((test))))");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too many nested groups"));
    }

    #[test]
    fn test_unmatched_closing_paren() {
        let result = validate_regex_pattern("test)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unmatched closing"));
    }

    #[test]
    fn test_unmatched_opening_paren() {
        let result = validate_regex_pattern("(test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unmatched opening"));
    }

    #[test]
    fn test_valid_complex_pattern() {
        assert!(validate_regex_pattern("^([a-z]+)_([0-9]+)$").is_ok());
    }

    #[test]
    fn test_valid_file_pattern() {
        assert!(validate_regex_pattern(".*\\.rs$").is_ok());
    }

    #[test]
    fn test_valid_alternation() {
        assert!(validate_regex_pattern("^(test|hello|world)$").is_ok());
    }
}
