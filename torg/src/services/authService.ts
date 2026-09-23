import apiClient from '../api/client';

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
