# Custom Code Tokenizer Implementation for Klask

## Overview

This document describes the implementation of a custom tokenizer for Klask's code search engine. The tokenizer is designed to handle code-specific tokenization patterns including camelCase splitting, underscore/hyphen preservation, and acronym handling.

## Files Modified

### 1. `/workspace/klask-rs/src/services/tokenizer.rs` (NEW)
Custom tokenizer module implementing code-aware token splitting.

**Key Components:**
- `CodeTokenizer` struct: Implements the Tantivy `Tokenizer` trait
- `CodeTokenStream` struct: Implements the Tantivy `TokenStream` trait
- `tokenize_code()` function: Core tokenization logic

**Features:**
- Splits on camelCase transitions: `camelCase` → `["camel", "case"]`
- Handles acronyms intelligently: `getHTTPResponse` → `["get", "http", "response"]`
- Preserves underscores and hyphens: `my_func` → `["my_func"]`, `snake-case` → `["snake-case"]`
- Lowercases all tokens for case-insensitive search
- Includes comprehensive test suite with 12 test cases

**Tokenization Algorithm:**
1. Iterate through characters tracking position and case transitions
2. Split when transitioning from lowercase to uppercase
3. Split before the last uppercase letter when followed by lowercase (handles acronyms)
4. Never split on underscores or hyphens (preserve them in tokens)
5. Lowercase all output tokens

### 2. `/workspace/klask-rs/src/services/mod.rs`
Added module export for the new tokenizer.

**Changes:**
```rust
pub mod tokenizer;  // Added this line
```

### 3. `/workspace/klask-rs/src/services/search.rs`
Integrated the custom tokenizer into the search service.

**Changes Made:**

a. **Import the tokenizer:**
```rust
use crate::services::tokenizer::CodeTokenizer;
```

b. **Register tokenizer in `SearchService::new()`:**
```rust
// Register custom tokenizer for code search
// This must be done immediately after opening the index and before any document operations
{
    let mut tokenizer_manager = index.tokenizers();
    tokenizer_manager.register("code_tokenizer", CodeTokenizer::new());
    debug!("Registered custom code_tokenizer for code-aware search");
}
```

c. **Update schema in `build_schema()` method:**
```rust
// Configure custom code tokenizer for code-specific text fields
let code_text_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("code_tokenizer")
            .set_index_option(IndexRecordOption::WithFreqs)
    )
    .set_stored();

schema_builder.add_text_field("file_name", code_text_options.clone());
schema_builder.add_text_field("file_path", code_text_options.clone());
schema_builder.add_text_field("content", code_text_options);
```

## Tokenization Examples

### CamelCase
Input: `camelCase`
Output: `["camel", "case"]`

### PascalCase
Input: `PascalCase`
Output: `["pascal", "case"]`

### Acronyms
Input: `HTMLParser`
Output: `["html", "parser"]`

Input: `getHTTPResponse`
Output: `["get", "http", "response"]`

### Snake Case (Preserved)
Input: `snake_case`
Output: `["snake_case"]`

Input: `NGINX_URL`
Output: `["nginx_url"]`

### Hyphenated (Preserved)
Input: `my-function-name`
Output: `["my-function-name"]`

### Mixed Snake and Camel
Input: `my_functionName`
Output: `["my_function", "name"]`

### Complex Examples
Input: `base64Encode`
Output: `["base64", "encode"]`

Input: `HTTPSConnection`
Output: `["https", "connection"]`

## Search Benefits

With this tokenizer, searches will now match:

1. **Acronym Search**: Searching for "HTTP" will match `getHTTPResponse`, `HTTPSConnection`, etc.
2. **Camel Case Components**: Searching for "case" will match `camelCase`, `PascalCase`, etc.
3. **Underscore Preservation**: Searching for `my_func` will match `my_func` exactly (no spurious matches)
4. **Case Insensitivity**: All searches are case-insensitive by default (e.g., "http" matches "HTTP", "Http", etc.)

## Test Coverage

The tokenizer includes 12 test cases covering:
- Basic camelCase and PascalCase
- Acronym handling
- Snake case preservation
- Hyphenated names
- Mixed naming conventions
- Edge cases (single word, empty string, single char)
- Numbers in identifiers

### Running Tests

Once the build environment is set up, run:
```bash
cd /workspace/klask-rs
cargo test services::tokenizer
```

## Implementation Notes

1. **Stateless Design**: The `CodeTokenizer` is stateless and can be reused safely
2. **Memory Efficient**: Uses a single pass through the input text
3. **Tantivy Compatible**: Implements standard Tantivy tokenizer traits
4. **Logging**: Logs tokenizer registration at DEBUG level for troubleshooting
5. **Index Rebuild**: Existing indexes need to be rebuilt for the tokenizer to take effect on previously indexed files

## Integration with Search Workflow

1. When `SearchService::new()` is called, it opens/creates the Tantivy index
2. The custom tokenizer is immediately registered with the index
3. When documents are indexed via `upsert_file()`, the text fields use the custom tokenizer
4. When searches are performed, the same tokenizer is used for query parsing

## Future Enhancements

Possible improvements for future iterations:
1. Configurable tokenizer behavior (e.g., preserving numbers separately)
2. Language-specific variants (Java, Python, Rust naming conventions)
3. Custom stopword filtering
4. Phonetic matching for searching similar-sounding identifiers

## Backwards Compatibility

The schema was updated to use the custom tokenizer for searchable text fields:
- `file_name`: Now uses custom tokenizer
- `file_path`: Now uses custom tokenizer
- `content`: Now uses custom tokenizer

The raw versions (`file_name_raw`, `file_path_raw`) continue using the "raw" tokenizer for case-sensitive regex matching.

Filter fields (`repository`, `project`, `version`, `extension`) remain unchanged as they require exact matching.

## Building and Testing

The implementation is complete and ready for testing once the Rust build environment is properly configured:

```bash
cd /workspace/klask-rs
cargo build
cargo test
cargo clippy -- -D warnings
```

All code follows Klask's existing patterns and error handling conventions.
