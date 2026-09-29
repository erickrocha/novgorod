import { api } from "./api";
import type { ShippingSettingsResponse, ShippingSettingsUpdate } from "./types";

export const shippingSettingsService = {
  async getSettings(tenantId: number): Promise<ShippingSettingsResponse> {
    const response = await api.get<ShippingSettingsResponse>(
      `/tenants/${tenantId}/shipping-settings`
    );
    return response.data;
  },

  async updateSettings(
    tenantId: number,
    data: ShippingSettingsUpdate
  ): Promise<void> {
    await api.put(`/tenants/${tenantId}/shipping-settings`, data);
  },
};
