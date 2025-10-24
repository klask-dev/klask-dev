import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../lib/api';
import type {
  IndexStatsResponse,
  IndexHealthResponse,
  TuningSettingsResponse,
  OptimizeIndexResponse,
  OptimizeIndexRequest
} from '../types/tantivy';

// Query Keys
const indexMetricsKeys = {
  all: ['index-metrics'] as const,
  stats: () => [...indexMetricsKeys.all, 'stats'] as const,
  health: () => [...indexMetricsKeys.all, 'health'] as const,
  tuning: () => [...indexMetricsKeys.all, 'tuning'] as const,
};

// Response validators
function validateIndexStatsResponse(data: unknown): IndexStatsResponse {
  const obj = data as any;
  if (
    typeof obj?.total_documents !== 'number' ||
    typeof obj?.total_size_mb !== 'number' ||
    typeof obj?.total_size_bytes !== 'number' ||
    typeof obj?.segment_count !== 'number' ||
    !Array.isArray(obj?.segments) ||
    !obj?.space_usage ||
    !obj?.cache_stats
  ) {
    throw new Error('Invalid IndexStatsResponse structure from backend');
  }
  return obj as IndexStatsResponse;
}

function validateIndexHealthResponse(data: unknown): IndexHealthResponse {
  const obj = data as any;
  if (
    !obj?.status ||
    !obj?.checked_at ||
    !obj?.index_stats ||
    !obj?.health_checks ||
    !Array.isArray(obj?.issues)
  ) {
    throw new Error('Invalid IndexHealthResponse structure from backend');
  }
  return obj as IndexHealthResponse;
}

function validateTuningSettingsResponse(data: unknown): TuningSettingsResponse {
  const obj = data as any;
  if (
    !obj?.current_metrics ||
    !obj?.health_status ||
    !Array.isArray(obj?.recommendations) ||
    !obj?.analyzed_at
  ) {
    throw new Error('Invalid TuningSettingsResponse structure from backend');
  }
  return obj as TuningSettingsResponse;
}

function validateOptimizeIndexResponse(data: unknown): OptimizeIndexResponse {
  const obj = data as any;
  if (
    typeof obj?.success !== 'boolean' ||
    typeof obj?.message !== 'string' ||
    typeof obj?.segments_before !== 'number' ||
    typeof obj?.segments_after !== 'number' ||
    typeof obj?.duration_ms !== 'number'
  ) {
    throw new Error('Invalid OptimizeIndexResponse structure from backend');
  }
  return obj as OptimizeIndexResponse;
}

// Fetch Functions
async function fetchIndexStats(): Promise<IndexStatsResponse> {
  const response = await api.get<IndexStatsResponse>('/api/admin/search/index-stats');
  return validateIndexStatsResponse(response);
}

async function fetchIndexHealth(): Promise<IndexHealthResponse> {
  const response = await api.get<IndexHealthResponse>('/api/admin/search/index-health');
  return validateIndexHealthResponse(response);
}

async function fetchTuningRecommendations(): Promise<TuningSettingsResponse> {
  const response = await api.get<TuningSettingsResponse>('/api/admin/search/tuning-recommendations');
  return validateTuningSettingsResponse(response);
}

async function optimizeIndex(request: OptimizeIndexRequest): Promise<OptimizeIndexResponse> {
  const response = await api.post<OptimizeIndexResponse>('/api/admin/search/optimize-index', request);
  return validateOptimizeIndexResponse(response);
}

/**
 * Hook to fetch index statistics
 * Auto-refresh support with configurable interval
 */
export function useIndexStats(refetchInterval?: number | false) {
  return useQuery({
    queryKey: indexMetricsKeys.stats(),
    queryFn: fetchIndexStats,
    staleTime: 30000, // 30 seconds
    refetchInterval: refetchInterval !== undefined ? refetchInterval : false,
    retry: 2,
    retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Hook to fetch index health status
 */
export function useIndexHealth(refetchInterval?: number | false) {
  return useQuery({
    queryKey: indexMetricsKeys.health(),
    queryFn: fetchIndexHealth,
    staleTime: 60000, // 1 minute
    refetchInterval: refetchInterval !== undefined ? refetchInterval : false,
    retry: 2,
    retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Hook to fetch tuning recommendations
 */
export function useTuningRecommendations() {
  return useQuery({
    queryKey: indexMetricsKeys.tuning(),
    queryFn: fetchTuningRecommendations,
    staleTime: 5 * 60000, // 5 minutes
    retry: 2,
    retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Mutation to optimize the index
 */
export function useOptimizeIndex() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: OptimizeIndexRequest) => optimizeIndex(request),
    onSuccess: () => {
      // Invalidate related queries to refetch updated data
      queryClient.invalidateQueries({ queryKey: indexMetricsKeys.stats() });
      queryClient.invalidateQueries({ queryKey: indexMetricsKeys.health() });
    },
  });
}

/**
 * Hook to combine all index metrics with auto-refresh
 * Returns all metrics along with loading/error states
 */
export function useAllIndexMetrics(autoRefreshMs: number | false = false) {
  const statsQuery = useIndexStats(autoRefreshMs);
  const healthQuery = useIndexHealth(autoRefreshMs);
  const tuningQuery = useTuningRecommendations();

  const isLoading = statsQuery.isLoading || healthQuery.isLoading || tuningQuery.isLoading;
  const error = statsQuery.error || healthQuery.error || tuningQuery.error;
  const data = {
    stats: statsQuery.data,
    health: healthQuery.data,
    tuning: tuningQuery.data,
  };

  return {
    data,
    isLoading,
    error: error as Error | null,
    refetch: async () => {
      await Promise.all([
        statsQuery.refetch(),
        healthQuery.refetch(),
        tuningQuery.refetch(),
      ]);
    },
  };
}
