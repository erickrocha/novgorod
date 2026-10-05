import { api } from "./api";
import type { PagedResult, PageQueryParams, Tenant, TenantInput, TenantPlan, TenantPlanInput } from "./types";

export interface TenantListing {
  listed: boolean;
}

export const tenantService = {
  async getListing(id: number): Promise<TenantListing> {
    const response = await api.get<TenantListing>(`/tenant/${id}/listing`);
    return response.data;
  },

  async setListing(id: number, listed: boolean): Promise<TenantListing> {
    const response = await api.put<TenantListing>(`/tenant/${id}/listing`, { listed });
    return response.data;
  },

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
