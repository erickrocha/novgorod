import { api } from "./api";
import type { PageQueryParams, PagedResult, SkuStock, SkuStockInput } from "./types";

export const skuStockService = {
  async list() {
    return (await api.get<SkuStock[]>("/sku-stocks")).data;
  },
  async paged(params?: PageQueryParams & { skuId?: number }) {
    return (await api.get<PagedResult<SkuStock>>("/sku-stocks/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<SkuStock>(`/sku-stocks/${id}`)).data;
  },
  async bySku(skuId: number) {
    return (await api.get<SkuStock>(`/sku-stocks/by-sku/${skuId}`)).data;
  },
  async create(data: SkuStockInput) {
    return (await api.post<SkuStock>("/sku-stocks", data)).data;
  },
  async update(id: number, data: SkuStockInput) {
    return (await api.put<SkuStock>(`/sku-stocks/${id}`, data)).data;
  },
  async delete(id: number) {
    return (await api.delete(`/sku-stocks/${id}`)).data;
  },
};
