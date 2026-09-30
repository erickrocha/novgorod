import { describe, it, expect } from "vitest";
import paymentSettingsReducer, {
  clearPaymentSettingsStatus,
  resetPaymentSettings,
  fetchPaymentSettings,
  updatePaymentSettings,
  type PaymentSettingsState,
} from "../paymentSettingsSlice";
import type { PaymentSettingsResponse } from "@/services/types";

describe("paymentSettingsSlice", () => {
  const initialState: PaymentSettingsState = {
    settings: null,
    loading: false,
    saving: false,
    error: null,
    saveSuccess: false,
  };

  const mockResponse: PaymentSettingsResponse = {
    provider: "pagseguro",
    version: 1,
    credentialsConfigured: true,
    publicKey: "PUB_KEY_PS_123",
    environment: "sandbox",
  };

  it("returns initial state", () => {
    const state = paymentSettingsReducer(undefined, { type: "" });
    expect(state).toEqual(initialState);
  });

  it("handles clearPaymentSettingsStatus", () => {
    const modifiedState: PaymentSettingsState = {
      ...initialState,
      error: "Some payment error",
      saveSuccess: true,
    };
    const state = paymentSettingsReducer(
      modifiedState,
      clearPaymentSettingsStatus()
    );
    expect(state.error).toBeNull();
    expect(state.saveSuccess).toBe(false);
  });

  it("handles resetPaymentSettings", () => {
    const modifiedState: PaymentSettingsState = {
      settings: mockResponse,
      loading: true,
      saving: true,
      error: "Error",
      saveSuccess: true,
    };
    const state = paymentSettingsReducer(
      modifiedState,
      resetPaymentSettings()
    );
    expect(state).toEqual(initialState);
  });

  it("handles fetchPaymentSettings.pending", () => {
    const state = paymentSettingsReducer(
      initialState,
      fetchPaymentSettings.pending("", 10)
    );
    expect(state.loading).toBe(true);
    expect(state.error).toBeNull();
  });

  it("handles fetchPaymentSettings.fulfilled", () => {
    const state = paymentSettingsReducer(
      { ...initialState, loading: true },
      fetchPaymentSettings.fulfilled(mockResponse, "", 10)
    );
    expect(state.loading).toBe(false);
    expect(state.settings).toEqual(mockResponse);
  });

  it("handles fetchPaymentSettings.rejected", () => {
    const state = paymentSettingsReducer(
      { ...initialState, loading: true },
      fetchPaymentSettings.rejected(
        new Error("Network Error"),
        "",
        10,
        "Tenant not found"
      )
    );
    expect(state.loading).toBe(false);
    expect(state.error).toBe("Tenant not found");
  });

  it("handles updatePaymentSettings.pending", () => {
    const state = paymentSettingsReducer(
      initialState,
      updatePaymentSettings.pending("", {
        tenantId: 10,
        data: { provider: "pagseguro" },
      })
    );
    expect(state.saving).toBe(true);
    expect(state.error).toBeNull();
    expect(state.saveSuccess).toBe(false);
  });

  it("handles updatePaymentSettings.fulfilled", () => {
    const state = paymentSettingsReducer(
      { ...initialState, saving: true },
      updatePaymentSettings.fulfilled(undefined, "", {
        tenantId: 10,
        data: { provider: "pagseguro" },
      })
    );
    expect(state.saving).toBe(false);
    expect(state.saveSuccess).toBe(true);
  });

  it("handles updatePaymentSettings.rejected", () => {
    const state = paymentSettingsReducer(
      { ...initialState, saving: true },
      updatePaymentSettings.rejected(
        new Error("Save Error"),
        "",
        {
          tenantId: 10,
          data: { provider: "pagseguro" },
        },
        "Validation failed"
      )
    );
    expect(state.saving).toBe(false);
    expect(state.error).toBe("Validation failed");
  });
});
