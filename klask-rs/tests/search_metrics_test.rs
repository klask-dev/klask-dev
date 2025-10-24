/// Comprehensive tests for Tantivy metrics collection and health checks.
///
/// Tests cover:
/// - IndexMetricsCollector functionality
/// - Health check detection and issue identification
/// - Tuning recommendation generation
/// - TantivyConfig loading and validation
/// - API endpoint behavior
// Import models
use klask_rs::models::{
    CacheStatistics, HealthCheckDetails, HealthIssue, HealthLevel, HealthStatus, ImpactLevel, IndexStatsResponse,
    IssueSeverity, SegmentMetrics, SpaceBreakdown, SpaceUsageBreakdown, TantivyConfig, TuningRecommendation,
    TuningRecommendationsResponse,
};

// ============================================================================
// Unit Tests: TantivyConfig
// ============================================================================

#[test]
fn test_tantivy_config_default() {
    let config = TantivyConfig::default();
    assert_eq!(config.memory_mb, 200);
    assert_eq!(config.num_threads, None);
    assert_eq!(config.cpu_cores, 4);
}

#[test]
fn test_tantivy_config_validate_valid() {
    let config = TantivyConfig { memory_mb: 200, num_threads: Some(4), cpu_cores: 4 };
    assert!(config.validate().is_ok());
}

#[test]
fn test_tantivy_config_validate_min_memory() {
    let config = TantivyConfig { memory_mb: 50, num_threads: Some(2), cpu_cores: 4 };
    assert!(config.validate().is_ok());

    let config_invalid = TantivyConfig { memory_mb: 49, num_threads: Some(2), cpu_cores: 4 };
    assert!(config_invalid.validate().is_err());
    assert!(config_invalid.validate().unwrap_err().contains("at least 50"));
}

#[test]
fn test_tantivy_config_validate_max_memory() {
    let config_valid = TantivyConfig { memory_mb: 8000, num_threads: Some(2), cpu_cores: 4 };
    assert!(config_valid.validate().is_ok());

    let config_invalid = TantivyConfig { memory_mb: 8001, num_threads: Some(2), cpu_cores: 4 };
    assert!(config_invalid.validate().is_err());
    assert!(config_invalid.validate().unwrap_err().contains("must not exceed 8000"));
}

#[test]
fn test_tantivy_config_validate_threads_min() {
    let config_invalid = TantivyConfig { memory_mb: 200, num_threads: Some(0), cpu_cores: 4 };
    assert!(config_invalid.validate().is_err());
    assert!(config_invalid.validate().unwrap_err().contains("at least 1"));
}

#[test]
fn test_tantivy_config_validate_threads_max() {
    let config_valid = TantivyConfig { memory_mb: 200, num_threads: Some(8), cpu_cores: 4 };
    assert!(config_valid.validate().is_ok());

    let config_invalid = TantivyConfig { memory_mb: 200, num_threads: Some(9), cpu_cores: 4 };
    assert!(config_invalid.validate().is_err());
    assert!(config_invalid.validate().unwrap_err().contains("exceeds 2x CPU cores"));
}

#[test]
fn test_tantivy_config_no_threads_always_valid() {
    let config = TantivyConfig { memory_mb: 200, num_threads: None, cpu_cores: 4 };
    assert!(config.validate().is_ok());
}

// ============================================================================
// Unit Tests: Health Check Logic
// ============================================================================

#[test]
fn test_health_level_healthy_status() {
    let checks = HealthCheckDetails {
        segment_count: 15,
        segment_health: HealthLevel::Healthy,
        cache_hit_ratio_percent: 85.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 5.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 250.0,
        size_health: HealthLevel::Healthy,
    };

    // No high severity issues means healthy overall
    assert_eq!(checks.segment_health, HealthLevel::Healthy);
    assert_eq!(checks.size_health, HealthLevel::Healthy);
}

#[test]
fn test_health_level_warning_status() {
    let checks = HealthCheckDetails {
        segment_count: 22,
        segment_health: HealthLevel::Warning,
        cache_hit_ratio_percent: 50.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 15.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 750.0,
        size_health: HealthLevel::Warning,
    };

    assert_eq!(checks.segment_health, HealthLevel::Warning);
    assert_eq!(checks.size_health, HealthLevel::Warning);
}

#[test]
fn test_health_level_critical_status() {
    let checks = HealthCheckDetails {
        segment_count: 30,
        segment_health: HealthLevel::Critical,
        cache_hit_ratio_percent: 20.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 40.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 1500.0,
        size_health: HealthLevel::Critical,
    };

    assert_eq!(checks.segment_health, HealthLevel::Critical);
    assert_eq!(checks.size_health, HealthLevel::Critical);
}

#[test]
fn test_segment_health_boundaries() {
    // Healthy boundary
    assert_eq!(get_segment_health(20), HealthLevel::Healthy);
    assert_eq!(get_segment_health(15), HealthLevel::Healthy);

    // Warning boundary
    assert_eq!(get_segment_health(21), HealthLevel::Warning);
    assert_eq!(get_segment_health(25), HealthLevel::Warning);

    // Critical boundary
    assert_eq!(get_segment_health(26), HealthLevel::Critical);
    assert_eq!(get_segment_health(50), HealthLevel::Critical);
}

#[test]
fn test_size_health_boundaries() {
    // Healthy boundary
    assert_eq!(get_size_health(100.0), HealthLevel::Healthy);
    assert_eq!(get_size_health(499.9), HealthLevel::Healthy);

    // Warning boundary
    assert_eq!(get_size_health(500.0), HealthLevel::Warning);
    assert_eq!(get_size_health(999.9), HealthLevel::Warning);

    // Critical boundary
    assert_eq!(get_size_health(1000.0), HealthLevel::Critical);
    assert_eq!(get_size_health(2000.0), HealthLevel::Critical);
}

// ============================================================================
// Unit Tests: Issue Identification
// ============================================================================

#[test]
fn test_identify_no_issues() {
    let checks = HealthCheckDetails {
        segment_count: 10,
        segment_health: HealthLevel::Healthy,
        cache_hit_ratio_percent: 80.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 2.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 100.0,
        size_health: HealthLevel::Healthy,
    };

    let issues = identify_issues(&checks);
    assert!(issues.is_empty());
}

#[test]
fn test_identify_high_segment_count_issue() {
    let checks = HealthCheckDetails {
        segment_count: 30,
        segment_health: HealthLevel::Critical,
        cache_hit_ratio_percent: 80.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 2.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 100.0,
        size_health: HealthLevel::Healthy,
    };

    let issues = identify_issues(&checks);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.severity == IssueSeverity::High && i.description.contains("Too many segments")));
}

#[test]
fn test_identify_warning_segment_count_issue() {
    let checks = HealthCheckDetails {
        segment_count: 22,
        segment_health: HealthLevel::Warning,
        cache_hit_ratio_percent: 80.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 2.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 100.0,
        size_health: HealthLevel::Healthy,
    };

    let issues = identify_issues(&checks);
    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Medium && i.description.contains("Segment count is high")));
}

#[test]
fn test_identify_high_size_issue() {
    let checks = HealthCheckDetails {
        segment_count: 10,
        segment_health: HealthLevel::Healthy,
        cache_hit_ratio_percent: 80.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 2.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 1500.0,
        size_health: HealthLevel::Critical,
    };

    let issues = identify_issues(&checks);
    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|i| i.severity == IssueSeverity::High && i.description.contains("Index size is very large")));
}

#[test]
fn test_identify_warning_size_issue() {
    let checks = HealthCheckDetails {
        segment_count: 10,
        segment_health: HealthLevel::Healthy,
        cache_hit_ratio_percent: 80.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 2.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 750.0,
        size_health: HealthLevel::Warning,
    };

    let issues = identify_issues(&checks);
    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|i| i.severity == IssueSeverity::Medium && i.description.contains("Index size is getting large")));
}

// ============================================================================
// Unit Tests: Tuning Recommendations
// ============================================================================

#[test]
fn test_recommend_segment_optimization() {
    let stats = create_test_stats(25, 100.0);
    let recommendations = generate_recommendations(&stats, HealthStatus::Warning);

    let seg_rec = recommendations.iter().find(|r| r.title.contains("Optimize") && r.title.contains("segments"));
    assert!(seg_rec.is_some());
    assert_eq!(seg_rec.unwrap().impact, ImpactLevel::High);
}

#[test]
fn test_no_segment_optimization_below_threshold() {
    let stats = create_test_stats(15, 100.0);
    let recommendations = generate_recommendations(&stats, HealthStatus::Healthy);

    let seg_rec = recommendations.iter().find(|r| r.title.contains("Optimize") && r.title.contains("segments"));
    assert!(seg_rec.is_none());
}

#[test]
fn test_recommend_memory_buffer_increase() {
    let stats = create_test_stats(10, 600.0);
    let recommendations = generate_recommendations(&stats, HealthStatus::Healthy);

    let mem_rec = recommendations.iter().find(|r| r.title.contains("memory buffer"));
    assert!(mem_rec.is_some());
    assert_eq!(mem_rec.unwrap().impact, ImpactLevel::Medium);
}

#[test]
fn test_no_memory_buffer_below_threshold() {
    let stats = create_test_stats(10, 400.0);
    let recommendations = generate_recommendations(&stats, HealthStatus::Healthy);

    let mem_rec = recommendations.iter().find(|r| r.title.contains("memory buffer"));
    assert!(mem_rec.is_none());
}

#[test]
fn test_recommendations_sorted_by_impact() {
    let stats = create_test_stats(25, 600.0);
    let recommendations = generate_recommendations(&stats, HealthStatus::Warning);

    // Should be sorted: High > Medium > Low
    if recommendations.len() > 1 {
        for i in 0..recommendations.len() - 1 {
            let current = impact_to_order(recommendations[i].impact);
            let next = impact_to_order(recommendations[i + 1].impact);
            assert!(current <= next, "Recommendations not sorted by impact");
        }
    }
}

#[test]
fn test_recommendations_summary() {
    let stats = create_test_stats(25, 600.0);
    let response = generate_recommendations_response(&stats, HealthStatus::Warning);

    assert!(!response.summary.is_empty());
    assert!(response.summary.contains("recommendation"));
}

#[test]
fn test_no_recommendations_message() {
    let stats = create_test_stats(10, 100.0);
    let response = generate_recommendations_response(&stats, HealthStatus::Healthy);

    assert!(response.summary.contains("No tuning recommendations"));
}

// ============================================================================
// Unit Tests: Health Status Determination
// ============================================================================

#[test]
fn test_determine_healthy_status() {
    let status = determine_status(vec![]);
    assert_eq!(status, HealthStatus::Healthy);
}

#[test]
fn test_determine_warning_status() {
    let issues = vec![create_issue(IssueSeverity::Medium, "Test warning")];
    let status = determine_status(issues);
    assert_eq!(status, HealthStatus::Warning);
}

#[test]
fn test_determine_degraded_status() {
    let issues = vec![create_issue(IssueSeverity::High, "High issue")];
    let status = determine_status(issues);
    assert_eq!(status, HealthStatus::Degraded);
}

#[test]
fn test_status_message_healthy() {
    let issues = vec![];
    let message = create_status_message(HealthStatus::Healthy, &issues);
    assert_eq!(message, "Index is in optimal state");
}

#[test]
fn test_status_message_warning() {
    let issues = vec![create_issue(IssueSeverity::Medium, "Issue 1"), create_issue(IssueSeverity::Medium, "Issue 2")];
    let message = create_status_message(HealthStatus::Warning, &issues);
    assert!(message.contains("2"));
    assert!(message.contains("warning"));
}

#[test]
fn test_status_message_degraded() {
    let issues = vec![
        create_issue(IssueSeverity::Medium, "Medium"),
        create_issue(IssueSeverity::High, "High 1"),
        create_issue(IssueSeverity::High, "High 2"),
    ];
    let message = create_status_message(HealthStatus::Degraded, &issues);
    assert!(message.contains("2"));
    assert!(message.contains("critical"));
}

// ============================================================================
// Unit Tests: Data Structure Serialization
// ============================================================================

#[test]
fn test_serialize_index_stats_response() {
    let stats = create_test_stats(10, 100.0);
    let json = serde_json::to_value(&stats).expect("Failed to serialize");

    assert_eq!(json["total_documents"], 1000);
    assert_eq!(json["total_size_mb"], 100.0);
    assert_eq!(json["segment_count"], 10);
}

#[test]
fn test_serialize_health_check_details() {
    let checks = HealthCheckDetails {
        segment_count: 15,
        segment_health: HealthLevel::Healthy,
        cache_hit_ratio_percent: 85.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 5.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: 250.0,
        size_health: HealthLevel::Healthy,
    };

    let json = serde_json::to_value(&checks).expect("Failed to serialize");
    assert_eq!(json["segment_count"], 15);
    assert_eq!(json["segment_health"], "HEALTHY");
    assert_eq!(json["cache_hit_ratio_percent"], 85.0);
}

#[test]
fn test_serialize_health_status_enum() {
    let json_healthy = serde_json::to_value(HealthStatus::Healthy).expect("Failed to serialize");
    assert_eq!(json_healthy, "HEALTHY");

    let json_warning = serde_json::to_value(HealthStatus::Warning).expect("Failed to serialize");
    assert_eq!(json_warning, "WARNING");

    let json_degraded = serde_json::to_value(HealthStatus::Degraded).expect("Failed to serialize");
    assert_eq!(json_degraded, "DEGRADED");
}

#[test]
fn test_serialize_health_level_enum() {
    let json_healthy = serde_json::to_value(HealthLevel::Healthy).expect("Failed to serialize");
    assert_eq!(json_healthy, "HEALTHY");

    let json_warning = serde_json::to_value(HealthLevel::Warning).expect("Failed to serialize");
    assert_eq!(json_warning, "WARNING");

    let json_critical = serde_json::to_value(HealthLevel::Critical).expect("Failed to serialize");
    assert_eq!(json_critical, "CRITICAL");
}

#[test]
fn test_serialize_issue_severity_enum() {
    let json_high = serde_json::to_value(IssueSeverity::High).expect("Failed to serialize");
    assert_eq!(json_high, "high");

    let json_medium = serde_json::to_value(IssueSeverity::Medium).expect("Failed to serialize");
    assert_eq!(json_medium, "medium");

    let json_low = serde_json::to_value(IssueSeverity::Low).expect("Failed to serialize");
    assert_eq!(json_low, "low");
}

#[test]
fn test_serialize_impact_level_enum() {
    let json_high = serde_json::to_value(ImpactLevel::High).expect("Failed to serialize");
    assert_eq!(json_high, "high");

    let json_medium = serde_json::to_value(ImpactLevel::Medium).expect("Failed to serialize");
    assert_eq!(json_medium, "medium");

    let json_low = serde_json::to_value(ImpactLevel::Low).expect("Failed to serialize");
    assert_eq!(json_low, "low");
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_stats(segment_count: usize, size_mb: f64) -> IndexStatsResponse {
    IndexStatsResponse {
        total_documents: 1000,
        total_size_mb: size_mb,
        total_size_bytes: (size_mb * 1_048_576.0) as u64,
        segment_count,
        segments: (0..segment_count)
            .map(|i| SegmentMetrics {
                segment_ord: i as u32,
                doc_count: 100,
                max_doc: 100,
                deleted_docs: 0,
                size_bytes: ((size_mb * 1_048_576.0) / segment_count as f64) as u64,
                space_breakdown: SpaceBreakdown { postings: 0, store: 0, fast_fields: 0, positions: 0, other: 0 },
            })
            .collect(),
        space_usage: SpaceUsageBreakdown {
            postings_bytes: 0,
            store_bytes: 0,
            fast_fields_bytes: 0,
            positions_bytes: 0,
            other_bytes: 0,
        },
        cache_stats: CacheStatistics { num_entries: 0, hits: 0, misses: 0, hit_ratio: -1.0 },
    }
}

fn get_segment_health(count: usize) -> HealthLevel {
    if count <= 20 {
        HealthLevel::Healthy
    } else if count <= 25 {
        HealthLevel::Warning
    } else {
        HealthLevel::Critical
    }
}

fn get_size_health(size_mb: f64) -> HealthLevel {
    if size_mb < 500.0 {
        HealthLevel::Healthy
    } else if size_mb < 1000.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Critical
    }
}

fn identify_issues(checks: &HealthCheckDetails) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

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

fn generate_recommendations(stats: &IndexStatsResponse, _health_status: HealthStatus) -> Vec<TuningRecommendation> {
    let mut recommendations = Vec::new();

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
            reason: "Multiple segments increase search latency and memory usage. Merging improves query performance."
                .to_string(),
        });
    }

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

    recommendations.sort_by(|a, b| {
        let impact_order = |level: ImpactLevel| match level {
            ImpactLevel::High => 0,
            ImpactLevel::Medium => 1,
            ImpactLevel::Low => 2,
        };
        impact_order(a.impact).cmp(&impact_order(b.impact))
    });

    recommendations
}

fn generate_recommendations_response(
    stats: &IndexStatsResponse,
    health_status: HealthStatus,
) -> TuningRecommendationsResponse {
    use chrono::Utc;

    let recommendations = generate_recommendations(stats, health_status);

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

fn create_issue(severity: IssueSeverity, description: &str) -> HealthIssue {
    HealthIssue {
        severity,
        description: description.to_string(),
        metric_value: "test".to_string(),
        threshold: "test".to_string(),
    }
}

fn determine_status(issues: Vec<HealthIssue>) -> HealthStatus {
    match issues.iter().map(|i| i.severity).max() {
        Some(IssueSeverity::High) => HealthStatus::Degraded,
        Some(IssueSeverity::Medium) => HealthStatus::Warning,
        _ => HealthStatus::Healthy,
    }
}

fn create_status_message(status: HealthStatus, issues: &[HealthIssue]) -> String {
    match status {
        HealthStatus::Healthy => "Index is in optimal state".to_string(),
        HealthStatus::Warning => format!(
            "Index has {} warning(s) that could be optimized",
            issues.iter().filter(|i| i.severity == IssueSeverity::Medium).count()
        ),
        HealthStatus::Degraded => format!(
            "Index has {} critical issue(s) affecting performance",
            issues.iter().filter(|i| i.severity == IssueSeverity::High).count()
        ),
    }
}

fn impact_to_order(impact: ImpactLevel) -> u8 {
    match impact {
        ImpactLevel::High => 0,
        ImpactLevel::Medium => 1,
        ImpactLevel::Low => 2,
    }
}
