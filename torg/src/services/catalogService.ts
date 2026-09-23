import apiClient from '../api/client';
import type { Product, ProductCategory, PaginatedResult, ProductSku, ProductVariant } from '../types';

export interface GetProductsParams {
  category?: string;
  query?: string;
  minPrice?: number;
  maxPrice?: number;
  sortBy?: string;
  cursor?: number | null;
  limit?: number;
}

// Map summary product representation to frontend UI model
const mapProductJsonToProduct = (item: any): Product => ({
  id: String(item.id),
  slug: item.slug || String(item.id),
  name: item.name,
  description: item.description || '',
  price: item.priceCents ? item.priceCents / 100 : 99.90, 
  originalPrice: item.compareAtPriceCents ? item.compareAtPriceCents / 100 : undefined,
  currency: 'BRL',
  category: item.categorySlugs?.[0] ?? 'Vinhos',
  categorySlug: item.categorySlugs?.[0] ?? 'vinhos',
  images: item.primaryImageUrl ? [item.primaryImageUrl] : [],
  thumbnail: item.primaryImageUrl ?? 'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?q=80&w=600&auto=format&fit=crop',
  rating: item.rating ?? 4.8,
  reviewCount: item.reviewCount ?? 12,
  stock: item.stock ?? 10,
  isFeatured: item.isFeatured ?? false,
  isNew: item.isNew ?? false,
  attributes: {
    brand: item.brand,
    ...item.attributes
  }
});

// Map full product detail representation from /api/public/products/{slug}
const mapProductDetailJsonToProduct = (item: any): Product => {
  const images = (item.images || []).map((img: any) => img.url);
  const primaryImage = (item.images || []).find((img: any) => img.isPrimary)?.url || images[0] || 'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?q=80&w=600&auto=format&fit=crop';

  const skus: ProductSku[] = (item.skus || []).map((s: any) => ({
    id: s.id,
    uuid: s.uuid,
    code: s.code,
    variantKey: s.variantKey,
    priceCents: s.priceCents,
    compareAtPriceCents: s.compareAtPriceCents,
    weightG: s.weightG,
    widthMm: s.widthMm,
    heightMm: s.heightMm,
    lengthMm: s.lengthMm,
    active: s.active,
    stock: s.stock ?? 0,
    attributes: (s.attributes || []).map((a: any) => ({
      attributeId: a.attributeId,
      name: a.name,
      value: a.value,
    })),
  }));

  const variants: ProductVariant[] = skus.map((s) => ({
    id: String(s.id),
    name: s.variantKey || s.code,
    sku: s.code,
    price: s.priceCents / 100,
    originalPrice: s.compareAtPriceCents ? s.compareAtPriceCents / 100 : undefined,
    stock: s.stock,
    attributes: s.attributes.reduce((acc, curr) => {
      acc[curr.name] = curr.value;
      return acc;
    }, {} as Record<string, string>),
  }));

  const specsMap: Record<string, string> = {};
  if (item.brand) {
    specsMap['Marca'] = item.brand;
  }
  if (skus.length > 0 && skus[0].attributes) {
    for (const a of skus[0].attributes) {
      specsMap[a.name] = a.value;
    }
  }

  const primarySku = skus[0];
  const price = primarySku ? primarySku.priceCents / 100 : 99.90;
  const originalPrice = primarySku?.compareAtPriceCents ? primarySku.compareAtPriceCents / 100 : undefined;
  const totalStock = skus.reduce((sum, s) => sum + s.stock, 0);

  return {
    id: String(item.id),
    slug: item.slug || String(item.id),
    name: item.name,
    description: item.description || '',
    price,
    originalPrice,
    currency: 'BRL',
    category: item.categorySlugs?.[0] ?? 'Vinhos',
    categorySlug: item.categorySlugs?.[0] ?? 'vinhos',
    images: images.length > 0 ? images : [primaryImage],
    thumbnail: primaryImage,
    rating: item.rating ?? 4.8,
    reviewCount: item.reviewCount ?? 24,
    stock: totalStock,
    isFeatured: item.isFeatured ?? false,
    isNew: item.isNew ?? false,
    attributes: {
      brand: item.brand,
      ...specsMap,
    },
    variants,
    seller: item.seller ? {
      id: item.seller.id,
      businessName: item.seller.businessName,
      companyName: item.seller.companyName,
      email: item.seller.email,
      phone: item.seller.phone,
      webSite: item.seller.webSite,
      locality: item.seller.locality,
      administrativeArea: item.seller.administrativeArea,
      postalCode: item.seller.postalCode,
      countryCode: item.seller.countryCode,
    } : undefined,
    detailImages: item.images || [],
    skus,
    productAttributes: item.attributes || [],
  };
};

export const catalogService = {
  /**
   * Fetch paginated products with filters and search
   */
  async getProducts(params?: GetProductsParams): Promise<PaginatedResult<Product>> {
    const limit = params?.limit || 12;

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
   * Fetch single product detail by slug or id from /api/public/products/{slug}
   */
  async getProductBySlug(slug: string): Promise<Product | null> {
    try {
      const response = await apiClient.get<any>(`/api/public/products/${slug}`);
      if (response.data) {
        return mapProductDetailJsonToProduct(response.data);
      }
      return null;
    } catch {
      // Fallback query if slug not directly matched
      try {
        const fallbackRes = await apiClient.get<PaginatedResult<any>>(`/api/public/products/query`, { params: { q: slug, limit: 1 } });
        const item = fallbackRes.data.items?.[0];
        return item ? mapProductJsonToProduct(item) : null;
      } catch {
        return null;
      }
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
      itemCount: 10,
    }));
  },

  /**
   * Fetch featured showcase products for vitrine
   */
  async getFeaturedProducts(): Promise<Product[]> {
    try {
      const response = await apiClient.get<PaginatedResult<any>>('/api/public/products/query', { params: { limit: 4 } });
      return (response.data.items || []).map(mapProductJsonToProduct);
    } catch {
      return [];
    }
  },

  /**
   * Validate discount coupon code
   */
  async validateCoupon(code: string) {
    const response = await apiClient.post('/marketing/coupons/validate', { code });
    return response.data;
  },
};

export default catalogService;
