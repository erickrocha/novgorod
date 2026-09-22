import axios, { AxiosError, type InternalAxiosRequestConfig } from 'axios';
import type { ApiError } from '../types/api';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  },
});

// Request Interceptor
apiClient.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    // Automatically attach auth token if present
    const token = localStorage.getItem('torg_customer_token');
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    // Attach current active session or tenant id if needed
    const tenantId = localStorage.getItem('novgorod_tenant_id');
    if (tenantId && config.headers) {
      config.headers['X-Tenant-Id'] = tenantId;
    }

    return config;
  },
  (error: unknown) => Promise.reject(error)
);

// Response Interceptor
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    const customError: ApiError = {
      message: error.message || 'Ocorreu um erro inesperado na comunicação.',
      status: error.response?.status,
      code: error.code,
    };

    if (error.response?.data && typeof error.response.data === 'object') {
      const data = error.response.data as Record<string, unknown>;
      if (typeof data.message === 'string') {
        customError.message = data.message;
      }
      if (typeof data.errors === 'object') {
        customError.errors = data.errors as Record<string, string[]>;
      }
    }

    return Promise.reject(customError);
  }
);

export default apiClient;
