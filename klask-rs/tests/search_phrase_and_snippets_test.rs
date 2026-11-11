//! Exact phrase search (double quotes) and snippet rendering.

use klask_rs::services::search::{FileData, SearchResultsWithTotal};
use klask_rs::services::{SearchQuery, SearchService};
use tempfile::TempDir;
use uuid::Uuid;

async fn new_service() -> (SearchService, TempDir) {
    let temp_dir = TempDir::new().expect("temp dir");
    let index_path = temp_dir.path().join("index");
    let service = SearchService::new(&index_path).expect("search service");
    (service, temp_dir)
}

async fn index(service: &SearchService, name: &str, content: &str) {
    service
        .upsert_file(FileData {
            file_id: Uuid::new_v4(),
            file_name: name,
            file_path: name,
            content,
            repository: "repo",
            project: "project",
            version: "main",
            extension: "txt",
            size: content.len() as u64,
        })
        .await
        .expect("upsert");
    service.commit().await.expect("commit");
}

fn names(result: &SearchResultsWithTotal) -> Vec<String> {
    let mut names: Vec<String> = result.results.iter().map(|r| r.file_name.clone()).collect();
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// Snippet regressions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn camel_case_match_across_fragment_boundary_does_not_panic() {
    let (service, _dir) = new_service().await;
    // The camelCase word starts at byte 296, so a fragment boundary of the
    // tantivy snippet generator falls between its split tokens.
    let content = format!("{} aaaBbbCcc end", "x".repeat(295));
    index(&service, "a.txt", &content).await;

    let result = service.search(SearchQuery::new("aaaBbbCcc".to_string())).await.expect("search must not fail");
    assert_eq!(result.total, 1);
    assert!(result.results[0].content_snippet.contains("<b>"));
}

#[tokio::test]
async fn non_ascii_content_with_phrase_highlight_is_rendered() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "İstanbul é ü fichier tests ici").await;

    let result = service.search(SearchQuery::new("\"fichier tests\"".to_string())).await.expect("search");
    assert_eq!(result.total, 1);
    assert!(result.results[0].content_snippet.contains("<b>fichier tests</b>"));
}

#[tokio::test]
async fn snippet_escapes_html_in_content() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "<script>alert(1)</script> & fichier").await;

    let result = service.search(SearchQuery::new("fichier".to_string())).await.expect("search");
    let snippet = &result.results[0].content_snippet;
    assert!(!snippet.contains("<script>"), "snippet: {snippet}");
    assert!(snippet.contains("&lt;script&gt;"), "snippet: {snippet}");
    assert!(snippet.contains("&amp;"), "snippet: {snippet}");
}

#[tokio::test]
async fn line_number_is_the_line_of_the_displayed_match() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "one\ntwo\nthree fichier\nfour").await;

    let result = service.search(SearchQuery::new("fichier".to_string())).await.expect("search");
    assert_eq!(result.results[0].line_number, Some(3));
}

// ---------------------------------------------------------------------------
// Exact phrases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn phrase_excludes_documents_where_words_are_separate() {
    let (service, _dir) = new_service().await;
    index(&service, "exact.txt", "voici le fichier tests du projet").await;
    index(&service, "separate.txt", "fichier de tests").await;

    let result = service.search(SearchQuery::new("\"fichier tests\"".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["exact.txt"]);

    // Without quotes both words are required, so both documents match.
    let result = service.search(SearchQuery::new("fichier tests".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["exact.txt", "separate.txt"]);
}

#[tokio::test]
async fn phrase_is_case_insensitive_by_default() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "Fichier TESTS").await;

    let result = service.search(SearchQuery::new("\"fichier tests\"".to_string())).await.expect("search");
    assert_eq!(result.total, 1);
}

#[tokio::test]
async fn phrase_is_case_sensitive_when_requested() {
    let (service, _dir) = new_service().await;
    index(&service, "upper.txt", "Fichier Tests").await;
    index(&service, "lower.txt", "fichier tests").await;

    let query = SearchQuery::new("\"Fichier Tests\"".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.expect("search");
    assert_eq!(names(&result), vec!["upper.txt"]);
}

#[tokio::test]
async fn phrase_matches_camel_case_identifier_as_one_unit() {
    let (service, _dir) = new_service().await;
    index(&service, "camel.rs", "let x = vlanID;").await;
    index(&service, "words.rs", "let x = vlan id;").await;

    let result = service.search(SearchQuery::new("\"vlanID\"".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["camel.rs"]);
}

#[tokio::test]
async fn phrase_matches_camel_case_identifier_inside_a_phrase() {
    let (service, _dir) = new_service().await;
    index(&service, "a.rs", "set vlanID value").await;
    index(&service, "b.rs", "set vlanID other").await;

    let result = service.search(SearchQuery::new("\"vlanID value\"".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["a.rs"]);
}

#[tokio::test]
async fn phrase_mixed_with_plain_words() {
    let (service, _dir) = new_service().await;
    index(&service, "ok.txt", "mot fichier tests autre").await;
    index(&service, "ko.txt", "mot fichier autre tests").await;
    index(&service, "ko2.txt", "fichier mot tests autre").await;

    let result = service.search(SearchQuery::new("mot \"fichier tests\" autre".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["ok.txt"]);
}

#[tokio::test]
async fn unclosed_quote_is_ignored_and_words_still_match() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "fichier de tests").await;

    let result = service.search(SearchQuery::new("\"fichier tests".to_string())).await.expect("search");
    assert_eq!(names(&result), vec!["a.txt"]);
}

#[tokio::test]
async fn single_word_phrase_behaves_like_a_word() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "contient fichier ici").await;

    let result = service.search(SearchQuery::new("\"fichier\"".to_string())).await.expect("search");
    assert_eq!(result.total, 1);
}

#[tokio::test]
async fn empty_phrase_matches_nothing() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "contient fichier ici").await;

    let result = service.search(SearchQuery::new("\"\"".to_string())).await.expect("search");
    assert_eq!(result.total, 0);
}

#[tokio::test]
async fn phrase_is_not_fuzzy_but_plain_words_are() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "fichir tests").await;

    let phrase = SearchQuery::new("\"fichier tests\"".to_string()).with_fuzzy(true);
    assert_eq!(service.search(phrase).await.expect("search").total, 0);

    let words = SearchQuery::new("fichier tests".to_string()).with_fuzzy(true);
    assert_eq!(service.search(words).await.expect("search").total, 1);
}

#[tokio::test]
async fn snippet_highlights_the_phrase_only() {
    let (service, _dir) = new_service().await;
    index(&service, "a.txt", "le fichier de tests puis le fichier tests ici").await;

    let result = service.search(SearchQuery::new("\"fichier tests\"".to_string())).await.expect("search");
    let snippet = &result.results[0].content_snippet;
    assert!(snippet.contains("<b>fichier tests</b>"), "snippet: {snippet}");
    assert!(!snippet.contains("<b>fichier</b> de"), "snippet: {snippet}");
}
