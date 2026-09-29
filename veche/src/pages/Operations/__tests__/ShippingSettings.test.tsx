import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { Provider } from "react-redux";
import { configureStore } from "@reduxjs/toolkit";
import { HelmetProvider } from "react-helmet-async";
import { ShippingSettings } from "../ShippingSettings";
import authReducer from "@/store/authSlice";
import tenantReducer from "@/store/tenantSlice";
import shippingSettingsReducer from "@/store/shippingSettingsSlice";
import { ROLES } from "@/utils/enums";
import type { ShippingSettingsResponse } from "@/services/types";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback || key,
  }),
}));

const mockSettings: ShippingSettingsResponse = {
  configuration: {
    mode: "correios",
    originCep: "01001-000",
    services: [
      { code: "03220", name: "SEDEX" },
      { code: "03298", name: "PAC" },
    ],
    packaging: {
      weightG: 150,
      lengthMm: 15,
      widthMm: 12,
      heightMm: 8,
    },
  },
  version: 1,
  credentialsConfigured: true,
};

function createTestStore(overrides?: {
  role?: string;
  tenantId?: number | null;
  settings?: ShippingSettingsResponse | null;
}) {
  return configureStore({
    reducer: {
      auth: authReducer,
      tenant: tenantReducer,
      shippingSettings: shippingSettingsReducer,
    },
    preloadedState: {
      auth: {
        token: "fake-jwt",
        refreshToken: null,
        user: {
          accessToken: "fake-jwt",
          email: "owner@example.com",
          uuid: "u-1",
          name: "Owner",
          userId: 1,
          role: (overrides?.role as any) || ROLES.TENANT_OWNER,
          tenantId: overrides?.tenantId !== undefined ? overrides.tenantId : 10,
          firstLogin: false,
        },
        person: null,
        addresses: [],
        isAuthenticated: true,
        isInitializing: false,
        isSysAdmin: overrides?.role === ROLES.SYS_ADMIN,
        loading: false,
        error: null,
      },
      tenant: {
        tenantsList: [
          { id: 10, businessName: "Wine Store" },
          { id: 20, businessName: "Beer Store" },
        ],
        activeTenant: null,
        loading: false,
        error: null,
        total: 2,
        page: 1,
        pageSize: 10,
      },
      shippingSettings: {
        settings: overrides?.settings !== undefined ? overrides.settings : mockSettings,
        loading: false,
        saving: false,
        error: null,
        saveSuccess: false,
      },
    },
  });
}

function renderComponent(store = createTestStore()) {
  return render(
    <HelmetProvider>
      <Provider store={store}>
        <MemoryRouter>
          <ShippingSettings />
        </MemoryRouter>
      </Provider>
    </HelmetProvider>
  );
}

describe("ShippingSettings component", () => {
  it("renders page title and breadcrumbs", () => {
    renderComponent();
    expect(screen.getByRole("heading", { level: 2, name: "Shipping Settings" })).toBeInTheDocument();
  });

  it("renders shipping modes: Fixed Rates and Correios", () => {
    renderComponent();
    expect(
      screen.getByText("Fixed Rates by State (UF)")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Correios (Carrier API)")
    ).toBeInTheDocument();
  });

  it("displays credentials configured badge when credentialsConfigured is true", () => {
    renderComponent();
    expect(
      screen.getByText("Credentials configured on server")
    ).toBeInTheDocument();
  });

  it("renders packaging allowances fields", () => {
    renderComponent();
    expect(screen.getByText("Packaging Allowances")).toBeInTheDocument();
    expect(screen.getByText("Packaging Weight (g)")).toBeInTheDocument();
    expect(screen.getByText("Length Allowance (mm)")).toBeInTheDocument();
    expect(screen.getByText("Width Allowance (mm)")).toBeInTheDocument();
    expect(screen.getByText("Height Allowance (mm)")).toBeInTheDocument();
  });

  it("renders Save Settings and Cancel buttons", () => {
    renderComponent();
    expect(
      screen.getByRole("button", { name: /Save Settings/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Cancel/i })
    ).toBeInTheDocument();
  });

  it("renders seller selector when user is SysAdmin", () => {
    const store = createTestStore({ role: ROLES.SYS_ADMIN, tenantId: null });
    renderComponent(store);
    const selectors = screen.getAllByText("Select Seller");
    expect(selectors.length).toBeGreaterThanOrEqual(1);
  });
});
