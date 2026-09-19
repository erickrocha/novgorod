import { api } from "./api";
import type { Cart, CartInput, CartItem, CartItemInput, PagedResult, PageQueryParams } from "./types";

export const cartService = {
  // Carts
  async list() {
    return (await api.get<Cart[]>("/carts")).data;
  },
  async paged(params?: PageQueryParams & { customerId?: number; status?: string }) {
    return (await api.get<PagedResult<Cart>>("/carts/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<Cart>(`/carts/${id}`)).data;
  },
  async create(data: CartInput) {
    return (await api.post<Cart>("/carts", data)).data;
  },
  async update(id: number, data: CartInput) {
    return (await api.put<Cart>(`/carts/${id}`, data)).data;
  },

  // Cart Items
  async itemsList() {
    return (await api.get<CartItem[]>("/cart-items")).data;
  },
  async itemsPaged(params?: PageQueryParams & { cartId?: number }) {
    return (await api.get<PagedResult<CartItem>>("/cart-items/paged", { params })).data;
  },
  async getItemById(id: number) {
    return (await api.get<CartItem>(`/cart-items/${id}`)).data;
  },
  async createItem(data: CartItemInput) {
    return (await api.post<CartItem>("/cart-items", data)).data;
  },
  async updateItem(id: number, data: CartItemInput) {
    return (await api.put<CartItem>(`/cart-items/${id}`, data)).data;
  },
};
