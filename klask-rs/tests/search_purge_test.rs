use klask_rs::services::search::{FileData, SearchService};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tempfile::TempDir;
use uuid::Uuid;

fn new_service() -> (SearchService, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let service = SearchService::new(dir.path().join("index")).expect("search service");
    (service, dir)
}

/// Adds one document per name for `repository`, with a file_id derived from the name so that
/// re-adding the same names simulates a re-crawl.
async fn add_documents(service: &SearchService, repository: &str, names: &[&str]) {
    for name in names {
        let mut hasher = DefaultHasher::new();
        (repository, name).hash(&mut hasher);
        let file_id = Uuid::from_u128(u128::from(hasher.finish()));
        service
            .index_file(FileData {
                file_id,
                file_name: name,
                file_path: name,
                content: "fn main() {}",
                repository,
                project: repository,
                version: "main",
                extension: "rs",
                size: 12,
            })
            .await
            .expect("index file");
    }
}

#[tokio::test]
async fn queued_purge_is_applied_by_the_next_commit_without_duplicates() {
    let (service, _dir) = new_service();
    let names = ["a.rs", "b.rs", "c.rs"];

    add_documents(&service, "repo", &names).await;
    service.commit().await.expect("first commit");
    assert_eq!(service.get_document_count().expect("count"), 3);

    // Re-crawl: queue the purge, then index the same files again, then a single commit.
    service.queue_project_deletion("repo").await;
    add_documents(&service, "repo", &names).await;
    service.commit().await.expect("second commit");

    assert_eq!(
        service.get_document_count().expect("count"),
        3,
        "old documents must be replaced, not kept"
    );
}

#[tokio::test]
async fn queued_purge_keeps_documents_of_other_repositories() {
    let (service, _dir) = new_service();

    add_documents(&service, "repo", &["a.rs", "b.rs"]).await;
    add_documents(&service, "other", &["x.rs"]).await;
    service.commit().await.expect("first commit");

    service.queue_project_deletion("repo").await;
    service.commit().await.expect("commit");

    assert_eq!(service.get_document_count().expect("count"), 1);
}

#[tokio::test]
async fn documents_stay_searchable_until_the_purge_is_committed() {
    let (service, _dir) = new_service();

    add_documents(&service, "repo", &["a.rs", "b.rs"]).await;
    service.commit().await.expect("first commit");

    // Queuing the purge must not make the repository disappear before the commit.
    service.queue_project_deletion("repo").await;
    assert_eq!(service.get_document_count().expect("count"), 2);
}
