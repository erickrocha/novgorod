import { api } from "./api";
import type { PaymentSettingsResponse, PaymentSettingsUpdate } from "./types";

export const paymentSettingsService = {
  async getSettings(tenantId: number): Promise<PaymentSettingsResponse> {
    const response = await api.get<PaymentSettingsResponse>(
      `/tenants/${tenantId}/payment-settings`
    );
    return response.data;
  },

  async updateSettings(
    tenantId: number,
    data: PaymentSettingsUpdate
  ): Promise<void> {
    await api.put(`/tenants/${tenantId}/payment-settings`, data);
  },
};
