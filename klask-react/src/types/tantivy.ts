// Tantivy Search Index Types - matching Rust backend models

/**
 * Represents metrics for a single segment in the search index
 */
export interface SegmentMetrics {
  segment_ord: number;
  doc_count: number;
  max_doc: number;
  deleted_docs: number;
  size_bytes: number;
  space_breakdown: SpaceBreakdown;
}

/**
 * Space breakdown for a segment
 */
export interface SpaceBreakdown {
  postings: number;
  store: number;
  fast_fields: number;
  positions: number;
  other: number;
}

/**
 * Space usage breakdown by component type
 */
export interface SpaceUsageBreakdown {
  postings_bytes: number;
  store_bytes: number;
  fast_fields_bytes: number;
  positions_bytes: number;
  other_bytes: number;
}

/**
 * Cache statistics for the index
 */
export interface CacheStats {
  num_entries: number;
  hits: number;
  misses: number;
  hit_ratio: number;
}


/**
 * Complete index statistics response
 */
export interface IndexStatsResponse {
  total_documents: number;
  total_size_mb: number;
  total_size_bytes: number;
  segment_count: number;
  segments: SegmentMetrics[];
  space_usage: SpaceUsageBreakdown;
  cache_stats: CacheStats;
}

/**
 * Health status enum
 */
export enum IndexHealthStatus {
  HEALTHY = 'Healthy',
  WARNING = 'Warning',
  DEGRADED = 'Degraded'
}

/**
 * Health level for specific metric
 */
export enum HealthLevel {
  HEALTHY = 'Healthy',
  WARNING = 'Warning',
  CRITICAL = 'Critical'
}

/**
 * Issue severity
 */
export enum IssueSeverity {
  HIGH = 'High',
  MEDIUM = 'Medium',
  LOW = 'Low'
}

/**
 * A single health issue found during assessment
 */
export interface HealthIssue {
  severity: IssueSeverity;
  description: string;
  metric_value: string;
  threshold: string;
}

/**
 * Details of health checks performed
 */
export interface HealthCheckDetails {
  segment_count: number;
  segment_health: HealthLevel;
  cache_hit_ratio_percent: number;
  cache_health: HealthLevel;
  deleted_docs_ratio_percent: number;
  deletion_health: HealthLevel;
  index_size_mb: number;
  size_health: HealthLevel;
}

/**
 * Complete index health response
 */
export interface IndexHealthResponse {
  status: IndexHealthStatus;
  status_message: string;
  checked_at: string;
  index_stats: IndexStatsResponse;
  health_checks: HealthCheckDetails;
  issues: HealthIssue[];
}

/**
 * Impact level of a tuning recommendation
 */
export enum ImpactLevel {
  HIGH = 'High',
  MEDIUM = 'Medium',
  LOW = 'Low'
}

/**
 * Tuning recommendation for index optimization
 */
export interface TuningRecommendation {
  impact: ImpactLevel;
  title: string;
  description: string;
  parameter?: string;
  current_value?: string;
  recommended_value?: string;
  reason: string;
}

/**
 * Tuning recommendations response
 */
export interface TuningSettingsResponse {
  current_metrics: IndexStatsResponse;
  health_status: IndexHealthStatus;
  recommendations: TuningRecommendation[];
  analyzed_at: string;
  summary: string;
}

/**
 * Response from optimize index operation
 */
export interface OptimizeIndexResponse {
  success: boolean;
  message: string;
  segments_before: number;
  segments_after: number;
  size_before_mb: number;
  size_after_mb: number;
  size_reduction_percent: number;
  duration_ms: number;
}

/**
 * Request to trigger index optimization
 */
export interface OptimizeIndexRequest {
  remove_deleted_docs?: boolean;
  merge_segments?: boolean;
  rebuild_cache?: boolean;
}

/**
 * Index operation status
 */
export interface IndexOperationStatus {
  operation: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  progress_percentage: number;
  started_at: string;
  estimated_duration_ms?: number;
  error_message?: string;
}

