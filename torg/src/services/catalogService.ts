import apiClient from '../api/client';
import type { Product, ProductCategory, PaginatedResult } from '../types';
import { MOCK_PRODUCTS, MOCK_CATEGORIES, MOCK_COUPONS } from './mockData';

export interface GetProductsParams {
  category?: string;
  query?: string;
  minPrice?: number;
  maxPrice?: number;
  sortBy?: string;
  page?: number;
  pageSize?: number;
}

export const catalogService = {
  /**
   * Fetch paginated products with filters and search
   */
  async getProducts(params?: GetProductsParams): Promise<PaginatedResult<Product>> {
    try {
      const response = await apiClient.get<PaginatedResult<Product>>('/catalog/products', { params });
      if (response.data && response.data.items) {
        return response.data;
      }
    } catch {
      // Fallback to local mock catalog data
    }

    // Filter mock data locally
    let filtered = [...MOCK_PRODUCTS];

    if (params?.category && params.category !== 'all') {
      filtered = filtered.filter(p => p.categorySlug === params.category);
    }

    if (params?.query && params.query.trim() !== '') {
      const q = params.query.toLowerCase().trim();
      filtered = filtered.filter(p => 
        p.name.toLowerCase().includes(q) || 
        p.description.toLowerCase().includes(q) ||
        p.category.toLowerCase().includes(q) ||
        p.tags?.some(tag => tag.toLowerCase().includes(q))
      );
    }

    if (params?.minPrice !== undefined) {
      filtered = filtered.filter(p => p.price >= params.minPrice!);
    }

    if (params?.maxPrice !== undefined) {
      filtered = filtered.filter(p => p.price <= params.maxPrice!);
    }

    // Sorting
    if (params?.sortBy) {
      switch (params.sortBy) {
        case 'price-asc':
          filtered.sort((a, b) => a.price - b.price);
          break;
        case 'price-desc':
          filtered.sort((a, b) => b.price - a.price);
          break;
        case 'rating':
          filtered.sort((a, b) => b.rating - a.rating);
          break;
        case 'newest':
          filtered.sort((a, b) => (b.isNew ? 1 : 0) - (a.isNew ? 1 : 0));
          break;
        default:
          // featured
          filtered.sort((a, b) => (b.isFeatured ? 1 : 0) - (a.isFeatured ? 1 : 0));
      }
    }

    const page = params?.page || 1;
    const pageSize = params?.pageSize || 12;
    const total = filtered.length;
    const totalPages = Math.ceil(total / pageSize);
    const paginatedItems = filtered.slice((page - 1) * pageSize, page * pageSize);

    return {
      items: paginatedItems,
      total,
      page,
      pageSize,
      totalPages,
    };
  },

  /**
   * Fetch single product by slug or id
   */
  async getProductBySlug(slug: string): Promise<Product | null> {
    try {
      const response = await apiClient.get<Product>(`/catalog/products/${slug}`);
      if (response.data) return response.data;
    } catch {
      // Fallback
    }

    const found = MOCK_PRODUCTS.find(p => p.slug === slug || p.id === slug);
    return found || null;
  },

  /**
   * Fetch all categories
   */
  async getCategories(): Promise<ProductCategory[]> {
    try {
      const response = await apiClient.get<ProductCategory[]>('/catalog/categories');
      if (response.data && Array.isArray(response.data)) return response.data;
    } catch {
      // Fallback
    }

    return MOCK_CATEGORIES;
  },

  /**
   * Fetch featured showcase products for vitrine
   */
  async getFeaturedProducts(): Promise<Product[]> {
    try {
      const response = await apiClient.get<Product[]>('/catalog/featured');
      if (response.data && Array.isArray(response.data)) return response.data;
    } catch {
      // Fallback
    }

    return MOCK_PRODUCTS.filter(p => p.isFeatured);
  },

  /**
   * Validate discount coupon code
   */
  async validateCoupon(code: string) {
    try {
      const response = await apiClient.post('/marketing/coupons/validate', { code });
      return response.data;
    } catch {
      // Fallback check against mock coupons
      const match = MOCK_COUPONS.find(c => c.code.toUpperCase() === code.trim().toUpperCase());
      if (match) return match;
      throw new Error('Cupom inválido ou expirado');
    }
  }
};

export default catalogService;
