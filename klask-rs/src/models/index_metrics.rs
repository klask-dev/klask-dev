use serde::{Deserialize, Serialize};

/// Response structure for detailed index statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatsResponse {
    /// Total number of documents in the index
    pub total_documents: u64,
    /// Total index size in megabytes
    pub total_size_mb: f64,
    /// Total index size in bytes
    pub total_size_bytes: u64,
    /// Number of segments in the index
    pub segment_count: usize,
    /// Metrics for each segment
    pub segments: Vec<SegmentMetrics>,
    /// Space usage breakdown by component
    pub space_usage: SpaceUsageBreakdown,
    /// Document cache statistics
    pub cache_stats: CacheStatistics,
}

/// Metrics for a single index segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMetrics {
    /// Segment ordinal (identifier)
    pub segment_ord: u32,
    /// Number of documents in this segment
    pub doc_count: u64,
    /// Maximum document ID in segment
    pub max_doc: u32,
    /// Number of deleted documents
    pub deleted_docs: u32,
    /// Size in bytes
    pub size_bytes: u64,
    /// Space usage breakdown for this segment
    pub space_breakdown: SpaceBreakdown,
}

/// Breakdown of index space usage by component type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceUsageBreakdown {
    /// Total space for postings (inverted index)
    pub postings_bytes: u64,
    /// Total space for stored fields (doc store)
    pub store_bytes: u64,
    /// Total space for fast fields
    pub fast_fields_bytes: u64,
    /// Total space for positions
    pub positions_bytes: u64,
    /// Total space for other components
    pub other_bytes: u64,
}

/// Detailed space breakdown for a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceBreakdown {
    /// Postings space in bytes
    pub postings: u64,
    /// Doc store space in bytes
    pub store: u64,
    /// Fast fields space in bytes
    pub fast_fields: u64,
    /// Positions space in bytes
    pub positions: u64,
    /// Other components in bytes
    pub other: u64,
}

/// Document store cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Number of documents in cache
    pub num_entries: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache hit ratio (0.0 to 1.0, or -1.0 if no data)
    pub hit_ratio: f64,
}

/// Health status enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus {
    /// Index is in optimal state
    Healthy,
    /// Index has minor issues that could be optimized
    Warning,
    /// Index has significant performance issues
    Degraded,
}

/// Detailed health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexHealthResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Explanation of the health status
    pub status_message: String,
    /// Timestamp of health check
    pub checked_at: chrono::DateTime<chrono::Utc>,
    /// Index statistics used for health assessment
    pub index_stats: IndexStatsResponse,
    /// Health check details
    pub health_checks: HealthCheckDetails,
    /// Issues found (if any)
    pub issues: Vec<HealthIssue>,
}

/// Details of health checks performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckDetails {
    /// Number of segments (target: <= 20)
    pub segment_count: usize,
    /// Is segment count healthy?
    pub segment_health: HealthLevel,
    /// Cache hit ratio percentage (0-100)
    pub cache_hit_ratio_percent: f64,
    /// Is cache health good?
    pub cache_health: HealthLevel,
    /// Ratio of deleted to total docs (0-100%)
    pub deleted_docs_ratio_percent: f64,
    /// Is deleted docs ratio acceptable?
    pub deletion_health: HealthLevel,
    /// Index size in MB
    pub index_size_mb: f64,
    /// Is index size acceptable?
    pub size_health: HealthLevel,
}

/// Health level for specific metric.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthLevel {
    /// Metric is healthy
    Healthy,
    /// Metric is concerning
    Warning,
    /// Metric is problematic
    Critical,
}

/// A single health issue found during assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Severity: high, medium, low
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// Metric value that caused the issue
    pub metric_value: String,
    /// Threshold that was exceeded/not met
    pub threshold: String,
}

/// Severity level of a health issue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// High priority - needs immediate attention
    High,
    /// Medium priority - should be addressed soon
    Medium,
    /// Low priority - nice to optimize
    Low,
}

/// Response for index optimization operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeIndexResponse {
    /// Whether optimization was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Segment count before optimization
    pub segments_before: usize,
    /// Segment count after optimization
    pub segments_after: usize,
    /// Index size before in MB
    pub size_before_mb: f64,
    /// Index size after in MB
    pub size_after_mb: f64,
    /// Percentage reduction in size
    pub size_reduction_percent: f64,
    /// Time taken for optimization in milliseconds
    pub duration_ms: u64,
}

/// Tuning recommendation for index optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningRecommendation {
    /// Impact level: high, medium, low
    pub impact: ImpactLevel,
    /// Recommendation title
    pub title: String,
    /// Detailed recommendation description
    pub description: String,
    /// Environment variable or setting to adjust
    pub parameter: Option<String>,
    /// Current value
    pub current_value: Option<String>,
    /// Recommended value
    pub recommended_value: Option<String>,
    /// Why this recommendation is important
    pub reason: String,
}

/// Impact level of a tuning recommendation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    /// High impact optimization
    High,
    /// Medium impact optimization
    Medium,
    /// Low impact optimization
    Low,
}

/// Response containing tuning recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningRecommendationsResponse {
    /// Current index metrics used for analysis
    pub current_metrics: IndexStatsResponse,
    /// Current health status
    pub health_status: HealthStatus,
    /// List of recommendations sorted by impact
    pub recommendations: Vec<TuningRecommendation>,
    /// Analysis timestamp
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
    /// Summary of key findings
    pub summary: String,
}

/// Configuration for Tantivy search engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantivyConfig {
    /// Memory buffer for index writer in MB
    pub memory_mb: usize,
    /// Number of threads for index writer
    pub num_threads: Option<usize>,
    /// Number of CPU cores detected
    pub cpu_cores: usize,
}

impl Default for TantivyConfig {
    fn default() -> Self {
        Self {
            memory_mb: 200,
            num_threads: None,
            cpu_cores: 4, // Default to 4 cores
        }
    }
}

impl TantivyConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        let memory_mb =
            std::env::var("KLASK_TANTIVY_MEMORY_MB").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(200);

        let num_threads = std::env::var("KLASK_TANTIVY_NUM_THREADS").ok().and_then(|v| v.parse::<usize>().ok());

        Self {
            memory_mb,
            num_threads,
            cpu_cores: 4, // Default to 4 cores (can be overridden)
        }
    }

    /// Validate configuration values.
    pub fn validate(&self) -> Result<(), String> {
        if self.memory_mb < 50 {
            return Err("KLASK_TANTIVY_MEMORY_MB must be at least 50".to_string());
        }
        if self.memory_mb > 8000 {
            return Err("KLASK_TANTIVY_MEMORY_MB must not exceed 8000".to_string());
        }
        if let Some(threads) = self.num_threads {
            if threads < 1 {
                return Err("KLASK_TANTIVY_NUM_THREADS must be at least 1".to_string());
            }
            if threads > self.cpu_cores * 2 {
                return Err(format!(
                    "KLASK_TANTIVY_NUM_THREADS ({}) exceeds 2x CPU cores ({})",
                    threads, self.cpu_cores
                ));
            }
        }
        Ok(())
    }
}
