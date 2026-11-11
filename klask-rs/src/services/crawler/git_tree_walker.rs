use anyhow::{Result, anyhow};
use gix::ObjectId;
use gix::bstr::ByteSlice;
use tracing::{debug, info};

/// Maximum file size to process (10MB)
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Represents a file entry in a Git tree
#[derive(Debug, Clone)]
pub struct GitFileEntry {
    pub path: String,
    pub oid: ObjectId,
}

/// Helper struct to walk Git trees and read file contents directly from Git database
pub struct GitTreeWalker;

impl GitTreeWalker {
    /// Recursively walk a Git tree and collect all file entries
    pub fn walk_tree(repo: &gix::Repository, tree_id: &ObjectId, base_path: &str) -> Result<Vec<GitFileEntry>> {
        let mut files = Vec::new();
        let tree = repo.find_object(*tree_id)?.try_into_tree().map_err(|_| anyhow!("Object is not a tree"))?;

        for entry in tree.iter() {
            let entry = entry?;
            let name = entry.filename().to_str().map_err(|_| anyhow!("Invalid UTF-8 in filename"))?;
            let full_path = if base_path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", base_path, name)
            };

            // Check entry mode to determine if it's a file or directory
            if entry.mode().is_blob() {
                // It's a file
                files.push(GitFileEntry { path: full_path, oid: entry.oid().to_owned() });
            } else if entry.mode().is_tree() {
                // It's a directory, recurse
                let subtree_files = Self::walk_tree(repo, &entry.oid().to_owned(), &full_path)?;
                files.extend(subtree_files);
            }
            // Skip links, submodules, etc.
        }

        Ok(files)
    }

    /// Read a blob as UTF-8 text.
    ///
    /// The object size is read from its header first, so oversized blobs are rejected without
    /// being inflated. Otherwise the blob is loaded once and its buffer is moved into the
    /// resulting `String` (no copy). Returns `Ok(None)` for oversized or non-UTF-8 (binary) blobs.
    pub fn read_text_blob(repo: &gix::Repository, oid: &ObjectId) -> Result<Option<String>> {
        let header = repo.find_header(*oid)?;
        if header.size() > MAX_FILE_SIZE {
            debug!(
                "[BLOB] Skipping blob {} ({} bytes > {} bytes)",
                oid,
                header.size(),
                MAX_FILE_SIZE
            );
            return Ok(None);
        }

        let mut obj = repo.find_object(*oid)?;
        if obj.kind != gix::object::Kind::Blob {
            return Err(anyhow!("Object {} is not a blob", oid));
        }

        // `Object` returns its buffer to the repository on drop, so take the data out instead of cloning it.
        let data = std::mem::take(&mut obj.data);
        let blob_size = data.len();
        match String::from_utf8(data) {
            Ok(content) => Ok(Some(content)),
            Err(e) => {
                debug!(
                    "[BLOB] Blob {} is not UTF-8 ({} bytes, valid up to byte {})",
                    oid,
                    blob_size,
                    e.utf8_error().valid_up_to()
                );
                Ok(None)
            }
        }
    }

    /// Get all branches from a gix repository
    pub fn get_all_branches(repo: &gix::Repository) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        let mut branch_set = std::collections::HashSet::new();

        // Get all references
        let references = repo.references()?;

        for reference in references.all()? {
            let reference = reference.map_err(|e| anyhow!("Failed to iterate references: {:?}", e))?;
            let name = reference.name().as_bstr().to_string();

            // Check for local branches (refs/heads/*)
            if let Some(branch_name) = name.strip_prefix("refs/heads/") {
                info!("Found local branch: {}", branch_name);
                branch_set.insert(branch_name.to_string());
            }
            // Check for remote branches (refs/remotes/origin/*)
            else if let Some(branch_name) = name.strip_prefix("refs/remotes/origin/")
                && branch_name != "HEAD"
            {
                info!("Found remote branch: {}", branch_name);
                branch_set.insert(branch_name.to_string());
            }
        }

        branches.extend(branch_set);
        Ok(branches)
    }

    /// Get the tree ID for a specific branch
    pub fn get_branch_tree_id(repo: &gix::Repository, branch_name: &str) -> Result<ObjectId> {
        // Try remote branch first (refs/remotes/origin/branch_name)
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let local_ref = format!("refs/heads/{}", branch_name);

        let reference = repo.find_reference(&remote_ref).or_else(|_| repo.find_reference(&local_ref))?;

        let commit_id = reference.id().detach();
        let commit = repo
            .find_object(commit_id)?
            .try_into_commit()
            .map_err(|_| anyhow!("Reference does not point to a commit"))?;

        let tree_id = commit.tree_id()?.into();
        Ok(tree_id)
    }
}
