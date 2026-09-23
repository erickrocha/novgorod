export interface ProductCategory {
  id: string;
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  imageUrl?: string;
  itemCount?: number;
}

export interface ProductVariant {
  id: string;
  name: string;
  sku: string;
  price: number;
  originalPrice?: number;
  stock: number;
  attributes?: Record<string, string>;
}

export interface Seller {
  id: number;
  businessName: string;
  companyName?: string;
  email?: string;
  phone?: string;
  webSite?: string;
  locality?: string;
  administrativeArea?: string;
  postalCode?: string;
  countryCode?: string;
}

export interface ProductDetailImage {
  id: number;
  url: string;
  altText?: string;
  sortOrder: number;
  isPrimary: boolean;
  widthPx?: number;
  heightPx?: number;
}

export interface SkuAttributeValue {
  attributeId: number;
  name: string;
  value: string;
}

export interface ProductSku {
  id: number;
  uuid: string;
  code: string;
  variantKey: string;
  priceCents: number;
  compareAtPriceCents?: number;
  weightG?: number;
  widthMm?: number;
  heightMm?: number;
  lengthMm?: number;
  active: boolean;
  stock: number;
  attributes: SkuAttributeValue[];
}

export interface ProductAttribute {
  id: number;
  attributeId: number;
  name: string;
  displayType: string;
  required: boolean;
  sortOrder: number;
}

export interface Product {
  id: string;
  slug: string;
  name: string;
  subtitle?: string;
  description: string;
  price: number;
  originalPrice?: number;
  currency: string;
  category: string;
  categorySlug: string;
  images: string[];
  thumbnail: string;
  rating: number;
  reviewCount: number;
  stock: number;
  isFeatured?: boolean;
  isNew?: boolean;
  tags?: string[];
  attributes?: {
    brand?: string;
    origin?: string;
    year?: number;
    weight?: string;
    sku?: string;
    [key: string]: unknown;
  };
  variants?: ProductVariant[];
  seller?: Seller;
  detailImages?: ProductDetailImage[];
  skus?: ProductSku[];
  productAttributes?: ProductAttribute[];
}

export interface ProductFilterState {
  searchQuery: string;
  category: string;
  minPrice: number;
  maxPrice: number;
  sortBy: 'featured' | 'price-asc' | 'price-desc' | 'rating' | 'newest';
  inStockOnly: boolean;
}
