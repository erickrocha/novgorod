import apiClient from '../api/client';
import type { Product, ProductCategory, PaginatedResult, ProductSku, ProductVariant } from '../types';
import type { ApiError } from '../types/api';

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
  price: item.priceCents != null ? item.priceCents / 100 : 0,
  originalPrice: item.compareAtPriceCents ? item.compareAtPriceCents / 100 : undefined,
  currency: 'BRL',
  category: item.categorySlugs?.[0] ?? '',
  categorySlug: item.categorySlugs?.[0] ?? '',
  images: item.primaryImageUrl ? [item.primaryImageUrl] : [],
  thumbnail: item.primaryImageUrl ?? undefined,
  // rating and review count only when kremlin sends them; the summary view has no stock
  rating: item.rating ?? undefined,
  reviewCount: item.reviewCount ?? undefined,
  isFeatured: item.isFeatured ?? false,
  isNew: item.isNew ?? false,
  seller: item.seller ?? null,
  attributes: {
    brand: item.brand,
    ...item.attributes
  }
});

// Map full product detail representation from /api/public/products/{slug}
const mapProductDetailJsonToProduct = (item: any): Product => {
  const images = (item.images || []).map((img: any) => img.url);
  const primaryImage: string | undefined = (item.images || []).find((img: any) => img.isPrimary)?.url || images[0];

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
  const price = primarySku ? primarySku.priceCents / 100 : 0;
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
    category: item.categorySlugs?.[0] ?? '',
    categorySlug: item.categorySlugs?.[0] ?? '',
    images,
    thumbnail: primaryImage,
    rating: item.rating ?? undefined,
    reviewCount: item.reviewCount ?? undefined,
    stock: totalStock,
    isFeatured: item.isFeatured ?? false,
    isNew: item.isNew ?? false,
    attributes: {
      brand: item.brand,
      ...specsMap,
    },
    variants,
    seller: item.seller?.id > 0 ? {
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
    } : null,
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
      queryParams.minPrice = Math.round(params.minPrice * 100);
    }
    
    if (params?.maxPrice !== undefined) {
      queryParams.maxPrice = Math.round(params.maxPrice * 100);
    }

    if (params?.sortBy) {
      queryParams.sortBy = params.sortBy;
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
      const response = await apiClient.get<any>(`/api/public/products/${encodeURIComponent(slug)}`);
      return response.data ? mapProductDetailJsonToProduct(response.data) : null;
    } catch (error) {
      // only "not found" means no such product; any other failure is an error the caller must show
      if ((error as ApiError).status === 404) return null;
      throw error;
    }
  },

  /**
   * Fetch all categories
   */
  async getCategories(): Promise<ProductCategory[]> {
    const response = await apiClient.get<any[]>('/api/public/categories');
    const slugs = new Set<string>();
    return response.data.filter(c => {
      if (c.active === false || slugs.has(c.slug)) return false;
      slugs.add(c.slug);
      return true;
    }).map(c => ({ id: String(c.id), name: c.name, slug: c.slug }));
  },

  /**
   * Fetch featured showcase products for vitrine
   */
  async getFeaturedProducts(): Promise<Product[]> {
    const response = await apiClient.get<PaginatedResult<any>>('/api/public/products/query', { params: { limit: 4 } });
    return (response.data.items || []).map(mapProductJsonToProduct);
  },

};

export default catalogService;
