import axios, {
  AxiosHeaders,
  type AxiosError,
  type InternalAxiosRequestConfig,
} from "axios";

import type { ApiErrorResponse, AuthResponse } from "./types";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

export const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    "Content-Type": "application/json",
    "Accept-Language": "pt-BR",
  },
});

api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem("token");
    if (token) {
      config.headers.set("Authorization", `Bearer ${token}`);
    }
    return config;
  },
  (error: unknown) => Promise.reject(error),
);

interface RetryableRequestConfig extends InternalAxiosRequestConfig {
  _retry?: boolean;
}

interface FailedRequest {
  resolve: (token: string) => void;
  reject: (reason?: unknown) => void;
}

let isRefreshing = false;
let failedQueue: FailedRequest[] = [];

const processQueue = (error: unknown | null, token: string | null = null) => {
  failedQueue.forEach((request) => {
    if (error) {
      request.reject(error);
    } else if (token) {
      request.resolve(token);
    } else {
      request.reject(new Error("Token refresh did not return an access token"));
    }
  });
  failedQueue = [];
};

const clearStoredAuth = () => {
  localStorage.removeItem("token");
  localStorage.removeItem("refreshToken");
  localStorage.removeItem("user");
};

api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as RetryableRequestConfig | undefined;

    if (
      error.response?.status === 401 &&
      originalRequest &&
      !originalRequest._retry &&
      !originalRequest.url?.includes("/login") &&
      !originalRequest.url?.includes("/refresh")
    ) {
      const refreshToken = localStorage.getItem("refreshToken");

      if (!refreshToken) {
        clearStoredAuth();
        return Promise.reject(error);
      }

      if (isRefreshing) {
        return new Promise<string>((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        }).then((token) => {
          originalRequest.headers.set("Authorization", `Bearer ${token}`);
          return api(originalRequest);
        });
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        const response = await api.post<AuthResponse>("/refresh", {
          refreshToken,
        });
        const data = response.data;
        const newToken = data.accessToken || data.access_token;
        const newRefreshToken =
          data.refreshToken || data.refresh_token || refreshToken;

        if (!newToken) {
          throw new Error("Token refresh did not return an access token");
        }

        localStorage.setItem("token", newToken);
        localStorage.setItem("refreshToken", newRefreshToken);
        localStorage.setItem("user", JSON.stringify(data));

        api.defaults.headers.common.Authorization = `Bearer ${newToken}`;
        originalRequest.headers = AxiosHeaders.from(originalRequest.headers);
        originalRequest.headers.set("Authorization", `Bearer ${newToken}`);

        processQueue(null, newToken);
        return api(originalRequest);
      } catch (refreshError: unknown) {
        processQueue(refreshError);
        clearStoredAuth();
        return Promise.reject(refreshError);
      } finally {
        isRefreshing = false;
      }
    }

    if (error.response?.status === 401) {
      clearStoredAuth();
    }

    return Promise.reject(error);
  },
);

export const getApiErrorMessage = (
  error: unknown,
  fallback: string,
): string => {
  if (axios.isAxiosError<ApiErrorResponse>(error)) {
    return error.response?.data?.message || fallback;
  }
  return fallback;
};

export { authService } from "./authService";
export { tenantService } from "./tenantService";
export { userService } from "./userService";
export type * from "./types";
