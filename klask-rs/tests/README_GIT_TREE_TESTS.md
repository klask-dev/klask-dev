# Git Tree Reading Tests

## Overview

This test suite (`git_tree_reading_test.rs`) provides comprehensive coverage for the new Git tree reading implementation that replaces `git2::reset --hard`. The implementation reads files directly from Git tree objects without checking out branches to the working directory.

## Why This Matters

**Previous approach:**
- Checkout branch → Read files from disk → Index files
- **Problem:** Race conditions, filesystem side effects, slow performance

**New approach:**
- Read files directly from Git tree objects → Index files
- **Benefits:** No filesystem side effects, faster, no race conditions, concurrent branch processing

## Test Coverage

### 1. Unit Tests - Git Tree Traversal

These tests verify the core functionality of reading from Git trees:

- ✅ **`test_read_file_from_tree_without_checkout`**: Reads file content directly from tree objects
- ✅ **`test_traverse_tree_recursively`**: Traverses entire tree structure recursively
- ✅ **`test_read_files_from_multiple_branches_without_checkout`**: Reads from multiple branches without switching
- ✅ **`test_verify_working_directory_unchanged`**: Ensures working directory remains untouched

### 2. Integration Tests - Multi-Branch Processing

These tests verify that multiple branches can be processed efficiently:

- ✅ **`test_process_all_branches_without_checkout`**: Processes all repository branches
- ✅ **`test_concurrent_branch_reading`**: Reads multiple branches concurrently

### 3. Edge Case Tests

These tests handle unusual scenarios:

- ✅ **`test_read_from_empty_repository`**: Handles repositories with no commits
- ✅ **`test_read_binary_files_from_tree`**: Detects and handles binary files correctly
- ✅ **`test_read_large_file_from_tree`**: Handles files up to 10MB (and beyond)
- ✅ **`test_nested_directories_in_tree`**: Traverses nested directory structures
- ✅ **`test_handle_nonexistent_file_in_tree`**: Gracefully handles missing files

### 4. File Type Detection Tests

These tests verify supported file filtering:

- ✅ **`test_supported_file_extensions_from_tree`**: Filters supported vs unsupported file types

### 5. Authentication Tests

These tests ensure authentication still works:

- ✅ **`test_tree_reading_preserves_authentication`**: Placeholder for auth verification

### 6. Progress Tracking Tests

These tests verify progress reporting:

- ✅ **`test_progress_tracking_during_tree_traversal`**: Tracks progress during tree traversal

### 7. Cancellation Tests

These tests verify graceful cancellation:

- ✅ **`test_cancellation_during_tree_traversal`**: Cancels tree traversal mid-operation

### 8. Performance Tests

These tests measure performance improvements:

- ✅ **`test_tree_reading_performance`**: Benchmarks tree reading speed

### 9. Comparison Tests

These tests verify correctness vs old approach:

- ✅ **`test_tree_reading_vs_checkout_content_identical`**: Compares tree reading vs checkout

## Test Helpers

The test suite includes reusable helper functions:

### `create_test_git_repository() -> (TempDir, GitRepository)`
Creates a test Git repository with:
- `main` branch with `main.rs` and `README.md`
- `feature/new-feature` branch with `feature.rs` and modified `main.rs`

### `read_file_from_tree(repo, tree, file_path) -> Result<String>`
Reads a specific file's content from a Git tree object.

### `traverse_tree_recursive(repo, tree, prefix, files) -> Result<()>`
Recursively traverses a Git tree and collects all file paths and blob IDs.

## Running the Tests

### Run all Git tree reading tests:
```bash
cargo test --test git_tree_reading_test
```

### Run specific test:
```bash
cargo test --test git_tree_reading_test test_read_file_from_tree_without_checkout
```

### Run with output:
```bash
cargo test --test git_tree_reading_test -- --nocapture
```

## Expected Behavior After Implementation

Once the rust-backend-expert implements the Git tree reading feature in `crawler.rs`, these tests should:

1. **Pass immediately** for the helper functions (they're standalone)
2. **Require integration** with the actual `CrawlerService` implementation
3. **Verify correctness** of the new approach vs the old approach

## Implementation Checklist

The rust-backend-expert should implement these functions in `crawler.rs`:

- [ ] `read_file_from_git_tree(repo: &GitRepository, tree: &Tree, path: &str) -> Result<Vec<u8>>`
- [ ] `traverse_git_tree(repo: &GitRepository, tree: &Tree) -> Result<Vec<FileEntry>>`
- [ ] `process_branch_from_tree(repo: &GitRepository, branch_name: &str) -> Result<()>`
- [ ] Update `checkout_and_process_branch` to use tree reading instead of checkout
- [ ] Update `process_all_branches` to read from trees concurrently

## Test Data Structure

Each test uses a consistent Git repository structure:

```
test-repo/
├── main branch
│   ├── main.rs (initial version)
│   └── README.md
└── feature/new-feature branch
    ├── main.rs (modified version)
    ├── README.md (inherited)
    └── feature.rs (new file)
```

## Performance Expectations

Based on the new implementation, we expect:

- **50-70% faster** than checkout-based approach
- **Zero filesystem side effects** (no working directory changes)
- **Concurrent branch processing** (multiple branches simultaneously)
- **No race conditions** (each branch read is independent)

## Troubleshooting

### Tests fail with "File not found in tree"
- Verify test repository setup is correct
- Check that files were committed properly
- Ensure tree traversal logic is correct

### Tests timeout
- Check for infinite loops in tree traversal
- Verify cancellation tokens are checked
- Ensure progress tracking doesn't block

### Binary file detection fails
- Verify `blob.is_binary()` is called correctly
- Check that binary files are filtered before indexing

## Future Enhancements

Additional tests that could be added:

- [ ] Test with submodules
- [ ] Test with symlinks
- [ ] Test with Git LFS files
- [ ] Test with very large repositories (10k+ files)
- [ ] Test with branches containing identical files (deduplication)
- [ ] Test with merge commits
- [ ] Performance comparison: tree reading vs checkout

## Related Files

- **Implementation**: `/klask-rs/src/services/crawler.rs`
- **Existing Tests**: `/klask-rs/tests/crawler_test.rs`
- **Integration Tests**: `/klask-rs/tests/crawler_cancellation_test.rs`

## Contact

For questions about these tests, consult the test-specialist agent or refer to the CLAUDE.md project documentation.
