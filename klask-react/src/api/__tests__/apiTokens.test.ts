import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as apiTokensModule from '../apiTokens';

// Mock react-query
vi.mock('@tanstack/react-query', () => ({
  useQuery: vi.fn(),
  useMutation: vi.fn(),
  useQueryClient: vi.fn(),
}));

// Mock the api module
vi.mock('../../lib/api', () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
  },
}));

describe('API Tokens', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should export useApiTokens hook', () => {
    expect(typeof apiTokensModule.useApiTokens).toBe('function');
  });

  it('should export useCreateApiToken hook', () => {
    expect(typeof apiTokensModule.useCreateApiToken).toBe('function');
  });

  it('should export useRevokeApiToken hook', () => {
    expect(typeof apiTokensModule.useRevokeApiToken).toBe('function');
  });
});
