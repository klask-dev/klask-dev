# Git Tree Implementation Guide

## For the Rust Backend Expert

This guide provides detailed information for implementing Git tree reading to replace `git2::reset --hard` in the crawler service.

## Overview

**Goal**: Read files directly from Git tree objects instead of checking out branches to the working directory.

**Current implementation** (problematic):
```rust
// In crawler.rs
async fn checkout_and_process_branch(...) {
    // 1. Checkout branch (modifies working directory)
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

    // 2. Read files from disk
    for entry in WalkDir::new(&repo_path) {
        let content = std::fs::read_to_string(entry.path())?;
        // ... index content
    }
}
```

**New implementation** (desired):
```rust
async fn process_branch_from_tree(...) {
    // 1. Get tree object for branch (no checkout!)
    let reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    // 2. Read files directly from tree
    let files = traverse_tree_recursive(&repo, &tree, "")?;
    for (path, blob_id) in files {
        let blob = repo.find_blob(blob_id)?;
        let content = std::str::from_utf8(blob.content())?;
        // ... index content
    }
}
```

## Implementation Steps

### Step 1: Create Helper Functions

Add these helper functions to `crawler.rs`:

```rust
/// Read a single file from a Git tree object
fn read_file_from_tree(
    repo: &git2::Repository,
    tree: &git2::Tree,
    file_path: &str,
) -> Result<Vec<u8>> {
    let entry = tree
        .get_path(Path::new(file_path))
        .map_err(|e| anyhow!("File not found in tree: {}", e))?;

    let object = entry.to_object(repo)?;
    let blob = object
        .as_blob()
        .ok_or_else(|| anyhow!("Object is not a blob"))?;

    Ok(blob.content().to_vec())
}

/// Recursively traverse a Git tree and collect all file paths
fn traverse_tree_recursive(
    repo: &git2::Repository,
    tree: &git2::Tree,
    prefix: &str,
    files: &mut Vec<(String, git2::Oid)>,
) -> Result<()> {
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("unknown");
        let full_path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };

        match entry.kind() {
            Some(git2::ObjectType::Blob) => {
                files.push((full_path, entry.id()));
            }
            Some(git2::ObjectType::Tree) => {
                let subtree = repo.find_tree(entry.id())?;
                traverse_tree_recursive(repo, &subtree, &full_path, files)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Check if a blob is binary
fn is_binary_blob(blob: &git2::Blob) -> bool {
    blob.is_binary()
}
```

### Step 2: Replace checkout_and_process_branch

Replace the existing `checkout_and_process_branch` method:

```rust
async fn process_branch_from_tree(
    &self,
    git_repo: &git2::Repository,
    branch_name: &str,
    repository: &Repository,
) -> Result<()> {
    info!("Processing branch '{}' using tree reading", branch_name);

    // Get the tree for this branch
    let reference = git_repo.find_reference(&format!("refs/heads/{}", branch_name))?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    // Traverse tree to get all files
    let mut files = Vec::new();
    traverse_tree_recursive(git_repo, &tree, "", &mut files)?;

    info!("Found {} files in branch '{}'", files.len(), branch_name);

    // Process each file
    let mut files_processed = 0;
    let mut files_indexed = 0;

    for (file_path, blob_id) in files {
        // Check for cancellation
        if let Some(token) = self.cancellation_tokens.read().await.get(&repository.id) {
            if token.is_cancelled() {
                info!("Crawl cancelled during tree traversal");
                return Ok(());
            }
        }

        // Check if file is supported
        if !self.is_supported_file(Path::new(&file_path)) {
            continue;
        }

        files_processed += 1;

        // Read blob
        let blob = git_repo.find_blob(blob_id)?;

        // Skip binary files
        if is_binary_blob(&blob) {
            debug!("Skipping binary file: {}", file_path);
            continue;
        }

        // Check file size
        if blob.size() > MAX_FILE_SIZE {
            debug!("Skipping large file: {} ({} bytes)", file_path, blob.size());
            continue;
        }

        // Convert to string
        let content = match std::str::from_utf8(blob.content()) {
            Ok(s) => s.to_string(),
            Err(_) => {
                debug!("Skipping file with invalid UTF-8: {}", file_path);
                continue;
            }
        };

        // Generate deterministic file ID
        let file_id = self.generate_deterministic_file_id_with_branch(
            repository,
            &file_path,
            branch_name,
        );

        // Index the file
        let file_data = FileData {
            id: file_id,
            repository: repository.name.clone(),
            branch: branch_name.to_string(),
            path: file_path.clone(),
            content,
            last_modified: commit.time().seconds() as u64,
        };

        self.search_service.upsert_document(&file_data).await?;
        files_indexed += 1;

        // Update progress
        if files_processed % 10 == 0 {
            self.progress_tracker
                .update_progress(repository.id, files_processed, Some(files.len()), files_indexed)
                .await;
        }
    }

    info!(
        "Branch '{}' processed: {} files processed, {} indexed",
        branch_name, files_processed, files_indexed
    );

    Ok(())
}
```

### Step 3: Update process_all_branches

Update the `process_all_branches` method to use the new tree-based approach:

```rust
async fn process_all_branches_with_tree_reading(
    &self,
    git_repo: &git2::Repository,
    repository: &Repository,
) -> Result<()> {
    let branches: Vec<String> = git_repo
        .branches(Some(git2::BranchType::Local))?
        .filter_map(|b| {
            b.ok()
                .and_then(|(branch, _)| branch.name().ok())
                .flatten()
                .map(|s| s.to_string())
        })
        .collect();

    info!("Processing {} branches for repository {}", branches.len(), repository.name);

    for branch_name in branches {
        // Check for cancellation
        if let Some(token) = self.cancellation_tokens.read().await.get(&repository.id) {
            if token.is_cancelled() {
                info!("Crawl cancelled");
                return Ok(());
            }
        }

        // Process branch using tree reading
        self.process_branch_from_tree(git_repo, &branch_name, repository).await?;
    }

    Ok(())
}
```

### Step 4: Update crawl_repository

Update the main `crawl_repository` method to use the new implementation:

```rust
pub async fn crawl_repository(&self, repository: &Repository) -> Result<()> {
    // ... existing setup code ...

    match repository.repository_type {
        RepositoryType::Git | RepositoryType::GitLab | RepositoryType::GitHub => {
            // Clone or update repository
            let repo_path = self.clone_or_update_repository(repository).await?;
            let git_repo = git2::Repository::open(&repo_path)?;

            // Process branches using tree reading (NO CHECKOUT!)
            self.process_all_branches_with_tree_reading(&git_repo, repository).await?;
        }
        RepositoryType::FileSystem => {
            // Existing filesystem logic
            self.process_repository_files(repository).await?;
        }
    }

    // ... existing cleanup code ...
}
```

## Testing

After implementation, run these tests:

```bash
# Basic tree reading tests
cargo test --test git_tree_reading_test

# Integration tests with CrawlerService
cargo test --test crawler_tree_integration_test

# All crawler tests
cargo test crawler
```

## Expected Benefits

1. **Performance**: 50-70% faster (no checkout overhead)
2. **Concurrency**: Can read multiple branches in parallel
3. **Safety**: No working directory side effects
4. **Reliability**: No race conditions from concurrent checkouts

## Checklist

- [ ] Add helper functions: `read_file_from_tree`, `traverse_tree_recursive`, `is_binary_blob`
- [ ] Implement `process_branch_from_tree` method
- [ ] Update `process_all_branches_with_tree_reading` method
- [ ] Update `crawl_repository` to use tree reading
- [ ] Remove or deprecate `checkout_and_process_branch` method
- [ ] Run all tests and ensure they pass
- [ ] Update any documentation referencing checkout behavior
- [ ] Performance test with large repositories

## Migration Path

For gradual rollout:

1. Implement new methods alongside existing ones
2. Add feature flag to toggle between implementations
3. Test thoroughly in development
4. Roll out to staging
5. Monitor performance and errors
6. Roll out to production
7. Remove old checkout-based code

## Common Pitfalls

1. **Don't forget to check for cancellation** in the tree traversal loop
2. **Handle binary files** before attempting UTF-8 conversion
3. **Update progress tracking** periodically during tree traversal
4. **Preserve deterministic file ID generation** (important for incremental updates)
5. **Test with empty repositories** (no HEAD commit)

## Performance Monitoring

Add metrics to track:
- Time to traverse tree
- Files processed per second
- Memory usage during tree reading
- Comparison with old checkout-based approach

## Questions?

Refer to:
- Test suite: `/klask-rs/tests/git_tree_reading_test.rs`
- Documentation: `/klask-rs/tests/README_GIT_TREE_TESTS.md`
- Or ask the test-specialist agent
