import type { Product, ProductVariant } from './product';

export interface CartItem {
  id: string; // unique item line id, e.g. productId + variantId
  product: Product;
  variant?: ProductVariant;
  quantity: number;
  unitPrice: number;
  totalPrice: number;
}

export interface ShippingQuote {
  cost: number;
  deliveryDays: number | null;
  /** Sellers the estimate covers; above one, the delivery time is the longest of them. */
  sellerCount?: number;
  service: string;
}

export interface CartSummary {
  subtotal: number;
  shippingAmount: number;
  taxAmount: number;
  total: number;
  totalItems: number;
}

