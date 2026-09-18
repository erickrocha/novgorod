import { api } from "./api";
import type { PagedResult, PageQueryParams, Tenant, TenantInput, TenantPlan, TenantPlanInput } from "./types";

export const tenantService = {
  async getTenants(): Promise<Tenant[]> {
    const response = await api.get<Tenant[]>("/tenant");
    return response.data;
  },

  async getTenantsPaged(params?: PageQueryParams): Promise<PagedResult<Tenant>> {
    const response = await api.get<PagedResult<Tenant>>("/tenant/paged", { params });
    return response.data;
  },

  async getTenantById(id: number): Promise<Tenant> {
    const response = await api.get<Tenant>(`/tenant/${id}`);
    return response.data;
  },

  async getTenantByUuid(uuid: string): Promise<Tenant> {
    const response = await api.get<Tenant>(`/tenant/uuid/${uuid}`);
    return response.data;
  },

  async createTenant(data: TenantInput): Promise<Tenant> {
    const response = await api.post<Tenant>("/tenant", data);
    return response.data;
  },

  async updateTenant(id: number, data: TenantInput): Promise<Tenant> {
    const response = await api.put<Tenant>(`/tenant/${id}`, data);
    return response.data;
  },

  async addTenantPlan(
    id: number,
    planData: TenantPlanInput,
  ): Promise<TenantPlan> {
    const response = await api.post<TenantPlan>(`/tenant/${id}/plan`, planData);
    return response.data;
  },

  async getTenantActivePlan(id: number): Promise<TenantPlan> {
    const response = await api.get<TenantPlan>(`/tenant/${id}/plan`);
    return response.data;
  },
};
