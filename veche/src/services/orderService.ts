import { api } from "./api";
import type {
  OrderItem,
  OrderItemInput,
  Orders,
  OrdersInput,
  OrderStatusHistory,
  OrderStatusHistoryInput,
  PagedResult,
  PageQueryParams,
} from "./types";

export const orderService = {
  // Orders
  async list() {
    return (await api.get<Orders[]>("/orders")).data;
  },
  async paged(params?: PageQueryParams & { customerId?: number; status?: string }) {
    return (await api.get<PagedResult<Orders>>("/orders/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<Orders>(`/orders/${id}`)).data;
  },
  async create(data: OrdersInput) {
    return (await api.post<Orders>("/orders", data)).data;
  },
  async update(id: number, data: OrdersInput) {
    return (await api.put<Orders>(`/orders/${id}`, data)).data;
  },

  // Order Items
  async itemsList() {
    return (await api.get<OrderItem[]>("/order-items")).data;
  },
  async itemsPaged(params?: PageQueryParams & { orderId?: number }) {
    return (await api.get<PagedResult<OrderItem>>("/order-items/paged", { params })).data;
  },
  async getItemById(id: number) {
    return (await api.get<OrderItem>(`/order-items/${id}`)).data;
  },
  async createItem(data: OrderItemInput) {
    return (await api.post<OrderItem>("/order-items", data)).data;
  },
  async updateItem(id: number, data: OrderItemInput) {
    return (await api.put<OrderItem>(`/order-items/${id}`, data)).data;
  },

  // Status History
  async statusHistoriesList() {
    return (await api.get<OrderStatusHistory[]>("/order-status-histories")).data;
  },
  async statusHistoriesPaged(params?: PageQueryParams & { orderId?: number }) {
    return (
      await api.get<PagedResult<OrderStatusHistory>>("/order-status-histories/paged", {
        params,
      })
    ).data;
  },
  async getStatusHistoryById(id: number) {
    return (await api.get<OrderStatusHistory>(`/order-status-histories/${id}`)).data;
  },
  async addStatusHistory(data: OrderStatusHistoryInput) {
    return (await api.post<OrderStatusHistory>("/order-status-histories", data)).data;
  },
};
