/// Parser configuration constants
///
/// This module contains all constants used by the parser system for content detection
/// and file analysis. Centralizing these constants makes it easier to adjust behavior
/// and understand the thresholds used throughout the parsing system.
/// Maximum number of bytes to analyze when detecting if content is text or binary
///
/// The content detector analyzes the first N bytes of a file to determine if it's
/// text or binary. This value is a balance between accuracy and performance.
/// - Too small: May incorrectly classify files (e.g., small binary header followed by text)
/// - Too large: Wasted analysis on large files
/// - 256 bytes: Good balance for most file types
pub const ANALYSIS_SIZE: usize = 256;

/// Confidence threshold (percentage) for text detection
///
/// When analyzing file content, we calculate a "text confidence" percentage based on
/// character analysis. Files with text_confidence >= this threshold are considered text files.
/// - Below 70%: Likely binary (many control chars or non-ASCII patterns)
/// - 70-90%: Likely text (mostly printable ASCII and common whitespace)
/// - 90%+: Very confident text (clean UTF-8 or ASCII)
pub const TEXT_CONFIDENCE_THRESHOLD: u8 = 70;

// Helper macro for compile-time assertions
#[allow(unused_macros)]
macro_rules! const_assert {
    ($cond:expr, $msg:expr) => {
        const _: () = assert!($cond, $msg);
    };
}

// Compile-time validation of constants
const _: () = {
    const_assert!(ANALYSIS_SIZE > 0, "ANALYSIS_SIZE must be positive");
    const_assert!(ANALYSIS_SIZE < 10000, "ANALYSIS_SIZE should not be too large");
    const_assert!(
        TEXT_CONFIDENCE_THRESHOLD <= 100,
        "TEXT_CONFIDENCE_THRESHOLD must be 0-100"
    );
    const_assert!(
        TEXT_CONFIDENCE_THRESHOLD >= 50,
        "TEXT_CONFIDENCE_THRESHOLD should be at least 50"
    );
};
