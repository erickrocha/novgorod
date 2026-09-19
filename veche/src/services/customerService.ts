import { api } from "./api";
import type {
  Customer,
  CustomerAddress,
  CustomerAddressInput,
  CustomerInput,
  PagedResult,
  PageQueryParams,
} from "./types";

export const customerService = {
  // Customers
  async list() {
    return (await api.get<Customer[]>("/customers")).data;
  },
  async paged(params?: PageQueryParams) {
    return (await api.get<PagedResult<Customer>>("/customers/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<Customer>(`/customers/${id}`)).data;
  },
  async create(data: CustomerInput) {
    return (await api.post<Customer>("/customers", data)).data;
  },
  async update(id: number, data: CustomerInput) {
    return (await api.put<Customer>(`/customers/${id}`, data)).data;
  },

  // Addresses
  async addressesList() {
    return (await api.get<CustomerAddress[]>("/customer-addresses")).data;
  },
  async addressesPaged(params?: PageQueryParams & { customerId?: number }) {
    return (
      await api.get<PagedResult<CustomerAddress>>("/customer-addresses/paged", { params })
    ).data;
  },
  async getAddressById(id: number) {
    return (await api.get<CustomerAddress>(`/customer-addresses/${id}`)).data;
  },
  async createAddress(data: CustomerAddressInput) {
    return (await api.post<CustomerAddress>("/customer-addresses", data)).data;
  },
  async updateAddress(id: number, data: CustomerAddressInput) {
    return (await api.put<CustomerAddress>(`/customer-addresses/${id}`, data)).data;
  },
};
