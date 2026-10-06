import { HelmetProvider } from "react-helmet-async";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import UserForm from "../UserForm";

const auth = vi.hoisted(() => ({
  user: { role: "TenantOwner", tenantId: 1 as number | null },
  getUserById: vi.fn(),
  dispatch: vi.fn(),
  translate: (_key: string, fallback?: string) => fallback ?? _key,
}));

vi.mock("@/store/hooks", () => ({
  useAppDispatch: () => auth.dispatch,
  useAppSelector: (selector: (state: unknown) => unknown) =>
    selector({
      auth: { user: auth.user },
      user: { loading: false, error: null },
      tenant: { tenantsList: [] },
    }),
}));

vi.mock("@/services/userService", () => ({
  userService: { getUserById: auth.getUserById },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: auth.translate,
  }),
}));

function renderEditRoute() {
  return render(
    <HelmetProvider>
      <MemoryRouter initialEntries={["/users/3/edit"]}>
        <Routes>
          <Route path="/users/:id/edit" element={<UserForm />} />
        </Routes>
      </MemoryRouter>
    </HelmetProvider>,
  );
}

describe("UserForm direct edit lookup", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    auth.user = { role: "TenantOwner", tenantId: 1 };
  });

  it("does not render an editable form while the target user is loading", async () => {
    auth.getUserById.mockReturnValue(new Promise(() => {}));

    renderEditRoute();

    expect(await screen.findByRole("status")).toHaveTextContent("Carregando...");
    expect(screen.queryByRole("button", { name: "Salvar alterações" })).not.toBeInTheDocument();
  });

  it("denies a cross-tenant user when the API returns 404", async () => {
    auth.getUserById.mockRejectedValue({
      isAxiosError: true,
      response: { status: 404, data: {} },
    });

    renderEditRoute();

    await waitFor(() => expect(auth.getUserById).toHaveBeenCalledWith(3));
    expect(auth.getUserById).toHaveBeenCalledTimes(1);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Você não tem acesso a este usuário.",
    );
    expect(screen.queryByRole("button", { name: "Salvar alterações" })).not.toBeInTheDocument();
  });

  it("denies a cross-tenant user even if the API returns its record", async () => {
    auth.getUserById.mockResolvedValue({
      id: 3,
      email: "owner-t2@example.test",
      enabled: true,
      firstLogin: false,
      role: "TenantOwner",
      tenantId: 2,
    });

    renderEditRoute();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Você não tem acesso a este usuário.",
    );
    expect(screen.queryByRole("button", { name: "Salvar alterações" })).not.toBeInTheDocument();
  });

  it("loads the user form for SysAdmin", async () => {
    auth.user = { role: "SysAdmin", tenantId: null };
    auth.getUserById.mockResolvedValue({
      id: 3,
      email: "owner-t2@example.test",
      enabled: true,
      firstLogin: false,
      role: "TenantOwner",
      tenantId: 2,
    });

    renderEditRoute();

    await waitFor(() =>
      expect(screen.getByLabelText("E-mail")).toHaveValue("owner-t2@example.test"),
    );
    expect(screen.getByRole("button", { name: "Salvar alterações" })).toBeInTheDocument();
  });
});