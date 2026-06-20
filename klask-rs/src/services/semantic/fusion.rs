// The fusion function is consumed by the hybrid query path, which is gated on
// the `semantic-search` feature; without that feature the module is unused, so
// silence the dead-code warning only in that build.
#![cfg_attr(not(feature = "semantic-search"), allow(dead_code))]

//! Reciprocal Rank Fusion (RRF) for hybrid search.
//!
//! Combines several ranked result lists (e.g. Tantivy BM25 and vector ANN)
//! into one ranking using only ranks, which sidesteps the problem of
//! normalizing incomparable scores (BM25 vs cosine similarity). See
//! docs/SEMANTIC_SEARCH_PLAN.md §3.4.

use std::collections::HashMap;
use std::hash::Hash;

/// Standard RRF dampening constant from the literature; larger values flatten
/// the contribution difference between top and deep ranks.
pub const DEFAULT_RRF_K: f32 = 60.0;

/// Fuse ranked lists with Reciprocal Rank Fusion.
///
/// Each input list is ordered best-first. An item's fused score is
/// `Σ 1 / (k + rank_i)` over the lists containing it (rank is 1-based).
/// Returns items ordered by fused score (descending), with ties broken by the
/// item key (ascending) so the output is deterministic.
pub fn reciprocal_rank_fusion<K>(rankings: &[Vec<K>], k: f32) -> Vec<(K, f32)>
where
    K: Eq + Hash + Ord + Clone,
{
    let mut scores: HashMap<K, f32> = HashMap::new();

    for ranking in rankings {
        for (index, item) in ranking.iter().enumerate() {
            let rank = (index + 1) as f32;
            *scores.entry(item.clone()).or_insert(0.0) += 1.0 / (k + rank);
        }
    }

    let mut fused: Vec<(K, f32)> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_in_both_lists_outranks_single_list_items() {
        let bm25 = vec!["a", "b", "c"];
        let vector = vec!["d", "b", "e"];
        let fused = reciprocal_rank_fusion(&[bm25, vector], DEFAULT_RRF_K);

        // "b" is rank 2 in both lists, every other item appears once
        assert_eq!(fused[0].0, "b");
    }

    #[test]
    fn test_single_list_preserves_order() {
        let only = vec!["x", "y", "z"];
        let fused = reciprocal_rank_fusion(&[only], DEFAULT_RRF_K);
        let order: Vec<&str> = fused.iter().map(|(k, _)| *k).collect();
        assert_eq!(order, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_empty_input() {
        let fused: Vec<(String, f32)> = reciprocal_rank_fusion(&[], DEFAULT_RRF_K);
        assert!(fused.is_empty());

        let fused: Vec<(String, f32)> = reciprocal_rank_fusion(&[vec![], vec![]], DEFAULT_RRF_K);
        assert!(fused.is_empty());
    }

    #[test]
    fn test_ties_break_deterministically() {
        // "a" and "b" both appear only at rank 1 of one list → equal scores
        let fused = reciprocal_rank_fusion(&[vec!["b"], vec!["a"]], DEFAULT_RRF_K);
        assert_eq!(fused[0].0, "a", "equal scores must order by key");
        assert_eq!(fused[1].0, "b");
        assert!((fused[0].1 - fused[1].1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scores_are_rrf_formula() {
        let fused = reciprocal_rank_fusion(&[vec!["a", "b"]], 60.0);
        let score_a = fused.iter().find(|(k, _)| *k == "a").unwrap().1;
        let score_b = fused.iter().find(|(k, _)| *k == "b").unwrap().1;
        assert!((score_a - 1.0 / 61.0).abs() < 1e-6);
        assert!((score_b - 1.0 / 62.0).abs() < 1e-6);
    }

    #[test]
    fn test_smaller_k_amplifies_top_ranks() {
        // With a small k, rank 1 in one list beats rank 3 in two lists;
        // with the default k, appearing in two lists wins.
        let lists = [vec!["solo", "x", "y", "both"], vec!["z", "w", "v", "both"]];
        let fused_small_k = reciprocal_rank_fusion(&lists, 0.5);
        let fused_default = reciprocal_rank_fusion(&lists, DEFAULT_RRF_K);

        assert_eq!(fused_small_k[0].0, "solo");
        assert_eq!(fused_default[0].0, "both");
    }
}
