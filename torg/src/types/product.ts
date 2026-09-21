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
}

export interface ProductFilterState {
  searchQuery: string;
  category: string;
  minPrice: number;
  maxPrice: number;
  sortBy: 'featured' | 'price-asc' | 'price-desc' | 'rating' | 'newest';
  inStockOnly: boolean;
}
