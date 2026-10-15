import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { configureStore } from "@reduxjs/toolkit";
import { HelmetProvider } from "react-helmet-async";
import { Provider } from "react-redux";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import PaymentSettings from "../PaymentSettings";
import authReducer from "@/store/authSlice";
import tenantReducer from "@/store/tenantSlice";
import paymentSettingsReducer from "@/store/paymentSettingsSlice";
import { paymentSettingsService } from "@/services/paymentSettingsService";
import { ROLES } from "@/utils/enums";
import type { PaymentSettingsResponse } from "@/services/types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

vi.mock("@/services/paymentSettingsService", () => ({
  paymentSettingsService: {
    getSettings: vi.fn(),
    updateSettings: vi.fn(),
  },
}));

const settings: PaymentSettingsResponse = {
  provider: "pagseguro",
  version: 1,
  credentialsConfigured: false,
  publicKey: null,
  environment: "sandbox",
};

function createTestStore() {
  return configureStore({
    reducer: {
      auth: authReducer,
      tenant: tenantReducer,
      paymentSettings: paymentSettingsReducer,
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
          role: ROLES.TENANT_OWNER,
          tenantId: 10,
          firstLogin: false,
        },
        person: null,
        addresses: [],
        isAuthenticated: true,
        isInitializing: false,
        isSysAdmin: false,
        loading: false,
        error: null,
      },
      tenant: {
        tenantsList: [],
        activeTenant: null,
        loading: false,
        error: null,
        total: 0,
        page: 1,
        pageSize: 10,
      },
      paymentSettings: {
        settings,
        loading: false,
        saving: false,
        error: null,
        saveSuccess: false,
      },
    },
  });
}

function renderPage() {
  const store = createTestStore();
  return render(
    <HelmetProvider>
      <Provider store={store}>
        <MemoryRouter>
          <PaymentSettings />
        </MemoryRouter>
      </Provider>
    </HelmetProvider>,
  );
}

describe("PaymentSettings page", () => {
  beforeEach(() => {
    vi.mocked(paymentSettingsService.getSettings).mockResolvedValue(settings);
    vi.mocked(paymentSettingsService.updateSettings).mockResolvedValue();
  });

  it("offers PagSeguro only and sends PagSeguro credentials on save", async () => {
    const user = userEvent.setup();
    renderPage();

    expect(await screen.findByText("PagSeguro / PagBank")).toBeInTheDocument();
    expect(screen.queryByText(/Mercado Pago/i)).not.toBeInTheDocument();
    expect(screen.getByLabelText("Token de Acesso PagSeguro")).toBeInTheDocument();
    expect(screen.getByLabelText("Chave Pública (Para criptografia no frontend)")).toBeInTheDocument();

    await user.type(screen.getByLabelText("Token de Acesso PagSeguro"), "local-token");
    await user.type(screen.getByLabelText("Chave Pública (Para criptografia no frontend)"), "local-public-key");
    await user.selectOptions(screen.getByLabelText("Ambiente"), "production");
    await user.click(screen.getByRole("button", { name: "Salvar Configurações" }));

    await waitFor(() =>
      expect(paymentSettingsService.updateSettings).toHaveBeenCalledWith(10, {
        provider: "pagseguro",
        credentials: {
          pagseguro: {
            token: "local-token",
            publicKey: "local-public-key",
            environment: "production",
          },
        },
      }),
    );
  });
});