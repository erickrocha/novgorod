import { api } from "./api";
import type { PageQueryParams, PagedResult, ProductCategory, ProductCategoryInput } from "./types";

export const productCategoryService = {
  async list() {
    return (await api.get<ProductCategory[]>("/product-categories")).data;
  },
  async paged(params?: PageQueryParams & { productId?: number; categoryId?: number; isPrimary?: boolean }) {
    return (await api.get<PagedResult<ProductCategory>>("/product-categories/paged", { params })).data;
  },
  async getById(id: number) {
    return (await api.get<ProductCategory>(`/product-categories/${id}`)).data;
  },
  async byProduct(productId: number) {
    return (await api.get<ProductCategory[]>(`/product-categories/by-product/${productId}`)).data;
  },
  async byCategory(categoryId: number) {
    return (await api.get<ProductCategory[]>(`/product-categories/by-category/${categoryId}`)).data;
  },
  async create(data: ProductCategoryInput) {
    return (await api.post<ProductCategory>("/product-categories", data)).data;
  },
  async update(id: number, data: ProductCategoryInput) {
    return (await api.put<ProductCategory>(`/product-categories/${id}`, data)).data;
  },
  async delete(id: number) {
    return (await api.delete(`/product-categories/${id}`)).data;
  },
};
