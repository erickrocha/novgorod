import { api } from "./api";
import type { City, ImportReport, PagedResult, PageQueryParams, Province } from "./types";

export const locationService = {
  async provinces(): Promise<Province[]> { return (await api.get<Province[]>("/province", { params: { countryCode: "BR" } })).data; },
  async provincesPaged(params?: PageQueryParams): Promise<PagedResult<Province>> {
    const finalParams = { countryCode: "BR", ...params };
    return (await api.get<PagedResult<Province>>("/province/paged", { params: finalParams })).data;
  },
  async cities(): Promise<City[]> { return (await api.get<City[]>("/cities")).data; },
  async citiesPaged(params?: PageQueryParams): Promise<PagedResult<City>> {
    return (await api.get<PagedResult<City>>("/cities/paged", { params })).data;
  },
  async createProvince(data: Province) { return (await api.post<Province>("/province", data)).data; },
  async updateProvince(id: number, data: Province) { return (await api.put<Province>(`/province/${id}`, data)).data; },
  async createCity(data: City) { return (await api.post<City>("/city", data)).data; },
  async updateCity(id: number, data: City) { return (await api.put<City>(`/city/${id}`, data)).data; },
  async importProvinces(file: File) { const body = new FormData(); body.append("file", file); return (await api.post<ImportReport>("/province/import", body, { headers: { "Content-Type": "multipart/form-data" } })).data; },
  async importCities(file: File) { const body = new FormData(); body.append("file", file); return (await api.post<ImportReport>("/city/import", body, { headers: { "Content-Type": "multipart/form-data" } })).data; },
};
