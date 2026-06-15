import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../lib/api';
import type { ApiTokenInfo, CreateTokenResponse } from '../types';

// Query Keys
const apiTokensKeys = {
  all: ['api-tokens'] as const,
  list: () => [...apiTokensKeys.all, 'list'] as const,
};

// Response validators
function validateApiTokenInfo(data: unknown): ApiTokenInfo {
  const obj = data as Record<string, unknown>;
  if (
    typeof obj?.id !== 'string' ||
    typeof obj?.name !== 'string' ||
    typeof obj?.token_prefix !== 'string' ||
    typeof obj?.scope !== 'string' ||
    typeof obj?.active !== 'boolean' ||
    typeof obj?.created_at !== 'string'
  ) {
    throw new Error('Invalid ApiTokenInfo structure from backend');
  }
  return obj as unknown as ApiTokenInfo;
}

function validateApiTokenInfoArray(data: unknown): ApiTokenInfo[] {
  if (!Array.isArray(data)) {
    throw new Error('Expected array of tokens');
  }
  return data.map(validateApiTokenInfo);
}

function validateCreateTokenResponse(data: unknown): CreateTokenResponse {
  const obj = data as Record<string, unknown>;
  if (
    typeof obj?.id !== 'string' ||
    typeof obj?.token !== 'string' ||
    typeof obj?.token_prefix !== 'string' ||
    typeof obj?.name !== 'string' ||
    typeof obj?.created_at !== 'string'
  ) {
    throw new Error('Invalid CreateTokenResponse structure from backend');
  }
  return obj as unknown as CreateTokenResponse;
}

// Fetch Functions
async function fetchApiTokens(): Promise<ApiTokenInfo[]> {
  const response = await api.get<ApiTokenInfo[]>('/api/user/tokens');
  return validateApiTokenInfoArray(response);
}

async function createApiToken(name: string): Promise<CreateTokenResponse> {
  const response = await api.post<CreateTokenResponse>('/api/user/tokens', { name });
  return validateCreateTokenResponse(response);
}

async function revokeApiToken(tokenId: string): Promise<void> {
  await api.delete(`/api/user/tokens/${tokenId}`);
}

/**
 * Hook to fetch all API tokens for the current user
 */
export function useApiTokens() {
  return useQuery({
    queryKey: apiTokensKeys.list(),
    queryFn: fetchApiTokens,
    staleTime: 5 * 60000, // 5 minutes
    retry: 2,
    retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Hook to create a new API token
 */
export function useCreateApiToken() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => createApiToken(name),
    onSuccess: () => {
      // Invalidate tokens list to refetch updated data
      queryClient.invalidateQueries({ queryKey: apiTokensKeys.list() });
    },
  });
}

/**
 * Hook to revoke an API token
 */
export function useRevokeApiToken() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (tokenId: string) => revokeApiToken(tokenId),
    onSuccess: () => {
      // Invalidate tokens list to refetch updated data
      queryClient.invalidateQueries({ queryKey: apiTokensKeys.list() });
    },
  });
}
