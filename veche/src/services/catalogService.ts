import { api } from "./api";
export interface Category { id?: number; uuid?: string; tenantId?: number | null; name: string; slug: string; parentId?: number | null; active: boolean }
export interface Product { id: number; uuid: string; tenantId?: number | null; name: string; slug: string; description?: string | null; brand?: string | null; active: boolean; ncm: string; cest?: string | null; origemMercadoria: number }
export interface Sku { id: number; uuid: string; tenantId?: number | null; productId: number; code: string; variantKey: string; priceCents: number; compareAtPriceCents?: number | null; weightG?: number | null; widthMm?: number | null; heightMm?: number | null; lengthMm?: number | null; active: boolean }
export type CategoryInput = Omit<Category, "id" | "uuid">;
export type ProductInput = Omit<Product, "id" | "uuid">;
export type SkuInput = Omit<Sku, "id" | "uuid">;
export const catalogService = {
  async categories() { return (await api.get<Category[]>("/categories")).data; },
  async products() { return (await api.get<Product[]>("/products")).data; },
  async skus() { return (await api.get<Sku[]>("/skus")).data; },
  async createCategory(data: CategoryInput) { return (await api.post<Category>("/categories", data)).data; },
  async updateCategory(id: number, data: CategoryInput) { return (await api.put<Category>(`/categories/${id}`, data)).data; },
  async createProduct(data: ProductInput) { return (await api.post<Product>("/products", data)).data; },
  async updateProduct(id: number, data: ProductInput) { return (await api.put<Product>(`/products/${id}`, data)).data; },
  async createSku(data: SkuInput) { return (await api.post<Sku>("/skus", data)).data; },
  async updateSku(id: number, data: SkuInput) { return (await api.put<Sku>(`/skus/${id}`, data)).data; },
};
