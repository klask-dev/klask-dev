//! Corpus-level retrieval evaluation: keyword vs semantic vs hybrid (plan phase 6).
//!
//! The model probe (`semantic-eval`) ranks 8 snippets against 8 queries. That
//! can eliminate an option but it cannot validate one: discriminating among 8
//! documents says little about finding one file among tens of thousands. This
//! binary runs the real query path over a real corpus and reports recall@10.
//!
//! It is self-contained on purpose: it builds its own Tantivy index and its own
//! LanceDB store from a directory on disk, so an evaluation needs no server, no
//! PostgreSQL and no crawl. That also makes it reproducible by anyone cloning
//! the project, which matters for an open-source tool where retrieval quality
//! claims should be checkable.
//!
//! ```sh
//! # index this repository and score the golden queries
//! cargo run --release --features semantic-search --bin semantic-recall -- --repo ..
//! # re-score without re-indexing (much faster while editing the golden set)
//! cargo run --release --features semantic-search --bin semantic-recall -- --repo .. --reuse
//! ```
//!
//! The golden set lives in `eval/golden_queries.json` so it can be reviewed and
//! extended without recompiling. Each entry pairs a natural-language question
//! with the file paths that legitimately answer it; a query counts as a hit
//! when any expected path appears in the top 10.

use anyhow::{Context, Result};
use klask_rs::config::SemanticSearchConfig;
use klask_rs::services::search::{FileData, SearchMode, SearchQuery, SearchService};
use klask_rs::services::semantic::chunker::ChunkOptions;
use klask_rs::services::semantic::embedder::{EmbeddingProvider, FastEmbedProvider};
use klask_rs::services::semantic::query::semantic_search;
use klask_rs::services::semantic::store::LanceVectorStore;
use klask_rs::services::semantic::{IndexJob, VectorIndexer, WriteMode};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use walkdir::WalkDir;

/// Extensions indexed by the harness. Deliberately a small text-only set: the
/// point is to score retrieval, not to re-test the crawler's binary detection.
const INDEXED_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "kt", "rb", "php", "cs", "sql", "sh", "yaml", "yml", "toml",
    "md",
];

/// Directories never worth embedding: build output and vendored dependencies.
const SKIPPED_DIRS: &[&str] =
    &[".git", "target", "node_modules", "dist", "build", "coverage", ".next", "vendor", "__pycache__"];

/// The harness must not index its own artifacts. The golden set contains every
/// evaluation query verbatim and the probe's doc comments describe the concepts
/// under test, so indexing them would let a query retrieve its own answer key.
const SKIPPED_PATH_FRAGMENTS: &[&str] = &["klask-rs/eval/", "src/bin/semantic_"];

/// Files above this size are skipped: generated bundles and lockfiles add
/// thousands of chunks and answer no question anyone asks.
const MAX_FILE_BYTES: u64 = 256 * 1024;

const TOP_K: usize = 10;

#[derive(Debug, Deserialize)]
struct GoldenSet {
    queries: Vec<GoldenQuery>,
}

#[derive(Debug, Deserialize)]
struct GoldenQuery {
    /// The natural-language question, as a user would type it.
    query: String,
    /// Path suffixes that legitimately answer it. A hit is any of them
    /// appearing in the top K.
    expect: Vec<String>,
    /// Why this is the expected answer. Documentation for the reviewer; the
    /// harness never reads it.
    #[serde(default)]
    #[allow(dead_code)]
    note: String,
}

struct Args {
    repo: PathBuf,
    queries: PathBuf,
    work: PathBuf,
    model: String,
    reuse: bool,
    /// Timed repetitions per query and per mode, for the latency percentiles.
    repeats: usize,
}

fn parse_args() -> Args {
    let mut args = Args {
        repo: PathBuf::from(".."),
        queries: PathBuf::from("eval/golden_queries.json"),
        work: PathBuf::from("target/eval"),
        model: "jinaai/jina-embeddings-v2-base-code".to_string(),
        reuse: false,
        repeats: 5,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--repo" => args.repo = PathBuf::from(it.next().expect("--repo needs a path")),
            "--queries" => args.queries = PathBuf::from(it.next().expect("--queries needs a path")),
            "--work" => args.work = PathBuf::from(it.next().expect("--work needs a path")),
            "--model" => args.model = it.next().expect("--model needs a model code"),
            "--reuse" => args.reuse = true,
            "--repeats" => {
                args.repeats = it.next().expect("--repeats needs a count").parse().expect("--repeats must be a number")
            }
            other => panic!("unknown flag {other}"),
        }
    }
    args
}

/// Stable id derived from the path, mirroring the crawler's own scheme so a
/// re-run re-indexes a file in place instead of duplicating it.
fn deterministic_id(path: &str) -> Uuid {
    let digest = Sha256::digest(path.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

/// Collect the files worth indexing, as (relative path, contents).
fn collect_files(root: &Path) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_str().is_some_and(|n| SKIPPED_DIRS.contains(&n) && e.file_type().is_dir()))
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !INDEXED_EXTENSIONS.contains(&ext) {
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue; // not valid UTF-8: the crawler would reject it too
        };
        let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
        if SKIPPED_PATH_FRAGMENTS.iter().any(|frag| relative.contains(frag)) {
            continue;
        }
        files.push((relative, content));
    }
    files.sort();
    Ok(files)
}

/// Index every file into both engines, then commit and compact.
async fn build_index(search: &SearchService, indexer: &VectorIndexer, files: &[(String, String)]) -> Result<()> {
    let started = std::time::Instant::now();
    for (i, (path, content)) in files.iter().enumerate() {
        let file_id = deterministic_id(path);
        let name = path.rsplit('/').next().unwrap_or(path).to_string();
        let ext = path.rsplit('.').next().unwrap_or("").to_string();

        search
            .upsert_file(FileData {
                file_id,
                file_name: &name,
                file_path: path,
                content,
                repository: "eval",
                project: "eval",
                version: "main",
                extension: &ext,
                size: content.len() as u64,
            })
            .await
            .with_context(|| format!("failed to index {path} in Tantivy"))?;

        indexer
            .index_file(IndexJob {
                file_id,
                repository: "eval".to_string(),
                project: "eval".to_string(),
                version: "main".to_string(),
                path: path.clone(),
                extension: ext,
                content: content.clone(),
                mode: WriteMode::Append,
            })
            .await?;

        if (i + 1) % 100 == 0 {
            println!(
                "  {} / {} files handed over ({} still to embed, {:.0}s)",
                i + 1,
                files.len(),
                indexer.pending(),
                started.elapsed().as_secs_f32()
            );
        }
    }

    search.commit().await?;
    println!("  Tantivy committed, waiting for the embedding queue to drain...");
    while indexer.pending() > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    indexer.optimize().await?;
    println!(
        "  indexed {} files / {} chunks in {:.0}s\n",
        files.len(),
        indexer.count().await.unwrap_or(0),
        started.elapsed().as_secs_f32()
    );
    Ok(())
}

/// Rank of the first expected path in the results, 1-based.
fn rank_of_expected(results: &[String], expect: &[String]) -> Option<usize> {
    results.iter().position(|path| expect.iter().any(|want| path.ends_with(want.as_str()))).map(|i| i + 1)
}

/// Percentile of already-collected samples, nearest-rank. `samples` is sorted
/// in place by the caller.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = ((p * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

#[derive(Default)]
struct ModeScore {
    hits: usize,
    rr_sum: f64,
    misses: Vec<String>,
    /// End-to-end query latencies in milliseconds, one per timed repetition.
    latencies: Vec<f64>,
}

impl ModeScore {
    fn record(&mut self, query: &str, rank: Option<usize>) {
        match rank {
            Some(r) => {
                self.hits += 1;
                self.rr_sum += 1.0 / r as f64;
            }
            None => self.misses.push(query.to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();

    let golden: GoldenSet = serde_json::from_str(
        &std::fs::read_to_string(&args.queries)
            .with_context(|| format!("cannot read golden set at {}", args.queries.display()))?,
    )
    .context("golden set is not valid JSON")?;

    let tantivy_dir = args.work.join("tantivy");
    let vector_dir = args.work.join("vectors");
    if !args.reuse && args.work.exists() {
        std::fs::remove_dir_all(&args.work).context("failed to wipe the previous eval index")?;
    }
    std::fs::create_dir_all(&tantivy_dir)?;

    let config = SemanticSearchConfig {
        enabled: true,
        model: args.model.clone(),
        cache_dir: "target/fastembed-cache".to_string(),
        vector_store_dir: vector_dir.to_string_lossy().to_string(),
        chunk_max_lines: 45,
        chunk_overlap_lines: 10,
        batch_size: 32,
        queue_capacity: 512,
    };

    println!("Loading {} ...", args.model);
    let provider = Arc::new(FastEmbedProvider::try_new(&config)?);
    let embedder: Arc<dyn EmbeddingProvider> = provider.clone();
    let store = Arc::new(LanceVectorStore::open(&config.vector_store_dir, embedder.dimension()).await?);
    let search = SearchService::new(&tantivy_dir)?;
    let indexer = VectorIndexer::start(
        embedder.clone(),
        store,
        ChunkOptions { max_lines: config.chunk_max_lines, overlap_lines: config.chunk_overlap_lines },
        config.batch_size,
        config.queue_capacity,
    );

    if args.reuse && indexer.count().await.unwrap_or(0) > 0 {
        println!(
            "Reusing the existing index ({} chunks)\n",
            indexer.count().await.unwrap_or(0)
        );
    } else {
        let files = collect_files(&args.repo)?;
        println!("Indexing {} files from {} ...", files.len(), args.repo.display());
        build_index(&search, &indexer, &files).await?;
    }

    let modes = [("keyword", SearchMode::Keyword), ("semantic", SearchMode::Semantic), ("hybrid", SearchMode::Hybrid)];
    let mut scores: Vec<(&str, ModeScore)> = modes.iter().map(|(n, _)| (*n, ModeScore::default())).collect();

    // Top hits for queries nothing found, so a reviewer can tell a weak engine
    // from a bad golden entry without re-running anything.
    let mut unexplained: Vec<(String, Vec<String>)> = Vec::new();

    println!("{:<58} {:>8} {:>9} {:>7}", "query", "keyword", "semantic", "hybrid");
    for golden_query in &golden.queries {
        let mut ranks = Vec::new();
        let mut last_paths = Vec::new();
        for (slot, (_, mode)) in modes.iter().enumerate() {
            let mut query = SearchQuery::new(golden_query.query.clone());
            query.limit = TOP_K;
            query.mode = *mode;

            // One untimed run warms whatever the first call would pay for
            // (page cache, ANN index load), then `repeats` timed ones. A
            // semantic query embeds the question, so every one of them pays a
            // model forward pass the keyword path does not.
            let run = async |q: SearchQuery| {
                if mode.needs_semantic() {
                    semantic_search(&search, &embedder, &indexer, q).await
                } else {
                    search.search(q).await
                }
            };

            let found = run(query.clone()).await?;
            for _ in 0..args.repeats {
                let started = std::time::Instant::now();
                run(query.clone()).await?;
                scores[slot].1.latencies.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            let paths: Vec<String> = found.results.iter().map(|r| r.file_path.clone()).collect();
            let rank = rank_of_expected(&paths, &golden_query.expect);
            scores[slot].1.record(&golden_query.query, rank);
            ranks.push(rank);
            last_paths = paths;
        }

        if ranks.iter().all(|r| r.is_none()) {
            unexplained.push((golden_query.query.clone(), last_paths.into_iter().take(3).collect()));
        }

        let cell = |r: Option<usize>| r.map(|r| format!("#{r}")).unwrap_or_else(|| "-".to_string());
        let truncated: String = golden_query.query.chars().take(56).collect();
        println!(
            "{:<58} {:>8} {:>9} {:>7}",
            truncated,
            cell(ranks[0]),
            cell(ranks[1]),
            cell(ranks[2])
        );
    }

    let n = golden.queries.len() as f64;
    println!(
        "\n{:<10} {:>10} {:>8} {:>9} {:>9} {:>9}",
        "mode", "recall@10", "MRR", "p50 ms", "p95 ms", "max ms"
    );
    for (name, score) in &mut scores {
        score.latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        println!(
            "{:<10} {:>10.2} {:>8.2} {:>9.0} {:>9.0} {:>9.0}",
            name,
            score.hits as f64 / n,
            score.rr_sum / n,
            percentile(&score.latencies, 0.50),
            percentile(&score.latencies, 0.95),
            score.latencies.last().copied().unwrap_or(0.0)
        );
    }
    println!(
        "({} timed runs per mode: {} queries x {} repetitions)",
        scores[0].1.latencies.len(),
        golden.queries.len(),
        args.repeats
    );

    if !unexplained.is_empty() {
        println!("\nMissed by every mode, with what the last mode actually returned.");
        println!("A plausible answer here means the golden entry is too narrow, not that retrieval failed:");
        for (query, top) in &unexplained {
            println!("  - {query}");
            for path in top {
                println!("      {path}");
            }
        }
    }

    Ok(())
}
