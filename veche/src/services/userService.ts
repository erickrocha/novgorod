import { api } from "./api";
import type { User, UserInput } from "./types";

export const userService = {
  async getUsers(): Promise<User[]> {
    const response = await api.get<User[]>("/user");
    return response.data;
  },

  async getUserById(id: number): Promise<User> {
    const response = await api.get<User>(`/user/${id}`);
    return response.data;
  },

  async createUser(userData: UserInput): Promise<User> {
    const response = await api.post<User>("/user", userData);
    return response.data;
  },

  async updateUser(id: number, userData: UserInput): Promise<User> {
    const response = await api.put<User>(`/user/${id}`, userData);
    return response.data;
  },
};
