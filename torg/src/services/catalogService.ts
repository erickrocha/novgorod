import apiClient from '../api/client';
import type { Product, ProductCategory, PaginatedResult } from '../types';

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
    const page = params?.page || 1;
    const pageSize = params?.pageSize || 25; // Default to 25 items as requested

    // Translate frontend params to backend expected params
    const queryParams: Record<string, any> = {
      page,
      page_size: pageSize,
    };

    if (params?.query) {
      queryParams.q = params.query;
    }

    // Note: If you need to filter by category, brand, etc., add them here based on backend support.
    // Assuming backend takes sort_by and sort_dir
    if (params?.sortBy) {
      if (params.sortBy === 'price-asc') {
        queryParams.sort_by = 'price';
        queryParams.sort_dir = 'asc';
      } else if (params.sortBy === 'price-desc') {
        queryParams.sort_by = 'price';
        queryParams.sort_dir = 'desc';
      } else if (params.sortBy === 'rating') {
        queryParams.sort_by = 'rating';
        queryParams.sort_dir = 'desc';
      } else if (params.sortBy === 'newest') {
        queryParams.sort_by = 'created_at';
        queryParams.sort_dir = 'desc';
      }
    }

    const response = await apiClient.get<PaginatedResult<Product>>('/api/public/products/paged', { params: queryParams });
    return response.data;
  },

  /**
   * Fetch single product by slug or id
   */
  async getProductBySlug(slug: string): Promise<Product | null> {
    try {
      const response = await apiClient.get<PaginatedResult<Product>>(`/api/public/products/paged`, { params: { q: slug, page_size: 1 } });
      return response.data.items?.[0] || null;
    } catch {
      return null;
    }
  },

  /**
   * Fetch all categories
   */
  async getCategories(): Promise<ProductCategory[]> {
    const response = await apiClient.get<ProductCategory[]>('/api/public/categories');
    return response.data;
  },

  /**
   * Fetch featured showcase products for vitrine
   */
  async getFeaturedProducts(): Promise<Product[]> {
    // Calling the paged products with a small page_size and default sorting (or a specific featured filter if supported)
    const response = await apiClient.get<PaginatedResult<Product>>('/api/public/products/paged', {
      params: { page: 1, page_size: 10 }
    });
    return response.data.items || [];
  },

  /**
   * Validate discount coupon code
   */
  async validateCoupon(code: string) {
    const response = await apiClient.post('/marketing/coupons/validate', { code });
    return response.data;
  }
};

export default catalogService;
