import apiClient from '../api/client';
import type { Product, ProductCategory, PaginatedResult } from '../types';

export interface GetProductsParams {
  category?: string;
  query?: string;
  minPrice?: number;
  maxPrice?: number;
  sortBy?: string;
  cursor?: number | null;
  limit?: number;
}

// Map backend product representation to frontend UI model
// The new backend endpoint currently lacks some display fields like price and thumbnail
const mapProductJsonToProduct = (item: any): Product => ({
  id: String(item.id),
  slug: item.slug || String(item.id),
  name: item.name,
  description: item.description || '',
  price: item.priceCents ? item.priceCents / 100 : 99.90, 
  originalPrice: item.compareAtPriceCents ? item.compareAtPriceCents / 100 : undefined,
  currency: 'BRL',
  category: item.categorySlugs?.[0] ?? 'Geral', // Backend doesn't return full category name yet, so use slug or wait for category mapping
  categorySlug: item.categorySlugs?.[0] ?? 'geral',
  images: item.primaryImageUrl ? [item.primaryImageUrl] : [],
  thumbnail: item.primaryImageUrl ?? 'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?q=80&w=600&auto=format&fit=crop',
  rating: item.rating ?? 4.5,
  reviewCount: item.reviewCount ?? 12,
  stock: item.stock ?? 10,
  isFeatured: item.isFeatured ?? false,
  isNew: item.isNew ?? false,
  attributes: {
    brand: item.brand,
    ...item.attributes
  }
});

export const catalogService = {
  /**
   * Fetch paginated products with filters and search
   */
  async getProducts(params?: GetProductsParams): Promise<PaginatedResult<Product>> {
    const limit = params?.limit || 12; // Use 12 as requested originally

    // Translate frontend params to backend expected params
    const queryParams: Record<string, any> = {
      limit,
    };

    if (params?.cursor !== undefined && params?.cursor !== null) {
      queryParams.cursor = params.cursor;
    }

    if (params?.query) {
      queryParams.q = params.query;
    }

    if (params?.category) {
      queryParams.category = params.category;
    }

    if (params?.minPrice !== undefined) {
      queryParams.min_price = Math.round(params.minPrice * 100);
    }
    
    if (params?.maxPrice !== undefined) {
      queryParams.max_price = Math.round(params.maxPrice * 100);
    }

    if (params?.sortBy) {
      queryParams.sort_by = params.sortBy;
    }

    const response = await apiClient.get<PaginatedResult<any>>('/api/public/products/query', { params: queryParams });
    return {
      ...response.data,
      items: (response.data.items || []).map(mapProductJsonToProduct)
    };
  },

  /**
   * Fetch single product by slug or id
   */
  async getProductBySlug(slug: string): Promise<Product | null> {
    try {
      const response = await apiClient.get<PaginatedResult<any>>(`/api/public/products/query`, { params: { q: slug, limit: 1 } });
      const item = response.data.items?.[0];
      return item ? mapProductJsonToProduct(item) : null;
    } catch {
      return null;
    }
  },

  /**
   * Fetch all categories
   */
  async getCategories(): Promise<ProductCategory[]> {
    const response = await apiClient.get<any[]>('/api/public/categories');
    return response.data.map(c => ({
      id: String(c.id),
      name: c.name,
      slug: c.slug,
      itemCount: 10, // Mock count until backend supports it
    }));
  },

  /**
   * Fetch featured showcase products for vitrine
   */
  async getFeaturedProducts(): Promise<Product[]> {
    // Calling the paged products with a small limit and default sorting
    const response = await apiClient.get<PaginatedResult<any>>('/api/public/products/query', {
      params: { limit: 10 }
    });
    return (response.data.items || []).map(mapProductJsonToProduct);
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
