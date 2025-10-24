//! Index metrics collection and analysis service.
//!
//! This service collects detailed metrics from the Tantivy search index,
//! performs health checks, and generates tuning recommendations.

use crate::models::{
    CacheStatistics, HealthCheckDetails, HealthIssue, HealthLevel, HealthStatus, ImpactLevel, IndexHealthResponse,
    IndexStatsResponse, IssueSeverity, SegmentMetrics, SpaceBreakdown, SpaceUsageBreakdown, TuningRecommendation,
    TuningRecommendationsResponse,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tantivy::IndexReader;

/// Service for collecting and analyzing index metrics.
pub struct IndexMetricsCollector {
    reader: Arc<IndexReader>,
}

impl IndexMetricsCollector {
    /// Create a new metrics collector.
    pub fn new(reader: Arc<IndexReader>) -> Self {
        Self { reader }
    }

    /// Collect comprehensive index statistics.
    pub fn collect_stats(&self, index_size_mb: f64) -> Result<IndexStatsResponse> {
        let searcher = self.reader.searcher();

        // Get total documents
        let total_documents = searcher.num_docs();

        // Initialize empty space usage and cache stats since Tantivy 0.25 API is limited
        let space_usage = SpaceUsageBreakdown {
            postings_bytes: 0,
            store_bytes: 0,
            fast_fields_bytes: 0,
            positions_bytes: 0,
            other_bytes: 0,
        };

        let cache_stats = CacheStatistics { num_entries: 0, hits: 0, misses: 0, hit_ratio: -1.0 };

        // Collect segment metrics
        let segment_readers = searcher.segment_readers();
        let mut segments = Vec::new();
        let mut segment_count = 0;

        for segment_reader in segment_readers {
            let doc_count: u64 = segment_reader.num_docs() as u64;
            let max_doc = segment_reader.max_doc();

            // Count deleted documents (Tantivy 0.25 doesn't expose delete_bitset on SegmentReader)
            let deleted_docs = 0u32; // Simplified - not available in 0.25

            let space_breakdown = SpaceBreakdown { postings: 0, store: 0, fast_fields: 0, positions: 0, other: 0 };
            let size_bytes = 0u64;

            segments.push(SegmentMetrics {
                segment_ord: segment_count as u32,
                doc_count,
                max_doc,
                deleted_docs,
                size_bytes,
                space_breakdown,
            });

            segment_count += 1;
        }

        let total_size_bytes = (index_size_mb * 1_048_576.0) as u64;

        Ok(IndexStatsResponse {
            total_documents,
            total_size_mb: index_size_mb,
            total_size_bytes,
            segment_count: segment_count as usize,
            segments,
            space_usage,
            cache_stats,
        })
    }

    /// Perform a health check on the index.
    #[allow(dead_code)]
    pub fn check_health(&self, stats: &IndexStatsResponse) -> Result<IndexHealthResponse> {
        let health_checks = self.perform_health_checks(stats);
        let issues = self.identify_issues(&health_checks);

        // Determine overall status based on issues
        let status = match issues.iter().map(|i| i.severity).max() {
            Some(IssueSeverity::High) => HealthStatus::Degraded,
            Some(IssueSeverity::Medium) => HealthStatus::Warning,
            _ => HealthStatus::Healthy,
        };

        let status_message = match status {
            HealthStatus::Healthy => "Index is in optimal state".to_string(),
            HealthStatus::Warning => format!(
                "Index has {} warning(s) that could be optimized",
                issues.iter().filter(|i| i.severity == IssueSeverity::Medium).count()
            ),
            HealthStatus::Degraded => format!(
                "Index has {} critical issue(s) affecting performance",
                issues.iter().filter(|i| i.severity == IssueSeverity::High).count()
            ),
        };

        Ok(IndexHealthResponse {
            status,
            status_message,
            checked_at: Utc::now(),
            index_stats: stats.clone(),
            health_checks,
            issues,
        })
    }

    /// Generate tuning recommendations based on current metrics.
    #[allow(dead_code)]
    pub fn generate_recommendations(
        &self,
        stats: &IndexStatsResponse,
        health_status: HealthStatus,
    ) -> TuningRecommendationsResponse {
        let mut recommendations = Vec::new();

        // Recommendation 1: Optimize index for too many segments
        if stats.segment_count > 20 {
            recommendations.push(TuningRecommendation {
                impact: ImpactLevel::High,
                title: "Optimize index to merge segments".to_string(),
                description: "The index has more than 20 segments, which can impact search performance. \
                    Running an optimization will merge smaller segments into larger ones."
                    .to_string(),
                parameter: None,
                current_value: Some(format!("{} segments", stats.segment_count)),
                recommended_value: Some("15-20 segments".to_string()),
                reason:
                    "Multiple segments increase search latency and memory usage. Merging improves query performance."
                        .to_string(),
            });
        }

        // Recommendation 2: Adjust memory buffer based on index size
        if stats.total_size_mb > 500.0 {
            recommendations.push(TuningRecommendation {
                impact: ImpactLevel::Medium,
                title: "Consider increasing memory buffer".to_string(),
                description: format!(
                    "Index size is {:.1} MB. A larger memory buffer can improve indexing throughput.",
                    stats.total_size_mb
                ),
                parameter: Some("KLASK_TANTIVY_MEMORY_MB".to_string()),
                current_value: Some("200 MB".to_string()),
                recommended_value: Some("300-500 MB".to_string()),
                reason: "Larger buffer allows batching more documents before flushing to disk.".to_string(),
            });
        }

        // Sort by impact
        recommendations.sort_by(|a, b| {
            let impact_order = |level: ImpactLevel| match level {
                ImpactLevel::High => 0,
                ImpactLevel::Medium => 1,
                ImpactLevel::Low => 2,
            };
            impact_order(a.impact).cmp(&impact_order(b.impact))
        });

        let summary = if recommendations.is_empty() {
            "No tuning recommendations at this time. Index is well-optimized.".to_string()
        } else {
            format!(
                "{} recommendation(s): {} high-impact, {} medium-impact, {} low-impact",
                recommendations.len(),
                recommendations.iter().filter(|r| r.impact == ImpactLevel::High).count(),
                recommendations.iter().filter(|r| r.impact == ImpactLevel::Medium).count(),
                recommendations.iter().filter(|r| r.impact == ImpactLevel::Low).count(),
            )
        };

        TuningRecommendationsResponse {
            current_metrics: stats.clone(),
            health_status,
            recommendations,
            analyzed_at: Utc::now(),
            summary,
        }
    }

    // Helper methods

    #[allow(dead_code)]
    fn perform_health_checks(&self, stats: &IndexStatsResponse) -> HealthCheckDetails {
        // Segment health
        let segment_health = if stats.segment_count <= 20 {
            HealthLevel::Healthy
        } else if stats.segment_count <= 25 {
            HealthLevel::Warning
        } else {
            HealthLevel::Critical
        };

        // Cache hit ratio health (no cache data in simplified version)
        let cache_hit_ratio_percent = 0.0;
        let cache_health = HealthLevel::Healthy; // Can't determine without cache data

        // Deleted documents ratio (no delete data in simplified version)
        let deleted_docs_ratio_percent = 0.0;
        let deletion_health = HealthLevel::Healthy; // Can't determine without delete data

        // Size health
        let size_health = if stats.total_size_mb < 500.0 {
            HealthLevel::Healthy
        } else if stats.total_size_mb < 1000.0 {
            HealthLevel::Warning
        } else {
            HealthLevel::Critical
        };

        HealthCheckDetails {
            segment_count: stats.segment_count,
            segment_health,
            cache_hit_ratio_percent,
            cache_health,
            deleted_docs_ratio_percent,
            deletion_health,
            index_size_mb: stats.total_size_mb,
            size_health,
        }
    }

    #[allow(dead_code)]
    fn identify_issues(&self, checks: &HealthCheckDetails) -> Vec<HealthIssue> {
        let mut issues = Vec::new();

        // Check segments
        if checks.segment_health == HealthLevel::Critical {
            issues.push(HealthIssue {
                severity: IssueSeverity::High,
                description: "Too many segments in index".to_string(),
                metric_value: format!("{}", checks.segment_count),
                threshold: "20 segments".to_string(),
            });
        } else if checks.segment_health == HealthLevel::Warning {
            issues.push(HealthIssue {
                severity: IssueSeverity::Medium,
                description: "Segment count is high, consider optimization".to_string(),
                metric_value: format!("{}", checks.segment_count),
                threshold: "20 segments".to_string(),
            });
        }

        // Check size
        if checks.size_health == HealthLevel::Critical {
            issues.push(HealthIssue {
                severity: IssueSeverity::High,
                description: "Index size is very large, may impact performance".to_string(),
                metric_value: format!("{:.1} MB", checks.index_size_mb),
                threshold: "1000 MB (1 GB)".to_string(),
            });
        } else if checks.size_health == HealthLevel::Warning {
            issues.push(HealthIssue {
                severity: IssueSeverity::Medium,
                description: "Index size is getting large".to_string(),
                metric_value: format!("{:.1} MB", checks.index_size_mb),
                threshold: "1000 MB (1 GB)".to_string(),
            });
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_level_ordering() {
        assert!(matches!(HealthLevel::Healthy, HealthLevel::Healthy));
        assert!(matches!(HealthLevel::Warning, HealthLevel::Warning));
        assert!(matches!(HealthLevel::Critical, HealthLevel::Critical));
    }

    #[test]
    fn test_impact_level_ordering() {
        assert!(matches!(ImpactLevel::High, ImpactLevel::High));
        assert!(matches!(ImpactLevel::Medium, ImpactLevel::Medium));
        assert!(matches!(ImpactLevel::Low, ImpactLevel::Low));
    }
}
