import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../lib/api';

/**
 * Status of the semantic (vector) search index and its backfill job.
 * Mirrors the backend `SemanticStatusResponse` (GET /api/admin/semantic/status).
 */
export interface SemanticStatusResponse {
  /** True when semantic indexing is active (feature built + enabled + store opened). */
  enabled: boolean;
  /** A rebuild is currently running. */
  running: boolean;
  /** Documents enqueued for re-embedding so far in the current/last run. */
  processed: number;
  /** Total documents to process (Tantivy count at start), if known. */
  total: number | null;
  /** Chunks currently stored in the vector index. */
  chunks_indexed: number;
  /** Files handed to the embedding worker but not yet embedded (queued + in-flight). */
  queue_depth: number;
  /** Embedding model id the index is built with. */
  model: string | null;
  /** Embedding dimension. */
  dimension: number | null;
  /** Last error message, if the last run failed. */
  error: string | null;
  /** True if the last run was cancelled. */
  cancelled: boolean;
  /** ISO-8601 start time of the current/last run. */
  started_at: string | null;
  /** ISO-8601 finish time of the current/last run. */
  finished_at: string | null;
}

export interface SemanticActionResponse {
  success: boolean;
  message: string;
}

const semanticKeys = {
  all: ['semantic-index'] as const,
  status: () => [...semanticKeys.all, 'status'] as const,
};

function validateSemanticStatus(data: unknown): SemanticStatusResponse {
  const obj = data as Record<string, unknown>;
  if (typeof obj?.enabled !== 'boolean' || typeof obj?.running !== 'boolean') {
    throw new Error('Invalid SemanticStatusResponse structure from backend');
  }
  return obj as unknown as SemanticStatusResponse;
}

async function fetchSemanticStatus(): Promise<SemanticStatusResponse> {
  const response = await api.get<SemanticStatusResponse>('/api/admin/semantic/status');
  return validateSemanticStatus(response);
}

/**
 * Poll the semantic index status.
 *
 * Polls automatically every `runningIntervalMs` while a rebuild is running
 * *or* the embedding worker still has queued files (e.g. a crawl feeding the
 * semantic index), and stops once both are idle — driven off the query's own
 * data via a function `refetchInterval`, so the component calls this hook
 * exactly once (no duplicate queries, no manual timers).
 */
export function useSemanticStatus(runningIntervalMs = 1500) {
  return useQuery({
    queryKey: semanticKeys.status(),
    queryFn: fetchSemanticStatus,
    staleTime: 2000,
    // `query.state.data` is the latest fetched status; poll while working.
    refetchInterval: (query) => {
      const status = query.state.data;
      return status?.running || (status?.queue_depth ?? 0) > 0 ? runningIntervalMs : false;
    },
    retry: 1,
    retryDelay: 1000,
    // Degrade gracefully: if the endpoint is unreachable, the card just hides.
    throwOnError: false,
  });
}

/**
 * Start a semantic index rebuild (backfill). The backend returns 202 on start
 * and 409 when one is already running (surfaced as a thrown error here).
 */
export function useStartBackfill() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (): Promise<SemanticActionResponse> => {
      return api.post<SemanticActionResponse>('/api/admin/semantic/backfill');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: semanticKeys.status() });
    },
  });
}

/** Request cancellation of a running rebuild. */
export function useCancelBackfill() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (): Promise<SemanticActionResponse> => {
      return api.post<SemanticActionResponse>('/api/admin/semantic/cancel');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: semanticKeys.status() });
    },
  });
}
