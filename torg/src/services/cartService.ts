import apiClient from '../api/client';
import type { CartItem, CartSummary, ShippingQuote } from '../types';

export const cartService = {
  /**
   * Sync cart with backend if logged in
   */
  async syncCart(items: CartItem[]): Promise<CartSummary | null> {
    try {
      const response = await apiClient.post<CartSummary>('/sales/cart/sync', { items });
      return response.data;
    } catch {
      return null;
    }
  },

  /**
   * Approximate shipping for an anonymous visitor: cheapest option of each seller, summed.
   * The delivery time is the slowest seller's. The authenticated checkout recalculates it.
   */
  async estimateShipping(cep: string, items: Array<{ skuId: number; quantity: number }>): Promise<ShippingQuote> {
    const { data } = await apiClient.post<{
      sellers: Array<{ tenantId: number; options: Array<{ serviceName: string; priceCents: number; transitDays: number | null }> }>;
    }>('/api/public/shipping/estimate', { cep, items });
    const cheapest = data.sellers.map(seller =>
      [...seller.options].sort((a, b) => a.priceCents - b.priceCents || (a.transitDays ?? Infinity) - (b.transitDays ?? Infinity))[0]);
    if (cheapest.length === 0 || cheapest.some(option => !option)) {
      throw new Error('Frete indisponível para este CEP');
    }
    const days = cheapest.map(option => option.transitDays).filter((d): d is number => d !== null);
    return {
      cost: cheapest.reduce((sum, option) => sum + option.priceCents, 0) / 100,
      deliveryDays: days.length ? Math.max(...days) : null,
      service: cheapest.length === 1 ? cheapest[0].serviceName : `${cheapest.length} vendedores`,
    };
  },
};

export default cartService;
