import { api } from "./api";
import type { AuthResponse } from "./types";

export const authService = {
  async login(email: string, password: string): Promise<AuthResponse> {
    const formData = new URLSearchParams();
    formData.append("email", email);
    formData.append("password", password);

    const response = await api.post<AuthResponse>("/login", formData, {
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
    });
    return response.data;
  },

  async refreshToken(refreshToken: string): Promise<AuthResponse> {
    const response = await api.post<AuthResponse>("/refresh", { refreshToken });
    return response.data;
  },
};
