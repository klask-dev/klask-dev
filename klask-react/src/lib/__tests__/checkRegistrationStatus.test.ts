import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { apiClient, ApiError, api } from '../api';
import type { RegistrationStatus } from '../../types';

// Mock fetch globally
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
vi.stubGlobal('localStorage', localStorageMock);

describe('API Client - checkRegistrationStatus', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockImplementation((key) => {
      if (key === 'authToken') return 'mock-token';
      return null;
    });
    (apiClient as any).setToken('mock-token');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const createMockResponse = (data: any, status = 200, ok = true) => ({
    ok,
    status,
    headers: new Headers({ 'Content-Type': 'application/json' }),
    json: async () => data,
    text: async () => JSON.stringify(data),
  });

  describe('checkRegistrationStatus method', () => {
    it('should make GET request to correct endpoint', async () => {
      const mockResponseData: RegistrationStatus = { registration_allowed: true };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponseData));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3000/api/auth/registration/status',
        {
          headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer mock-token',
          },
        }
      );

      expect(result).toEqual(mockResponseData);
    });

    it('should return registration_allowed true when enabled', async () => {
      const mockResponseData: RegistrationStatus = { registration_allowed: true };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponseData));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(result.registration_allowed).toBe(true);
    });

    it('should return registration_allowed false when disabled', async () => {
      const mockResponseData: RegistrationStatus = { registration_allowed: false };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponseData));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(result.registration_allowed).toBe(false);
    });

    it('should handle 200 OK response', async () => {
      const mockResponse: RegistrationStatus = { registration_allowed: true };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponse, 200, true));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(result).toEqual(mockResponse);
    });

    it('should handle 401 unauthorized error', async () => {
      const errorResponse = { error: 'Unauthorized' };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 401, false));

      try {
        await apiClient.auth.checkRegistrationStatus();
        throw new Error('Expected ApiError to be thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(401);
      }
    });

    it('should handle 500 server error', async () => {
      const errorResponse = { error: 'Internal server error' };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 500, false));

      try {
        await apiClient.auth.checkRegistrationStatus();
        throw new Error('Expected ApiError to be thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(500);
      }
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(apiClient.auth.checkRegistrationStatus()).rejects.toThrow('Network error');
    });

    it('should handle timeout errors', async () => {
      mockFetch.mockImplementationOnce(
        () =>
          new Promise((_, reject) =>
            setTimeout(() => reject(new Error('Request timeout')), 100)
          )
      );

      await expect(apiClient.auth.checkRegistrationStatus()).rejects.toThrow('Request timeout');
    });

    it('should include authorization header when token is present', async () => {
      const token = 'valid-auth-token';
      localStorageMock.getItem.mockReturnValue(token);

      const apiInstance = new (apiClient.constructor as any)('http://localhost:3000');
      apiInstance.setToken(token);

      mockFetch.mockResolvedValueOnce(createMockResponse({ registration_allowed: true }));

      await apiInstance.auth.checkRegistrationStatus();

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': `Bearer ${token}`,
          }),
        })
      );
    });

    it('should work without authentication token', async () => {
      localStorageMock.getItem.mockReturnValue(null);

      const clientWithoutToken = new (apiClient.constructor as any)('http://localhost:3000');
      const mockResponse: RegistrationStatus = { registration_allowed: true };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponse));

      const result = await clientWithoutToken.auth.checkRegistrationStatus();

      expect(result).toEqual(mockResponse);

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers).not.toHaveProperty('Authorization');
    });

    it('should handle malformed JSON response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => {
          throw new Error('Invalid JSON');
        },
        text: async () => 'Invalid JSON response',
      });

      await expect(apiClient.auth.checkRegistrationStatus()).rejects.toThrow();
    });

    it('should handle empty response', async () => {
      const emptyResponse = {};

      mockFetch.mockResolvedValueOnce(createMockResponse(emptyResponse));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(result).toEqual(emptyResponse);
    });

    it('should preserve error details in ApiError', async () => {
      const errorResponse = {
        error: 'Service unavailable',
        details: {
          reason: 'Database connection failed',
        },
      };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 503, false));

      try {
        await apiClient.auth.checkRegistrationStatus();
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).details).toEqual(errorResponse);
        expect((error as ApiError).status).toBe(503);
      }
    });

    it('should handle 403 Forbidden response', async () => {
      const errorResponse = { error: 'Forbidden' };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 403, false));

      try {
        await apiClient.auth.checkRegistrationStatus();
        throw new Error('Expected ApiError to be thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(403);
      }
    });

    it('should handle 404 Not Found response', async () => {
      const errorResponse = { error: 'Endpoint not found' };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 404, false));

      try {
        await apiClient.auth.checkRegistrationStatus();
        throw new Error('Expected ApiError to be thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(404);
      }
    });
  });

  describe('API wrapper - checkRegistrationStatus', () => {
    it('should call apiClient.auth.checkRegistrationStatus via api wrapper', async () => {
      const mockResponse: RegistrationStatus = { registration_allowed: true };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponse));

      const result = await api.checkRegistrationStatus();

      expect(result).toEqual(mockResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3000/api/auth/registration/status',
        {
          headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer mock-token',
          },
        }
      );
    });

    it('should propagate errors from apiClient', async () => {
      const errorResponse = { error: 'Test error' };

      mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 500, false));

      await expect(api.checkRegistrationStatus()).rejects.toThrow(ApiError);
    });
  });

  describe('Edge cases and error handling', () => {
    it('should handle concurrent requests', async () => {
      const mockResponses: RegistrationStatus[] = [
        { registration_allowed: true },
        { registration_allowed: false },
        { registration_allowed: true },
      ];

      mockFetch
        .mockResolvedValueOnce(createMockResponse(mockResponses[0]))
        .mockResolvedValueOnce(createMockResponse(mockResponses[1]))
        .mockResolvedValueOnce(createMockResponse(mockResponses[2]));

      const promises = [
        apiClient.auth.checkRegistrationStatus(),
        apiClient.auth.checkRegistrationStatus(),
        apiClient.auth.checkRegistrationStatus(),
      ];

      const results = await Promise.all(promises);

      expect(results).toHaveLength(3);
      expect(results[0].registration_allowed).toBe(true);
      expect(results[1].registration_allowed).toBe(false);
      expect(results[2].registration_allowed).toBe(true);
      expect(mockFetch).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed success and failure responses', async () => {
      mockFetch
        .mockResolvedValueOnce(createMockResponse({ registration_allowed: true }))
        .mockResolvedValueOnce(createMockResponse({ error: 'Failure' }, 500, false));

      const results = await Promise.allSettled([
        apiClient.auth.checkRegistrationStatus(),
        apiClient.auth.checkRegistrationStatus(),
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect(results[1].status).toBe('rejected');
    });

    it('should return correct type structure', async () => {
      const mockResponse: RegistrationStatus = { registration_allowed: false };

      mockFetch.mockResolvedValueOnce(createMockResponse(mockResponse));

      const result = await apiClient.auth.checkRegistrationStatus();

      expect(typeof result).toBe('object');
      expect('registration_allowed' in result).toBe(true);
      expect(typeof result.registration_allowed).toBe('boolean');
    });

    it('should use correct HTTP method', async () => {
      mockFetch.mockResolvedValueOnce(
        createMockResponse({ registration_allowed: true })
      );

      await apiClient.auth.checkRegistrationStatus();

      const callArgs = mockFetch.mock.calls[0][1];
      // GET is the default method when method is not explicitly set
      expect(callArgs.method === undefined || callArgs.method === 'GET').toBe(true);
    });

    it('should not include body in GET request', async () => {
      mockFetch.mockResolvedValueOnce(
        createMockResponse({ registration_allowed: true })
      );

      await apiClient.auth.checkRegistrationStatus();

      const callArgs = mockFetch.mock.calls[0][1];
      expect('body' in callArgs && callArgs.body).toBeFalsy();
    });
  });
});
