import apiClient from '../api/client';

export interface DeliveryAddress {
  recipient: string;
  addressLine1: string;
  addressLine2?: string;
  locality: string;
  administrativeArea: string;
  postalCode: string;
  countryCode: string;
}
export interface CheckoutQuote {
  id: number;
  expiresAt: string;
  shippingAddress: DeliveryAddress;
  items: Array<{ skuId: number; tenantId: number; name: string; quantity: number; unitPriceCents: number; totalCents: number }>;
  sellers: Array<{ tenantId: number; subtotalCents: number; discountCents: number; shippingCents: number; totalCents: number; couponCode?: string }>;
  subtotalCents: number;
  discountCents: number;
  shippingCents: number;
  totalCents: number;
}
export interface Purchase {
  id: number;
  status: string;
  totalCents: number;
  orders: Array<{ number: string; tenantId: number }>;
  payments: Array<{ id: number; status: string }>;
}
export interface PaymentState { purchaseId: number; status: string; reference?: string }

const checkoutService = {
  async quote(input: { items: Array<{ skuId: number; quantity: number }>; addressId?: number; shippingAddress?: DeliveryAddress; coupons: Array<{ tenantId: number; code: string }> }) {
    return (await apiClient.post<CheckoutQuote>('/checkout/quotes', input)).data;
  },
  async createPurchase(quoteId: number, email: string, phone: string) {
    const key = crypto.randomUUID();
    return (await apiClient.post<Purchase>('/purchases', { quoteId, email, phone }, { headers: { 'Idempotency-Key': key } })).data;
  },
  async paymentConfig() {
    return (await apiClient.get<{ publicKey: string }>('/checkout/payment-config')).data;
  },
  async submit(purchaseId: number, input: { token: string; paymentMethodId: string; issuerId?: string; installments: number }) {
    return (await apiClient.post<PaymentState>(`/purchases/${purchaseId}/payments/submit`, input)).data;
  },
  async status(purchaseId: number) {
    return (await apiClient.get<PaymentState>(`/purchases/${purchaseId}/payments/status`)).data;
  },
};
export default checkoutService;
