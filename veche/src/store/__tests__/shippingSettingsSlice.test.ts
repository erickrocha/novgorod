import { describe, it, expect } from "vitest";
import shippingSettingsReducer, {
  clearShippingSettingsStatus,
  resetShippingSettings,
  fetchShippingSettings,
  updateShippingSettings,
  type ShippingSettingsState,
} from "../shippingSettingsSlice";
import type { ShippingSettingsResponse } from "@/services/types";

describe("shippingSettingsSlice", () => {
  const initialState: ShippingSettingsState = {
    settings: null,
    loading: false,
    saving: false,
    error: null,
    saveSuccess: false,
  };

  const mockResponse: ShippingSettingsResponse = {
    configuration: {
      mode: "correios",
      originCep: "01001000",
      services: [
        { code: "03220", name: "SEDEX" },
        { code: "03298", name: "PAC" },
      ],
      packaging: {
        weightG: 100,
        lengthMm: 10,
        widthMm: 10,
        heightMm: 10,
      },
    },
    version: 2,
    credentialsConfigured: true,
  };

  it("returns initial state", () => {
    const state = shippingSettingsReducer(undefined, { type: "" });
    expect(state).toEqual(initialState);
  });

  it("handles clearShippingSettingsStatus", () => {
    const modifiedState: ShippingSettingsState = {
      ...initialState,
      error: "Some error",
      saveSuccess: true,
    };
    const state = shippingSettingsReducer(
      modifiedState,
      clearShippingSettingsStatus()
    );
    expect(state.error).toBeNull();
    expect(state.saveSuccess).toBe(false);
  });

  it("handles resetShippingSettings", () => {
    const modifiedState: ShippingSettingsState = {
      settings: mockResponse,
      loading: true,
      saving: true,
      error: "Error",
      saveSuccess: true,
    };
    const state = shippingSettingsReducer(
      modifiedState,
      resetShippingSettings()
    );
    expect(state).toEqual(initialState);
  });

  it("handles fetchShippingSettings.pending", () => {
    const state = shippingSettingsReducer(
      initialState,
      fetchShippingSettings.pending("", 1)
    );
    expect(state.loading).toBe(true);
    expect(state.error).toBeNull();
  });

  it("handles fetchShippingSettings.fulfilled", () => {
    const state = shippingSettingsReducer(
      { ...initialState, loading: true },
      fetchShippingSettings.fulfilled(mockResponse, "", 1)
    );
    expect(state.loading).toBe(false);
    expect(state.settings).toEqual(mockResponse);
  });

  it("handles fetchShippingSettings.rejected", () => {
    const state = shippingSettingsReducer(
      { ...initialState, loading: true },
      fetchShippingSettings.rejected(
        new Error("Network Error"),
        "",
        1,
        "Tenant not found"
      )
    );
    expect(state.loading).toBe(false);
    expect(state.error).toBe("Tenant not found");
  });

  it("handles updateShippingSettings.pending", () => {
    const state = shippingSettingsReducer(
      initialState,
      updateShippingSettings.pending("", {
        tenantId: 1,
        data: { configuration: mockResponse.configuration },
      })
    );
    expect(state.saving).toBe(true);
    expect(state.error).toBeNull();
    expect(state.saveSuccess).toBe(false);
  });

  it("handles updateShippingSettings.fulfilled", () => {
    const state = shippingSettingsReducer(
      { ...initialState, saving: true },
      updateShippingSettings.fulfilled(undefined, "", {
        tenantId: 1,
        data: { configuration: mockResponse.configuration },
      })
    );
    expect(state.saving).toBe(false);
    expect(state.saveSuccess).toBe(true);
  });

  it("handles updateShippingSettings.rejected", () => {
    const state = shippingSettingsReducer(
      { ...initialState, saving: true },
      updateShippingSettings.rejected(
        new Error("Save Error"),
        "",
        {
          tenantId: 1,
          data: { configuration: mockResponse.configuration },
        },
        "Validation failed"
      )
    );
    expect(state.saving).toBe(false);
    expect(state.error).toBe("Validation failed");
  });
});
