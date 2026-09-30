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
export interface ShippingOption {
  id: string;
  provider: string;
  serviceCode: string;
  serviceName: string;
  priceCents: number;
  transitDays?: number;
}

export interface ShippingSnapshot {
  configurationVersion: number;
  mode: 'fixed' | 'correios';
  originCep?: string | null;
  destinationCep: string;
  options: ShippingOption[];
  selectedOption: ShippingOption;
}

export interface ShippingSelection {
  tenantId: number;
  optionId: string;
}

export interface SellerQuote {
  tenantId: number;
  subtotalCents: number;
  discountCents: number;
  shippingCents: number;
  totalCents: number;
  couponCode?: string;
  shipping?: ShippingSnapshot;
}

export interface CheckoutQuote {
  id: number;
  expiresAt: string;
  shippingAddress: DeliveryAddress;
  items: Array<{ skuId: number; tenantId: number; name: string; quantity: number; unitPriceCents: number; totalCents: number }>;
  sellers: SellerQuote[];
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
  async selectShipping(quoteId: number, selections: ShippingSelection[]) {
    return (await apiClient.post<CheckoutQuote>(`/checkout/quotes/${quoteId}/shipping-selection`, selections)).data;
  },
  async createPurchase(quoteId: number, email: string, phone: string) {
    const key = crypto.randomUUID();
    return (await apiClient.post<Purchase>('/purchases', { quoteId, email, phone }, { headers: { 'Idempotency-Key': key } })).data;
  },
  async paymentConfig(params?: { tenantId?: number }) {
    return (await apiClient.get<{ provider?: string; publicKey: string }>('/checkout/payment-config', { params })).data;
  },
  async submit(purchaseId: number, input: { token: string; paymentMethodId: string; issuerId?: string; installments: number }) {
    return (await apiClient.post<PaymentState>(`/purchases/${purchaseId}/payments/submit`, input)).data;
  },
  async status(purchaseId: number) {
    return (await apiClient.get<PaymentState>(`/purchases/${purchaseId}/payments/status`)).data;
  },
};
export default checkoutService;
