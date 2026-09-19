import { api } from "./api";
import type { PagedResult, PageQueryParams, TaxRule, TaxRuleInput } from "./types";

export const taxRuleService = {
  async list() {
    return (await api.get<TaxRule[]>("/tax-rules")).data;
  },
  async paged(params?: PageQueryParams) {
    return (await api.get<PagedResult<TaxRule>>("/tax-rules/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<TaxRule>(`/tax-rules/${id}`)).data;
  },
  async create(data: TaxRuleInput) {
    return (await api.post<TaxRule>("/tax-rules", data)).data;
  },
  async update(id: number, data: TaxRuleInput) {
    return (await api.put<TaxRule>(`/tax-rules/${id}`, data)).data;
  },
};
