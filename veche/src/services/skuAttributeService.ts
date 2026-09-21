import { api } from "./api";
import type { PageQueryParams, PagedResult, SkuAttributeValue, SkuAttributeValueInput } from "./types";

export const skuAttributeService = {
  async list() {
    return (await api.get<SkuAttributeValue[]>("/sku-attributes")).data;
  },
  async paged(params?: PageQueryParams & { productId?: number; skuId?: number; attributeId?: number }) {
    return (await api.get<PagedResult<SkuAttributeValue>>("/sku-attributes/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<SkuAttributeValue>(`/sku-attributes/${id}`)).data;
  },
  async bySku(skuId: number) {
    return (await api.get<SkuAttributeValue[]>(`/sku-attributes/by-sku/${skuId}`)).data;
  },
  async byProduct(productId: number) {
    return (await api.get<SkuAttributeValue[]>(`/sku-attributes/by-product/${productId}`)).data;
  },
  async create(data: SkuAttributeValueInput) {
    return (await api.post<SkuAttributeValue>("/sku-attributes", data)).data;
  },
  async update(id: number, data: SkuAttributeValueInput) {
    return (await api.put<SkuAttributeValue>(`/sku-attributes/${id}`, data)).data;
  },
  async delete(id: number) {
    return (await api.delete(`/sku-attributes/${id}`)).data;
  },
};
