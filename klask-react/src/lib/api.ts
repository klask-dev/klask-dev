import type {
  ApiResponse,
  User,
  Repository,
  RepositoryWithStats,
  File,
  SearchQuery,
  SearchResponse,
  LoginRequest,
  RegisterRequest,
  AuthResponse,
  CreateRepositoryRequest,
  CreateUserRequest,
  UpdateUserRequest,
  UserStats,
  PaginatedResponse,
  AdminDashboardData,
  SystemStats,
  RepositoryStats,
  SearchStats,
  RecentActivity,
  CrawlProgressInfo,
  ScheduleRepositoryRequest,
  SchedulerStatus,
  UpdateProfileRequest,
  ChangePasswordRequest,
  UserActivity
} from '../types';

// API Error class
export class ApiError extends Error {
  public status: number;
  public details?: Record<string, any>;

  constructor(message: string, status: number, details?: Record<string, any>) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.details = details;
  }
}

// API Configuration with runtime override support
import { getApiBaseUrl } from './config';

class ApiClient {
  private baseURL: string;
  private token: string | null = null;
  private csrfToken: string | null = null;

  constructor(baseURL?: string) {
    this.baseURL = baseURL || getApiBaseUrl();
    this.token = localStorage.getItem('authToken');
    this.csrfToken = this.getCsrfToken();
  }

  /**
   * Get CSRF token from meta tag or generate a new one
   */
  private getCsrfToken(): string | null {
    try {
      // First, try to get from meta tag
      const metaTag = document.querySelector('meta[name="csrf-token"]');
      if (metaTag) {
        return metaTag.getAttribute('content');
      }

      // If not available, check localStorage
      let token = localStorage.getItem('csrfToken');
      if (!token) {
        // Generate a new token (should be validated by backend)
        token = this.generateToken();
        localStorage.setItem('csrfToken', token);
      }
      return token;
    } catch {
      return null;
    }
  }

  /**
   * Generate a random token for CSRF protection
   */
  private generateToken(): string {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    return Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join('');
  }

  /**
   * Refresh CSRF token for security
   */
  private refreshCsrfToken(): void {
    this.csrfToken = this.generateToken();
    localStorage.setItem('csrfToken', this.csrfToken);
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    const method = (options.method || 'GET').toUpperCase();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    // Add CSRF token for state-changing requests (POST, PUT, DELETE, PATCH)
    if (['POST', 'PUT', 'DELETE', 'PATCH'].includes(method) && this.csrfToken) {
      headers['X-CSRF-Token'] = this.csrfToken;
    }

    const config: RequestInit = {
      ...options,
      headers,
    };

    try {
      const response = await fetch(url, config);
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new ApiError(
          errorData.message || `HTTP ${response.status}: ${response.statusText}`,
          response.status,
          errorData
        );
      }

      // Handle empty responses (like 204 No Content)
      if (response.status === 204) {
        return {} as T;
      }

      return await response.json();
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      
      throw new ApiError(
        error instanceof Error ? error.message : 'Network error',
        0,
        { originalError: error }
      );
    }
  }

  // Authentication Methods
  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('authToken', token);
    } else {
      localStorage.removeItem('authToken');
    }
  }

  getToken(): string | null {
    return this.token;
  }

  // Auth API object
  auth = {
    login: async (credentials: LoginRequest): Promise<AuthResponse> => {
      const response = await this.request<AuthResponse>('/api/auth/login', {
        method: 'POST',
        body: JSON.stringify(credentials),
      });

      this.setToken(response.token);
      return response;
    },

    register: async (data: RegisterRequest): Promise<AuthResponse> => {
      const response = await this.request<AuthResponse>('/api/auth/register', {
        method: 'POST',
        body: JSON.stringify(data),
      });

      this.setToken(response.token);
      return response;
    },

    getProfile: async (): Promise<User> => {
      return this.request<User>('/api/auth/profile');
    },

    updateProfile: async (data: UpdateProfileRequest): Promise<User> => {
      return this.request<User>('/api/auth/profile', {
        method: 'PUT',
        body: JSON.stringify(data),
      });
    },

    uploadAvatar: async (file: globalThis.File): Promise<{ avatar_url: string }> => {
      const formData = new FormData();
      formData.append('file', file);

      const url = `${this.baseURL}/api/auth/avatar`;
      const headers: Record<string, string> = {};

      if (this.token) {
        headers['Authorization'] = `Bearer ${this.token}`;
      }

      const response = await fetch(url, {
        method: 'POST',
        headers,
        body: formData,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new ApiError(
          errorData.message || `HTTP ${response.status}: ${response.statusText}`,
          response.status,
          errorData
        );
      }

      return await response.json();
    },

    changePassword: async (data: ChangePasswordRequest): Promise<{ message: string }> => {
      return this.request<{ message: string }>('/api/auth/password', {
        method: 'PUT',
        body: JSON.stringify(data),
      });
    },

    getUserActivity: async (): Promise<UserActivity> => {
      return this.request<UserActivity>('/api/auth/activity');
    },

    deleteAccount: async (password: string): Promise<{ message: string }> => {
      return this.request<{ message: string }>('/api/auth/account', {
        method: 'DELETE',
        body: JSON.stringify({ password }),
      });
    },

    logout: () => {
      this.setToken(null);
    }
  };

  // Search API
  async search(query: SearchQuery): Promise<SearchResponse> {
    const params = new URLSearchParams();
    
    if (query.query) params.append('q', query.query);
    if (query.project) params.append('project', query.project);
    if (query.version) params.append('version', query.version);
    if (query.extension) params.append('extension', query.extension);
    if (query.maxResults) params.append('max_results', query.maxResults.toString());
    
    // Calculate page number from offset
    if (query.offset !== undefined && query.maxResults) {
      const page = Math.floor(query.offset / query.maxResults) + 1;
      params.append('page', page.toString());
    }

    return this.request<SearchResponse>(`/api/search?${params.toString()}`);
  }

  // File API
  async getFile(id: string): Promise<File> {
    return this.request<File>(`/api/files/${id}`);
  }

  async getFileByDocAddress(docAddress: string): Promise<File> {
    return this.request<File>(`/api/files/doc/${docAddress}`);
  }

  async getFiles(params?: {
    page?: number;
    size?: number;
    project?: string;
    version?: string;
    extension?: string;
  }): Promise<PaginatedResponse<File>> {
    const searchParams = new URLSearchParams();
    
    if (params?.page) searchParams.append('page', params.page.toString());
    if (params?.size) searchParams.append('size', params.size.toString());
    if (params?.project) searchParams.append('project', params.project);
    if (params?.version) searchParams.append('version', params.version);
    if (params?.extension) searchParams.append('extension', params.extension);

    return this.request<PaginatedResponse<File>>(`/api/files?${searchParams.toString()}`);
  }

  // Repository API
  async getRepositories(): Promise<RepositoryWithStats[]> {
    const response = await this.request<{ repositories: RepositoryWithStats[], total: number }>('/api/repositories');
    return response.repositories;
  }

  async getRepositoriesWithStats(): Promise<RepositoryWithStats[]> {
    const response = await this.request<{ repositories: RepositoryWithStats[], total: number }>('/api/repositories?include_stats=true');
    return response.repositories;
  }

  async getRepository(id: string): Promise<Repository> {
    return this.request<Repository>(`/api/repositories/${id}`);
  }

  async createRepository(data: CreateRepositoryRequest): Promise<Repository> {
    return this.request<Repository>('/api/repositories', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateRepository(id: string, data: Partial<CreateRepositoryRequest>): Promise<Repository> {
    return this.request<Repository>(`/api/repositories/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteRepository(id: string): Promise<void> {
    return this.request<void>(`/api/repositories/${id}`, {
      method: 'DELETE',
    });
  }

  async crawlRepository(id: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/api/repositories/${id}/crawl`, {
      method: 'POST',
    });
  }

  async stopCrawlRepository(id: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/api/repositories/${id}/crawl`, {
      method: 'DELETE',
    });
  }

  async getRepositoryProgress(id: string): Promise<CrawlProgressInfo | null> {
    return this.request<CrawlProgressInfo | null>(`/api/repositories/${id}/progress`);
  }

  async getActiveProgress(): Promise<CrawlProgressInfo[]> {
    const response = await this.request<{ active_progress: CrawlProgressInfo[] }>('/api/repositories/progress/active');
    return response.active_progress;
  }

  async updateRepositorySchedule(id: string, data: ScheduleRepositoryRequest): Promise<Repository> {
    return this.request<Repository>(`/api/repositories/${id}/schedule`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async discoverGitlabProjects(data: {
    gitlabUrl?: string;
    accessToken: string;
    namespace?: string;
    autoCrawlEnabled?: boolean;
    cronSchedule?: string;
    crawlFrequencyHours?: number;
    maxCrawlDurationMinutes?: number;
  }): Promise<Repository[]> {
    return this.request<Repository[]>('/api/repositories/gitlab/discover', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async discoverGitHubRepositories(data: {
    githubUrl?: string;
    accessToken: string;
    namespace?: string;
    autoCrawlEnabled?: boolean;
    cronSchedule?: string;
    crawlFrequencyHours?: number;
    maxCrawlDurationMinutes?: number;
  }): Promise<Repository[]> {
    return this.request<Repository[]>('/api/repositories/github/discover', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async testGitHubToken(data: {
    githubUrl?: string;
    accessToken: string;
  }): Promise<{ valid: boolean; message?: string }> {
    return this.request<{ valid: boolean; message?: string }>('/api/repositories/github/test-token', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getSchedulerStatus(): Promise<SchedulerStatus> {
    return this.request<SchedulerStatus>('/api/scheduler/status');
  }

  // User Management API
  async getUsers(): Promise<User[]> {
    return this.request<User[]>('/api/users');
  }

  async getUser(id: string): Promise<User> {
    return this.request<User>(`/api/users/${id}`);
  }

  async createUser(data: CreateUserRequest): Promise<User> {
    return this.request<User>('/api/users', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateUser(id: string, data: UpdateUserRequest): Promise<User> {
    return this.request<User>(`/api/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async updateUserRole(id: string, role: string): Promise<User> {
    return this.request<User>(`/api/users/${id}/role`, {
      method: 'PUT',
      body: JSON.stringify(role),
    });
  }

  async updateUserStatus(id: string, active: boolean): Promise<User> {
    return this.request<User>(`/api/users/${id}/status`, {
      method: 'PUT',
      body: JSON.stringify(active),
    });
  }

  async deleteUser(id: string): Promise<void> {
    return this.request<void>(`/api/users/${id}`, {
      method: 'DELETE',
    });
  }

  async getUserStats(): Promise<UserStats> {
    return this.request<UserStats>('/api/admin/users/stats');
  }

  // Admin API
  async getAdminDashboard(): Promise<AdminDashboardData> {
    return this.request<AdminDashboardData>('/api/admin/dashboard');
  }

  async getSystemStats(): Promise<SystemStats> {
    return this.request<SystemStats>('/api/admin/system');
  }

  async getAdminUserStats(): Promise<UserStats> {
    return this.request<UserStats>('/api/admin/users/stats');
  }

  async getRepositoryStats(): Promise<RepositoryStats> {
    return this.request<RepositoryStats>('/api/admin/repositories/stats');
  }


  async getAdminSearchStats(): Promise<SearchStats> {
    return this.request<SearchStats>('/api/admin/search/stats');
  }

  async getRecentActivity(): Promise<RecentActivity> {
    return this.request<RecentActivity>('/api/admin/activity/recent');
  }

  // Health Check
  async health(): Promise<{ status: string }> {
    return this.request<{ status: string }>('/health');
  }

  // Generic HTTP methods for hooks
  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint);
  }

  async post<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

}

// Create and export the API client instance
// Lazy instantiation to ensure runtime config is loaded
let _apiClient: ApiClient | null = null;

function getApiClient(): ApiClient {
  if (!_apiClient) {
    _apiClient = new ApiClient();
  }
  return _apiClient;
}

export const apiClient = new Proxy({} as ApiClient, {
  get(target, prop) {
    const client = getApiClient();
    const value = (client as any)[prop];
    return typeof value === 'function' ? value.bind(client) : value;
  }
});

// Export the class for testing
export { ApiClient };

// Utility functions for common API operations
export const api = {
  // Auth
  login: (credentials: LoginRequest) => apiClient.auth.login(credentials),
  register: (data: RegisterRequest) => apiClient.auth.register(data),
  getProfile: () => apiClient.auth.getProfile(),
  updateProfile: (data: UpdateProfileRequest) => apiClient.auth.updateProfile(data),
  uploadAvatar: (file: globalThis.File) => apiClient.auth.uploadAvatar(file),
  changePassword: (data: ChangePasswordRequest) => apiClient.auth.changePassword(data),
  getUserActivity: () => apiClient.auth.getUserActivity(),
  deleteAccount: (password: string) => apiClient.auth.deleteAccount(password),
  logout: () => apiClient.auth.logout(),

  // Search
  search: (query: SearchQuery) => apiClient.search(query),
  // Files
  getFile: (id: string) => apiClient.getFile(id),
  getFileByDocAddress: (docAddress: string) => apiClient.getFileByDocAddress(docAddress),
  getFiles: (params?: Parameters<typeof apiClient.getFiles>[0]) => apiClient.getFiles(params),

  // Repositories
  getRepositories: () => apiClient.getRepositories(),
  getRepositoriesWithStats: () => apiClient.getRepositoriesWithStats(),
  getRepository: (id: string) => apiClient.getRepository(id),
  createRepository: (data: CreateRepositoryRequest) => apiClient.createRepository(data),
  updateRepository: (id: string, data: Partial<CreateRepositoryRequest>) => 
    apiClient.updateRepository(id, data),
  deleteRepository: (id: string) => apiClient.deleteRepository(id),
  crawlRepository: (id: string) => apiClient.crawlRepository(id),
  stopCrawlRepository: (id: string) => apiClient.stopCrawlRepository(id),
  getRepositoryProgress: (id: string) => apiClient.getRepositoryProgress(id),
  getActiveProgress: () => apiClient.getActiveProgress(),
  updateRepositorySchedule: (id: string, data: ScheduleRepositoryRequest) => 
    apiClient.updateRepositorySchedule(id, data),
  getSchedulerStatus: () => apiClient.getSchedulerStatus(),

  // Health
  health: () => apiClient.health(),

  // Generic methods
  get: <T>(endpoint: string) => apiClient.get<T>(endpoint),
  post: <T>(endpoint: string, data?: any) => apiClient.post<T>(endpoint, data),

  // User Management
  getUsers: () => apiClient.getUsers(),
  getUser: (id: string) => apiClient.getUser(id),
  createUser: (data: CreateUserRequest) => apiClient.createUser(data),
  updateUser: (id: string, data: UpdateUserRequest) => apiClient.updateUser(id, data),
  updateUserRole: (id: string, role: string) => apiClient.updateUserRole(id, role),
  updateUserStatus: (id: string, active: boolean) => apiClient.updateUserStatus(id, active),
  deleteUser: (id: string) => apiClient.deleteUser(id),
  getUserStats: () => apiClient.getUserStats(),

  // Admin
  getAdminDashboard: () => apiClient.getAdminDashboard(),
  getSystemStats: () => apiClient.getSystemStats(),
  getAdminUserStats: () => apiClient.getAdminUserStats(),
  getRepositoryStats: () => apiClient.getRepositoryStats(),
  getAdminSearchStats: () => apiClient.getAdminSearchStats(),
  getRecentActivity: () => apiClient.getRecentActivity(),
};

// Error helper functions
export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

export function getErrorMessage(error: unknown): string {
  if (isApiError(error)) {
    return error.message;
  }
  
  if (error instanceof Error) {
    return error.message;
  }
  
  // Handle TanStack Query error objects
  if (error && typeof error === 'object') {
    if ('message' in error && typeof error.message === 'string') {
      return error.message;
    }
    
    // Try to extract message from nested error objects
    if ('error' in error && error.error instanceof Error) {
      return error.error.message;
    }
    
    // Handle response error objects
    if ('response' in error && error.response && typeof error.response === 'object') {
      const response = error.response as any;
      if (response.data && typeof response.data === 'object' && response.data.message) {
        return response.data.message;
      }
      if (response.statusText) {
        return response.statusText;
      }
    }
    
    // Safely convert object to string without causing conversion errors
    try {
      return JSON.stringify(error);
    } catch {
      return 'An unexpected error occurred';
    }
  }
  
  return 'An unexpected error occurred';
}

// Token utility functions
export function decodeToken(token: string): any {
  try {
    const payload = token.split('.')[1];
    const decoded = atob(payload);
    return JSON.parse(decoded);
  } catch {
    return null;
  }
}

export function isTokenExpired(token: string): boolean {
  const decoded = decodeToken(token);
  if (!decoded || !decoded.exp) return true;
  
  return Date.now() >= decoded.exp * 1000;
}

export function getTokenExpirationDate(token: string): Date | null {
  const decoded = decodeToken(token);
  if (!decoded || !decoded.exp) return null;
  
  return new Date(decoded.exp * 1000);
}