import { useQuery } from '@tanstack/react-query';
import { api } from '../lib/api';

/**
 * What the server's search engine can do, for the regular (non-admin) search UI.
 * Mirrors the backend `SearchCapabilities` (GET /api/search/capabilities).
 */
export interface SearchCapabilities {
  /** Semantic/hybrid modes are usable (feature built + model loaded + store open). */
  semantic_enabled: boolean;
}

const searchCapabilityKeys = {
  all: ['search-capabilities'] as const,
};

async function fetchSearchCapabilities(): Promise<SearchCapabilities> {
  const data = await api.get<SearchCapabilities>('/api/search/capabilities');
  // Defensive: an old/misbehaving backend just means "semantic off".
  return { semantic_enabled: typeof data?.semantic_enabled === 'boolean' ? data.semantic_enabled : false };
}

/**
 * Whether semantic/hybrid search is available on this server.
 *
 * Capabilities are effectively static for the life of the server process, so
 * this is cached aggressively and never refetched in the background. Degrades
 * gracefully: if the endpoint is unreachable the toggle simply stays hidden.
 */
export function useSearchCapabilities() {
  return useQuery({
    queryKey: searchCapabilityKeys.all,
    queryFn: fetchSearchCapabilities,
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    retry: 1,
    throwOnError: false,
  });
}
