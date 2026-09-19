import { api } from "./api";
import type { PagedResult, PageQueryParams, ShippingRate, ShippingRateInput } from "./types";

export const shippingRateService = {
  async list() {
    return (await api.get<ShippingRate[]>("/shipping-rates")).data;
  },
  async paged(params?: PageQueryParams) {
    return (await api.get<PagedResult<ShippingRate>>("/shipping-rates/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<ShippingRate>(`/shipping-rates/${id}`)).data;
  },
  async create(data: ShippingRateInput) {
    return (await api.post<ShippingRate>("/shipping-rates", data)).data;
  },
  async update(id: number, data: ShippingRateInput) {
    return (await api.put<ShippingRate>(`/shipping-rates/${id}`, data)).data;
  },
};
