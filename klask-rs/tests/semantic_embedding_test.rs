//! Integration tests for the semantic-search embedding provider.
//!
//! These tests load a real ONNX model (downloaded on first run into
//! target/fastembed-cache) and are therefore `#[ignore]`d by default:
//!
//! ```bash
//! cargo test --features semantic-search --test semantic_embedding_test -- --ignored
//! ```
#![cfg(feature = "semantic-search")]

use klask_rs::config::SemanticSearchConfig;
use klask_rs::services::semantic::chunker::{ChunkOptions, chunk_file};
use klask_rs::services::semantic::embedder::{EmbeddingProvider, FastEmbedProvider};

/// Small model (384 dims, ~30 MB) to keep the test download reasonable; the
/// production default is jinaai/jina-embeddings-v2-base-code (768 dims).
fn test_config() -> SemanticSearchConfig {
    SemanticSearchConfig {
        enabled: true,
        model: "Xenova/bge-small-en-v1.5".to_string(),
        cache_dir: "target/fastembed-cache".to_string(),
        chunk_max_lines: 60,
        chunk_overlap_lines: 15,
        batch_size: 32,
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

#[test]
#[ignore = "Downloads the embedding model on first run"]
fn test_embeddings_capture_code_semantics() {
    let provider = FastEmbedProvider::try_new(&test_config()).expect("model should load");
    assert_eq!(provider.dimension(), 384);
    assert_eq!(provider.model_id(), "Xenova/bge-small-en-v1.5");

    let texts = vec![
        // Two semantically close snippets about JWT validation…
        "fn validate_jwt_token(token: &str) -> bool { decode_and_check_signature(token) }".to_string(),
        "function verifyAuthToken(jwt) { return checkSignatureAndExpiry(jwt); }".to_string(),
        // …and one unrelated snippet
        "SELECT AVG(price) FROM products GROUP BY category ORDER BY 1 DESC;".to_string(),
    ];

    let vectors = provider.embed(&texts).expect("embedding should succeed");
    assert_eq!(vectors.len(), 3);
    for v in &vectors {
        assert_eq!(v.len(), provider.dimension());
    }

    let similar = cosine(&vectors[0], &vectors[1]);
    let dissimilar_a = cosine(&vectors[0], &vectors[2]);
    let dissimilar_b = cosine(&vectors[1], &vectors[2]);

    println!("similar={similar:.3} dissimilar_a={dissimilar_a:.3} dissimilar_b={dissimilar_b:.3}");
    assert!(
        similar > dissimilar_a && similar > dissimilar_b,
        "semantically related snippets must be closer than unrelated ones \
         (similar={similar:.3}, dissimilar={dissimilar_a:.3}/{dissimilar_b:.3})"
    );
}

#[test]
#[ignore = "Downloads the embedding model on first run"]
fn test_empty_batch_returns_empty() {
    let provider = FastEmbedProvider::try_new(&test_config()).expect("model should load");
    let vectors = provider.embed(&[]).expect("empty batch should succeed");
    assert!(vectors.is_empty());
}

/// Phase 1 benchmark (docs/SEMANTIC_SEARCH_PLAN.md §8): chunk a synthetic
/// source file and measure embedding throughput on this machine.
#[test]
#[ignore = "Benchmark; downloads the embedding model on first run"]
fn test_embedding_throughput_benchmark() {
    let provider = FastEmbedProvider::try_new(&test_config()).expect("model should load");

    // ~1200 lines of synthetic code → ~25 chunks with the default options
    let content = (0..200)
        .map(|i| format!("fn function_{i}(input: &str) -> Result<String> {{\n    let parsed = parse(input)?;\n    let validated = validate(&parsed)?;\n    Ok(render(validated))\n}}\n"))
        .collect::<Vec<_>>()
        .join("\n");
    let chunks = chunk_file("src/synthetic.rs", &content, &ChunkOptions::default());
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    assert!(
        texts.len() >= 10,
        "benchmark needs a meaningful batch, got {}",
        texts.len()
    );

    // Warm-up run, then timed run
    provider.embed(&texts[..2.min(texts.len())]).expect("warm-up should succeed");
    let started = std::time::Instant::now();
    let vectors = provider.embed(&texts).expect("embedding should succeed");
    let elapsed = started.elapsed();

    assert_eq!(vectors.len(), texts.len());
    println!(
        "Embedded {} chunks in {:.2}s → {:.1} chunks/s (model {}, dim {})",
        texts.len(),
        elapsed.as_secs_f64(),
        texts.len() as f64 / elapsed.as_secs_f64(),
        provider.model_id(),
        provider.dimension()
    );
}
