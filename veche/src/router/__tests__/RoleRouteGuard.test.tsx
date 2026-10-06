import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RoleRouteGuard } from "../RoleRouteGuard";

const auth = vi.hoisted(() => ({ role: "TenantUser" }));

vi.mock("@/store/hooks", () => ({
  useAppSelector: (selector: (state: unknown) => unknown) =>
    selector({ auth: { user: { role: auth.role } } }),
}));

function renderSettingsRoute(role: string) {
  auth.role = role;

  return render(
    <MemoryRouter initialEntries={["/settings"]}>
      <Routes>
        <Route
          path="/settings"
          element={
            <RoleRouteGuard allowedRoles={["SysAdmin", "TenantOwner"]}>
              <div>Settings</div>
            </RoleRouteGuard>
          }
        />
        <Route path="/" element={<div>Home</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("RoleRouteGuard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it.each(["SysAdmin", "TenantOwner"])("allows %s", (role) => {
    renderSettingsRoute(role);
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it.each(["TenantUser", "Customer"])("redirects %s", (role) => {
    renderSettingsRoute(role);
    expect(screen.getByText("Home")).toBeInTheDocument();
    expect(screen.queryByText("Settings")).not.toBeInTheDocument();
  });
});