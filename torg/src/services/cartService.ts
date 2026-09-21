import apiClient from '../api/client';
import type { CartItem, CartSummary } from '../types';

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
   * Calculate shipping quote based on postal code
   */
  async estimateShipping(cep: string, totalWeightKg = 1): Promise<{ cost: number; deliveryDays: number; service: string }> {
    try {
      const response = await apiClient.post('/shipping/calculate', { cep, weight: totalWeightKg });
      return response.data;
    } catch {
      // Mock calculation based on CEP
      const cleanCep = cep.replace(/\D/g, '');
      const isExpress = cleanCep.endsWith('0') || cleanCep.endsWith('5');
      return {
        cost: isExpress ? 35.00 : 22.50,
        deliveryDays: isExpress ? 2 : 5,
        service: isExpress ? 'Novgorod Express' : 'Entrega Padrão',
      };
    }
  }
};

export default cartService;
