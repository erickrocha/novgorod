import { api } from "./api";
import type { PagedResult, PageQueryParams, Warehouse, WarehouseInput } from "./types";

export const warehouseService = {
  async list() {
    return (await api.get<Warehouse[]>("/warehouses")).data;
  },
  async paged(params?: PageQueryParams) {
    return (await api.get<PagedResult<Warehouse>>("/warehouses/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<Warehouse>(`/warehouses/${id}`)).data;
  },
  async create(data: WarehouseInput) {
    return (await api.post<Warehouse>("/warehouses", data)).data;
  },
  async update(id: number, data: WarehouseInput) {
    return (await api.put<Warehouse>(`/warehouses/${id}`, data)).data;
  },
  async delete(id: number) {
    return (await api.delete(`/warehouses/${id}`)).data;
  },
};
