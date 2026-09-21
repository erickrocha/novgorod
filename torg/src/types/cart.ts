import type { Product, ProductVariant } from './product';

export interface CartItem {
  id: string; // unique item line id, e.g. productId + variantId
  product: Product;
  variant?: ProductVariant;
  quantity: number;
  unitPrice: number;
  totalPrice: number;
}

export interface Coupon {
  code: string;
  discountType: 'percentage' | 'fixed';
  discountValue: number;
  description?: string;
  minimumAmount?: number;
}

export interface CartSummary {
  subtotal: number;
  discountAmount: number;
  shippingAmount: number;
  taxAmount: number;
  total: number;
  totalItems: number;
  appliedCoupon?: Coupon | null;
}
