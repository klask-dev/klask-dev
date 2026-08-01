//! Hybrid/semantic query path (plan §5).
//!
//! Wires the three pieces that already exist in isolation — the Tantivy keyword
//! search ([`SearchService`]), the local embedding model ([`EmbeddingProvider`])
//! and the LanceDB vector store (via [`VectorIndexer::search`]) — into the two
//! semantic-aware modes:
//!
//! - **Semantic**: embed the query, ANN-search the vector store, return those
//!   hits hydrated with content snippets from Tantivy.
//! - **Hybrid**: run keyword and semantic in parallel and fuse them with
//!   Reciprocal Rank Fusion ([`super::fusion`]), which needs only ranks so the
//!   incomparable BM25 / cosine scores never have to be normalized.
//!
//! Both degrade to plain keyword search when the semantic backend is missing —
//! that decision lives in the API layer; this module is only reached when the
//! backend is present. Gated on the `semantic-search` feature.
#![cfg(feature = "semantic-search")]

use super::embedder::EmbeddingProvider;
use super::fusion::{DEFAULT_RRF_K, reciprocal_rank_fusion};
use super::indexer::VectorIndexer;
use super::store::VectorSearchFilters;
use crate::services::search::{MatchSource, SearchMode, SearchQuery, SearchResultsWithTotal, SearchService};
use anyhow::{Context, Result};
use std::sync::Arc;
use uuid::Uuid;

/// How many candidates to pull from each engine before fusion/paging.
///
/// We over-fetch beyond the requested page so fusion has enough overlap to be
/// meaningful and so paging into fused results stays stable. Bounded to keep
/// the vector scan and Tantivy fetch cheap.
const CANDIDATE_MULTIPLIER: usize = 5;
const MAX_CANDIDATES: usize = 500;

/// Run a semantic or hybrid search, returning a page of fused results.
///
/// `query` carries the page (`offset`/`limit`) and the facet filters, which are
/// applied to *both* engines so they see the same universe. Facets in the
/// response come from the keyword path only (plan decision): they describe the
/// keyword universe and stay consistent with current behaviour.
pub async fn semantic_search(
    search_service: &SearchService,
    embedder: &Arc<dyn EmbeddingProvider>,
    indexer: &VectorIndexer,
    query: SearchQuery,
) -> Result<SearchResultsWithTotal> {
    debug_assert!(
        query.mode.needs_semantic(),
        "semantic_search called for a keyword query"
    );

    // Candidate budget: enough to cover the requested page plus overlap.
    let page_end = query.offset.saturating_add(query.limit);
    let candidates = page_end.saturating_mul(CANDIDATE_MULTIPLIER).clamp(query.limit.max(1), MAX_CANDIDATES);

    // --- Vector side: embed the query, ANN-search the store. ---
    let vector_hits = vector_candidates(embedder, indexer, &query, candidates).await?;

    match query.mode {
        SearchMode::Semantic => hydrate_semantic_only(search_service, &query, vector_hits, page_end).await,
        SearchMode::Hybrid => hybrid_fuse(search_service, &query, vector_hits, candidates).await,
        // Unreachable: guarded by needs_semantic() at the call site + assert above.
        SearchMode::Keyword => search_service.search(query).await,
    }
}

/// Embed the query text and run the vector store ANN search with the query's
/// facet filters. Embedding is CPU-bound, so it runs on the blocking pool.
async fn vector_candidates(
    embedder: &Arc<dyn EmbeddingProvider>,
    indexer: &VectorIndexer,
    query: &SearchQuery,
    candidates: usize,
) -> Result<Vec<super::store::VectorHit>> {
    let text = query.query.clone();
    let embedder = embedder.clone();
    let vectors = tokio::task::spawn_blocking(move || embedder.embed(&[text]))
        .await
        .context("query embedding task panicked")??;
    let query_vector = vectors.into_iter().next().context("embedder returned no vector for the query")?;

    let filters = filters_from_query(query);
    indexer.search(&query_vector, candidates, &filters).await.context("vector search failed")
}

/// Translate the keyword search's comma-separated facet filters into the vector
/// store's filter shape so both engines restrict to the same universe.
fn filters_from_query(query: &SearchQuery) -> VectorSearchFilters {
    let split = |f: &Option<String>| -> Vec<String> {
        f.as_deref()
            .map(|s| s.split(',').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect())
            .unwrap_or_default()
    };
    VectorSearchFilters {
        repositories: split(&query.repository_filter),
        projects: split(&query.project_filter),
        versions: split(&query.version_filter),
        extensions: split(&query.extension_filter),
    }
}

/// Semantic-only: page the vector hits, then hydrate that page with full result
/// data (name, snippet, score) from Tantivy by file_id.
async fn hydrate_semantic_only(
    search_service: &SearchService,
    query: &SearchQuery,
    vector_hits: Vec<super::store::VectorHit>,
    page_end: usize,
) -> Result<SearchResultsWithTotal> {
    let ordered_ids: Vec<Uuid> = dedup_by_file(vector_hits.iter().map(|h| h.file_id));
    let total = ordered_ids.len() as u64;

    // Best (closest) chunk per file: its line range anchors/trims the snippet,
    // and its cosine distance becomes the result score. vector_hits arrive
    // closest-first, so first-seen wins.
    let mut best_chunk: std::collections::HashMap<Uuid, &super::store::VectorHit> = std::collections::HashMap::new();
    for hit in &vector_hits {
        best_chunk.entry(hit.file_id).or_insert(hit);
    }

    let page_ids: Vec<Uuid> = ordered_ids.iter().skip(query.offset).take(query.limit).copied().collect();
    // fetch_results_by_file_ids preserves input order, so the ranking is kept.
    let mut results = search_service.fetch_results_by_file_ids(&page_ids).await?;
    for result in &mut results {
        if let Some(hit) = best_chunk.get(&result.file_id) {
            result.line_number = Some(hit.start_line);
            // Replace Tantivy's keyword score (meaningless for a semantic query)
            // with the cosine similarity of the matched chunk.
            result.score = similarity_from_distance(hit.distance);
            // Show the matched chunk, not the whole file (which made semantic
            // results look unrelated even when the ranking was right).
            result.content_snippet = snippet_for_chunk(&result.content_snippet, hit.start_line, hit.end_line);
        }
        // Semantic-only: every result came from the vector engine.
        result.match_source = Some(MatchSource::Semantic);
    }

    let facets = maybe_keyword_facets(search_service, query, page_end).await?;
    Ok(SearchResultsWithTotal { results, total, facets })
}

/// Hybrid: run the keyword search, fuse its ranking with the vector ranking via
/// RRF (by file_id), page the fused order, then hydrate.
async fn hybrid_fuse(
    search_service: &SearchService,
    query: &SearchQuery,
    vector_hits: Vec<super::store::VectorHit>,
    candidates: usize,
) -> Result<SearchResultsWithTotal> {
    // Keyword candidate ranking (over-fetched, page 0..candidates), keeping its
    // facets for the response.
    let mut keyword_query = query.clone();
    keyword_query.mode = SearchMode::Keyword;
    keyword_query.offset = 0;
    keyword_query.limit = candidates;
    let keyword = search_service.search(keyword_query).await?;

    let keyword_ranking: Vec<Uuid> = dedup_by_file(keyword.results.iter().map(|r| r.file_id));
    let vector_ranking: Vec<Uuid> = dedup_by_file(vector_hits.iter().map(|h| h.file_id));

    // Membership sets for per-result provenance (the UI badge): a file can be a
    // keyword hit, a vector hit, or both. Built before the rankings are moved
    // into the fusion call.
    let in_keyword: std::collections::HashSet<Uuid> = keyword_ranking.iter().copied().collect();
    let in_vector: std::collections::HashSet<Uuid> = vector_ranking.iter().copied().collect();

    // Best vector chunk per file (for the snippet range on vector/both hits).
    let mut best_chunk: std::collections::HashMap<Uuid, &super::store::VectorHit> = std::collections::HashMap::new();
    for hit in &vector_hits {
        best_chunk.entry(hit.file_id).or_insert(hit);
    }

    let fused = reciprocal_rank_fusion(&[keyword_ranking, vector_ranking], DEFAULT_RRF_K);
    let total = fused.len() as u64;
    // Fused RRF score per file, so the response is sorted/scored by the actual
    // hybrid ranking rather than Tantivy's keyword score.
    let fused_score: std::collections::HashMap<Uuid, f32> = fused.iter().copied().collect();

    let page_ids: Vec<Uuid> = fused.iter().skip(query.offset).take(query.limit).map(|(id, _)| *id).collect();
    // fetch_results_by_file_ids preserves input (fused) order.
    let mut results = search_service.fetch_results_by_file_ids(&page_ids).await?;
    for result in &mut results {
        let source = match_source(&in_keyword, &in_vector, result.file_id);
        result.match_source = Some(source);
        if let Some(score) = fused_score.get(&result.file_id) {
            result.score = *score;
        }
        // For results the vector engine matched, anchor + trim the snippet on the
        // matched chunk; pure-keyword hits keep Tantivy's highlighted snippet.
        if source != MatchSource::Keyword
            && let Some(hit) = best_chunk.get(&result.file_id)
        {
            result.line_number = Some(hit.start_line);
            result.content_snippet = snippet_for_chunk(&result.content_snippet, hit.start_line, hit.end_line);
        }
    }

    Ok(SearchResultsWithTotal { results, total, facets: keyword.facets })
}

/// Keyword-path facets for semantic-only mode, only when the caller asked for
/// them (avoids an extra Tantivy query otherwise).
async fn maybe_keyword_facets(
    search_service: &SearchService,
    query: &SearchQuery,
    page_end: usize,
) -> Result<Option<crate::services::search::SearchFacets>> {
    if !query.include_facets {
        return Ok(None);
    }
    let mut facet_query = query.clone();
    facet_query.mode = SearchMode::Keyword;
    facet_query.offset = 0;
    facet_query.limit = page_end.max(1);
    Ok(search_service.search(facet_query).await?.facets)
}

/// Classify a fused result by which engine(s) it appeared in, for the UI badge.
/// A file present in both rankings is `Both`; otherwise it came from whichever
/// single engine contains it (every fused id is in at least one).
fn match_source(
    in_keyword: &std::collections::HashSet<Uuid>,
    in_vector: &std::collections::HashSet<Uuid>,
    file_id: Uuid,
) -> MatchSource {
    match (in_keyword.contains(&file_id), in_vector.contains(&file_id)) {
        (true, true) => MatchSource::Both,
        (false, true) => MatchSource::Semantic,
        // (true, false) and the impossible (false, false) → keyword.
        _ => MatchSource::Keyword,
    }
}

/// Map a cosine *distance* (0 = identical, 2 = opposite) to a similarity score
/// in `[0, 1]` for display/sorting, so semantic results carry a meaningful score
/// instead of Tantivy's keyword score. `1 - distance` is the cosine similarity;
/// clamped because distance can drift slightly outside `[0, 2]`.
fn similarity_from_distance(distance: f32) -> f32 {
    (1.0 - distance).clamp(0.0, 1.0)
}

/// Trim a file's stored content to the matched chunk's line range so the snippet
/// shows the semantically-relevant region, not the whole file. Lines are
/// 1-based and inclusive (matching `ChunkRecord`); out-of-range or empty input
/// falls back to the original content so we never show nothing.
fn snippet_for_chunk(content: &str, start_line: u32, end_line: u32) -> String {
    if content.is_empty() || start_line == 0 || end_line < start_line {
        return content.to_string();
    }
    let start = (start_line as usize).saturating_sub(1);
    let end = end_line as usize; // inclusive 1-based end == exclusive 0-based end
    let lines: Vec<&str> = content.lines().collect();
    if start >= lines.len() {
        return content.to_string();
    }
    let end = end.min(lines.len());
    lines[start..end].join("\n")
}

/// Distinct file_ids in first-seen order (a file can contribute several chunks
/// to the vector ranking; the file's best rank is the one that counts).
fn dedup_by_file(ids: impl Iterator<Item = Uuid>) -> Vec<Uuid> {
    let mut seen = std::collections::HashSet::new();
    ids.filter(|id| seen.insert(*id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::search::FileData;
    use crate::services::semantic::chunker::ChunkOptions;
    use crate::services::semantic::store::LanceVectorStore;
    use std::sync::Arc;
    use tempfile::TempDir;

    const DIM: usize = 16;

    /// Deterministic, ONNX-free embedder: each of a fixed vocabulary of topic
    /// words owns one dimension, and a text embeds to the (normalized) sum of
    /// the topic dimensions it contains. So a query sharing a topic word with a
    /// chunk lands close to it in cosine space — enough to assert ranking
    /// without downloading a model.
    struct TopicProvider;

    const TOPICS: [&str; 4] = ["authentication", "database", "rendering", "networking"];

    impl TopicProvider {
        fn vector_for(text: &str) -> Vec<f32> {
            let lower = text.to_lowercase();
            let mut v = vec![0.0_f32; DIM];
            for (i, topic) in TOPICS.iter().enumerate() {
                if lower.contains(topic) {
                    v[i] = 1.0;
                }
            }
            // Avoid an all-zero vector (undefined cosine): fall back to a fixed dim.
            if v.iter().all(|x| *x == 0.0) {
                v[DIM - 1] = 1.0;
            }
            v
        }
    }

    impl EmbeddingProvider for TopicProvider {
        fn dimension(&self) -> usize {
            DIM
        }
        fn model_id(&self) -> &str {
            "topic-mock"
        }
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|t| Self::vector_for(t)).collect())
        }
    }

    /// A document to seed into both Tantivy (keyword) and the vector store.
    struct Doc {
        file_id: Uuid,
        path: &'static str,
        content: &'static str,
    }

    /// Build a SearchService + VectorIndexer sharing the same documents, so the
    /// query path can be exercised end to end against real engines.
    async fn setup(docs: &[Doc]) -> (SearchService, VectorIndexer, Arc<dyn EmbeddingProvider>, TempDir) {
        let dir = TempDir::new().unwrap();
        let search = SearchService::new(dir.path().join("tantivy")).unwrap();

        let provider: Arc<dyn EmbeddingProvider> = Arc::new(TopicProvider);
        let store = Arc::new(LanceVectorStore::open(dir.path().join("vectors"), DIM).await.unwrap());
        let indexer = VectorIndexer::start(provider.clone(), store.clone(), ChunkOptions::default(), 8, 32);

        for doc in docs {
            search
                .upsert_file(FileData {
                    file_id: doc.file_id,
                    file_name: doc.path,
                    file_path: doc.path,
                    content: doc.content,
                    repository: "repo",
                    project: "repo",
                    version: "main",
                    extension: "rs",
                    size: doc.content.len() as u64,
                })
                .await
                .unwrap();
            indexer
                .index_file(crate::services::semantic::IndexJob {
                    file_id: doc.file_id,
                    repository: "repo".to_string(),
                    project: "repo".to_string(),
                    version: "main".to_string(),
                    path: doc.path.to_string(),
                    extension: "rs".to_string(),
                    content: doc.content.to_string(),
                })
                .await
                .unwrap();
        }
        search.commit().await.unwrap();
        // Let the embedding worker drain the queue before querying.
        wait_for_chunks(&indexer, docs.len() as u64).await;

        (search, indexer, provider, dir)
    }

    async fn wait_for_chunks(indexer: &VectorIndexer, at_least: u64) {
        for _ in 0..100 {
            if indexer.count().await.unwrap() >= at_least {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("vector store did not reach {at_least} chunks in time");
    }

    fn query(text: &str, mode: SearchMode) -> SearchQuery {
        SearchQuery { query: text.to_string(), limit: 10, offset: 0, mode, ..Default::default() }
    }

    #[tokio::test]
    async fn test_semantic_search_ranks_topic_match_first() {
        let auth = Uuid::new_v4();
        let db = Uuid::new_v4();
        let (search, indexer, provider, _dir) = setup(&[
            Doc { file_id: auth, path: "auth.rs", content: "fn login() { /* authentication */ }" },
            Doc { file_id: db, path: "db.rs", content: "fn connect() { /* database */ }" },
        ])
        .await;

        let results = semantic_search(
            &search,
            &provider,
            &indexer,
            query("authentication", SearchMode::Semantic),
        )
        .await
        .unwrap();

        assert!(
            !results.results.is_empty(),
            "semantic search should return the topic match"
        );
        assert_eq!(
            results.results[0].file_id, auth,
            "the authentication doc must rank first"
        );
    }

    #[tokio::test]
    async fn test_semantic_search_sets_line_number_anchor() {
        let auth = Uuid::new_v4();
        let (search, indexer, provider, _dir) =
            setup(&[Doc { file_id: auth, path: "auth.rs", content: "fn login() { /* authentication */ }" }]).await;

        let results = semantic_search(
            &search,
            &provider,
            &indexer,
            query("authentication", SearchMode::Semantic),
        )
        .await
        .unwrap();
        assert_eq!(
            results.results[0].line_number,
            Some(1),
            "snippet should anchor on the matched chunk's start line"
        );
        // The score must be the cosine similarity (0..=1), not Tantivy's keyword
        // score — that was the bug making semantic results look meaningless.
        let score = results.results[0].score;
        assert!(
            (0.0..=1.0).contains(&score),
            "semantic score should be a cosine similarity in [0,1], got {score}"
        );
    }

    #[tokio::test]
    async fn test_hybrid_search_includes_both_engines() {
        let auth = Uuid::new_v4();
        let db = Uuid::new_v4();
        let (search, indexer, provider, _dir) = setup(&[
            // "login" is a keyword hit; "authentication" is the semantic topic.
            Doc { file_id: auth, path: "auth.rs", content: "fn login() { /* authentication */ }" },
            Doc { file_id: db, path: "db.rs", content: "fn connect() { /* database */ }" },
        ])
        .await;

        // Query text matches the auth doc both lexically ("login") and
        // semantically ("authentication"), so it should top the fused ranking.
        let results = semantic_search(
            &search,
            &provider,
            &indexer,
            query("login authentication", SearchMode::Hybrid),
        )
        .await
        .unwrap();

        assert!(!results.results.is_empty());
        assert_eq!(
            results.results[0].file_id, auth,
            "doc winning both engines must rank first under RRF"
        );
        // auth matched lexically ("login") *and* semantically ("authentication").
        assert_eq!(
            results.results[0].match_source,
            Some(MatchSource::Both),
            "a doc hit by both engines must be tagged Both for the UI badge"
        );
    }

    #[tokio::test]
    async fn test_semantic_mode_tags_results_semantic() {
        let auth = Uuid::new_v4();
        let (search, indexer, provider, _dir) =
            setup(&[Doc { file_id: auth, path: "auth.rs", content: "fn login() { /* authentication */ }" }]).await;

        let results = semantic_search(
            &search,
            &provider,
            &indexer,
            query("authentication", SearchMode::Semantic),
        )
        .await
        .unwrap();
        assert_eq!(
            results.results[0].match_source,
            Some(MatchSource::Semantic),
            "semantic-only results must be tagged Semantic"
        );
    }

    #[test]
    fn test_similarity_from_distance() {
        assert_eq!(super::similarity_from_distance(0.0), 1.0); // identical
        assert!((super::similarity_from_distance(0.27) - 0.73).abs() < 1e-5);
        assert_eq!(super::similarity_from_distance(1.5), 0.0); // far → clamped
        assert_eq!(super::similarity_from_distance(-0.01), 1.0); // drift → clamped
    }

    #[test]
    fn test_snippet_for_chunk() {
        let content = "line1\nline2\nline3\nline4\nline5";
        // 1-based inclusive range [2,4] → lines 2..4.
        assert_eq!(super::snippet_for_chunk(content, 2, 4), "line2\nline3\nline4");
        // Single line.
        assert_eq!(super::snippet_for_chunk(content, 1, 1), "line1");
        // End past EOF clamps.
        assert_eq!(super::snippet_for_chunk(content, 4, 99), "line4\nline5");
        // Degenerate inputs fall back to full content.
        assert_eq!(super::snippet_for_chunk(content, 0, 3), content);
        assert_eq!(super::snippet_for_chunk(content, 9, 10), content);
        assert_eq!(super::snippet_for_chunk("", 1, 2), "");
    }

    #[test]
    fn test_match_source_classification() {
        use std::collections::HashSet;
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let kw: HashSet<Uuid> = [a, b].into_iter().collect();
        let vec: HashSet<Uuid> = [b, c].into_iter().collect();
        assert_eq!(match_source(&kw, &vec, a), MatchSource::Keyword);
        assert_eq!(match_source(&kw, &vec, b), MatchSource::Both);
        assert_eq!(match_source(&kw, &vec, c), MatchSource::Semantic);
    }

    #[tokio::test]
    async fn test_semantic_search_respects_filters() {
        let auth = Uuid::new_v4();
        let (search, indexer, provider, _dir) =
            setup(&[Doc { file_id: auth, path: "auth.rs", content: "fn login() { /* authentication */ }" }]).await;

        // Filtering to a non-existent repository must yield nothing.
        let mut q = query("authentication", SearchMode::Semantic);
        q.repository_filter = Some("nonexistent".to_string());
        let results = semantic_search(&search, &provider, &indexer, q).await.unwrap();
        assert!(
            results.results.is_empty(),
            "facet filter must restrict the vector universe"
        );
    }
}
