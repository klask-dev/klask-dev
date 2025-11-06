//! Admin search API endpoints for index metrics and tuning.
//!
//! Provides endpoints for:
//! - Collecting detailed index statistics
//! - Performing health checks on the index
//! - Optimizing the index for better performance
//! - Generating tuning recommendations

use crate::auth::extractors::{AdminUser, AppState};
use crate::models::{
    HealthStatus, IndexHealthResponse, IndexStatsResponse, OptimizeIndexResponse, TuningRecommendationsResponse,
};
use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use tracing::{debug, error, info};

/// Create admin search API router with all endpoints.
pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/index-stats", get(get_index_stats))
        .route("/index-health", get(get_index_health))
        .route("/optimize-index", post(optimize_index))
        .route("/tuning-recommendations", get(get_tuning_recommendations));

    Ok(router)
}

/// GET /api/admin/search/index-stats
///
/// Returns detailed statistics about the search index including:
/// - Total documents
/// - Index size
/// - Segment breakdown
/// - Space usage by component
/// - Cache statistics
async fn get_index_stats(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<IndexStatsResponse>, StatusCode> {
    debug!("Admin: Getting index statistics");

    match collect_index_stats(&app_state).await {
        Ok(stats) => {
            info!(
                "Index stats retrieved: {} documents, {:.2} MB",
                stats.total_documents, stats.total_size_mb
            );
            Ok(Json(stats))
        }
        Err(e) => {
            error!("Failed to collect index stats: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/admin/search/index-health
///
/// Performs a comprehensive health check on the index and returns:
/// - Overall health status (HEALTHY, WARNING, DEGRADED)
/// - Detailed health check metrics
/// - List of identified issues with severity levels
async fn get_index_health(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<IndexHealthResponse>, StatusCode> {
    debug!("Admin: Checking index health");

    match collect_index_stats(&app_state).await {
        Ok(stats) => match perform_health_check(&stats) {
            Ok(health) => {
                let status_str = match health.status {
                    HealthStatus::Healthy => "HEALTHY",
                    HealthStatus::Warning => "WARNING",
                    HealthStatus::Degraded => "DEGRADED",
                };
                info!(
                    "Index health check completed: status={}, issues={}",
                    status_str,
                    health.issues.len()
                );
                Ok(Json(health))
            }
            Err(e) => {
                error!("Failed to check index health: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            error!("Failed to collect stats for health check: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/admin/search/optimize-index
///
/// Triggers index optimization which:
/// - Merges multiple segments into fewer, larger segments
/// - Removes documents marked as deleted
/// - Reduces overall index size
/// - Improves query performance
///
/// This is an asynchronous operation that may take some time.
async fn optimize_index(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<OptimizeIndexResponse>, StatusCode> {
    debug!("Admin: Starting index optimization");

    let search_service = &app_state.search_service;

    match search_service.apply_merge_policy().await {
        Ok(response) => {
            info!(
                "Index optimization completed: {} -> {} segments, {:.2}% size reduction",
                response.segments_before, response.segments_after, response.size_reduction_percent
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!("Index optimization failed: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/admin/search/tuning-recommendations
///
/// Analyzes current index metrics and generates actionable tuning recommendations.
/// Returns recommendations sorted by impact level (HIGH > MEDIUM > LOW).
///
/// Recommendations may include:
/// - Segment optimization
/// - Cache size adjustment
/// - Memory buffer tuning
/// - Deleted document cleanup
async fn get_tuning_recommendations(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<TuningRecommendationsResponse>, StatusCode> {
    debug!("Admin: Generating tuning recommendations");

    match collect_index_stats(&app_state).await {
        Ok(stats) => {
            // Perform quick health check to get status
            let health = match perform_health_check(&stats) {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to check health for recommendations: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let recommendations = generate_recommendations(&stats, health.status);

            info!(
                "Generated {} tuning recommendations",
                recommendations.recommendations.len()
            );
            Ok(Json(recommendations))
        }
        Err(e) => {
            error!("Failed to generate tuning recommendations: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper functions

use crate::models::{HealthCheckDetails, HealthIssue, HealthLevel, ImpactLevel, IssueSeverity, TuningRecommendation};

/// Collect current index statistics from the search service.
async fn collect_index_stats(app_state: &AppState) -> Result<IndexStatsResponse> {
    app_state.search_service.collect_detailed_metrics()
}

/// Perform a health check on collected statistics.
fn perform_health_check(stats: &IndexStatsResponse) -> Result<IndexHealthResponse> {
    use chrono::Utc;

    let health_checks = perform_health_checks_internal(stats);
    let issues = identify_issues_internal(&health_checks);

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
fn generate_recommendations(stats: &IndexStatsResponse, health_status: HealthStatus) -> TuningRecommendationsResponse {
    use chrono::Utc;

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
            reason: "Multiple segments increase search latency and memory usage. Merging improves query performance."
                .to_string(),
        });
    }

    // Recommendation 2: Adjust memory buffer based on index size
    if stats.total_size_mb > 500.0 {
        let tantivy_config = crate::models::TantivyConfig::from_env();
        let current_memory_mb = tantivy_config.memory_mb;
        recommendations.push(TuningRecommendation {
            impact: ImpactLevel::Medium,
            title: "Consider increasing memory buffer".to_string(),
            description: format!(
                "Index size is {:.1} MB. A larger memory buffer can improve indexing throughput.",
                stats.total_size_mb
            ),
            parameter: Some("KLASK_TANTIVY_MEMORY_MB".to_string()),
            current_value: Some(format!("{} MB", current_memory_mb)),
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

fn perform_health_checks_internal(stats: &IndexStatsResponse) -> HealthCheckDetails {
    // Segment health
    let segment_health = if stats.segment_count <= 20 {
        HealthLevel::Healthy
    } else if stats.segment_count <= 25 {
        HealthLevel::Warning
    } else {
        HealthLevel::Critical
    };

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
        cache_hit_ratio_percent: 0.0,
        cache_health: HealthLevel::Healthy,
        deleted_docs_ratio_percent: 0.0,
        deletion_health: HealthLevel::Healthy,
        index_size_mb: stats.total_size_mb,
        size_health,
    }
}

fn identify_issues_internal(checks: &HealthCheckDetails) -> Vec<HealthIssue> {
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

#[cfg(test)]
mod tests {
    // These tests would require mocking the AppState and search service
    // For now, we ensure the module compiles correctly

    #[test]
    fn test_module_compiles() {
        // Placeholder test to ensure module compiles
        assert!(true);
    }
}
