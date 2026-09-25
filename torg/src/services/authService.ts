import apiClient from '../api/client';
import type { CustomerProfile } from '../store/slices/authSlice';

export interface AuthResponse {
  accessToken: string;
  tokenType: string;
  expireIn: number;
  refreshToken?: string;
  email: string;
  uuid: string;
  name: string;
  userId: number;
  role: string;
  tenantId?: number;
  firstLogin: boolean;
}

export interface SignupRequest {
  name: string;
  email: string;
  password: string;
  cpf?: string;
  phone?: string;
}

export interface CustomerResponse {
  id: number;
  uuid: string;
  name: string;
  email: string;
  cpf?: string;
  phone?: string;
}

export const authService = {
  async me(token?: string): Promise<CustomerProfile> {
    const response = await apiClient.get<CustomerProfile>('/customers/me', token ? { headers: { Authorization: `Bearer ${token}` } } : undefined);
    return { ...response.data, id: String(response.data.id) };
  },
  async completeCpf(cpf: string): Promise<CustomerProfile> {
    const response = await apiClient.post<CustomerProfile>('/customers/me/tax-id', { cpf: cpf.replace(/\D/g, '') });
    return { ...response.data, id: String(response.data.id) };
  },
  async login(identifier: string, password: string): Promise<AuthResponse> {
    const formData = new URLSearchParams();
    formData.append('email', identifier);
    formData.append('password', password);

    const response = await apiClient.post<AuthResponse>('/login', formData, {
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });
    return response.data;
  },

  async signup(data: SignupRequest): Promise<CustomerResponse> {
    const response = await apiClient.post<CustomerResponse>('/signup', data);
    return response.data;
  },
};

export default authService;
