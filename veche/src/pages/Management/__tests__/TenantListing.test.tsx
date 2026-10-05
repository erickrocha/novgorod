import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import TenantListing from "../TenantListing";
import {
  tenantService,
  type TenantListing as Listing,
} from "@/services/tenantService";

const auth = vi.hoisted(() => ({ user: { role: "TenantOwner", tenantId: 7 } }));
vi.mock("@/store/hooks", () => ({
  useAppSelector: () => ({ user: auth.user }),
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@/icons", () => ({ CheckLineIcon: () => null }));
vi.mock("@/services/tenantService", () => ({
  tenantService: { getListing: vi.fn(), setListing: vi.fn() },
}));

describe("tenant listing control", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    auth.user = { role: "TenantOwner", tenantId: 7 };
    vi.mocked(tenantService.getListing).mockResolvedValue({ listed: false });
    vi.mocked(tenantService.setListing).mockImplementation(
      async (_id, listed) => ({ listed }),
    );
  });

  it.each([true, false])("loads persisted flag %s", async (listed) => {
    vi.mocked(tenantService.getListing).mockResolvedValue({ listed });
    render(<TenantListing tenantId={7} />);
    const checkbox = screen.getByRole("checkbox");
    expect(checkbox).toBeDisabled();
    await waitFor(() => expect(checkbox).toBeEnabled());
    expect(checkbox).toHaveProperty("checked", listed);
    expect(
      screen.getByRole("button", { name: "tenants.listing.save" }),
    ).toBeDisabled();
  });

  it("saves independently without submitting the parent form", async () => {
    const submit = vi.fn((event) => event.preventDefault());
    render(
      <form onSubmit={submit}>
        <TenantListing tenantId={7} />
      </form>,
    );
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeEnabled());
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(
      screen.getByRole("button", { name: "tenants.listing.save" }),
    );
    await screen.findByText("tenants.listing.saved");
    expect(tenantService.setListing).toHaveBeenCalledWith(7, true);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(
      screen.getByRole("button", { name: "tenants.listing.save" }),
    );
    await screen.findByText("tenants.listing.saved");
    expect(tenantService.setListing).toHaveBeenLastCalledWith(7, false);
  });

  it.each(["TenantUser", "Customer", "TenantOwner"])(
    "denies unauthorized %s",
    (role) => {
      auth.user = { role, tenantId: 8 };
      const { container } = render(<TenantListing tenantId={7} />);
      expect(container).toBeEmptyDOMElement();
      expect(tenantService.getListing).not.toHaveBeenCalled();
    },
  );

  it("allows SysAdmin to edit another tenant", async () => {
    auth.user = { role: "SysAdmin", tenantId: 8 };
    render(<TenantListing tenantId={7} />);
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeEnabled());
  });

  it("does not assume false when reading fails and supports retry", async () => {
    vi.mocked(tenantService.getListing).mockRejectedValueOnce(
      new Error("offline"),
    );
    render(<TenantListing tenantId={7} />);
    await screen.findByRole("alert");
    expect(screen.getByRole("checkbox")).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "tenants.listing.retry" }),
    );
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeEnabled());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("preserves changes after a failed save and prevents duplicate writes", async () => {
    let rejectWrite!: (reason: Error) => void;
    vi.mocked(tenantService.setListing).mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          rejectWrite = reject;
        }),
    );
    render(<TenantListing tenantId={7} />);
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeEnabled());
    fireEvent.click(screen.getByRole("checkbox"));
    const button = screen.getByRole("button", { name: "tenants.listing.save" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(tenantService.setListing).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("checkbox")).toBeDisabled();
    await act(async () => rejectWrite(new Error("offline")));
    expect(screen.getByRole("checkbox")).toBeChecked();
    expect(screen.queryByText("tenants.listing.saved")).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "tenants.listing.save" }),
    );
    await screen.findByText("tenants.listing.saved");
  });

  it("ignores a delayed response belonging to the previous tenant", async () => {
    auth.user = { role: "SysAdmin", tenantId: 7 };
    let resolvePrevious!: (value: Listing) => void;
    vi.mocked(tenantService.getListing).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolvePrevious = resolve;
        }),
    );
    const view = render(<TenantListing tenantId={7} />);
    view.rerender(<TenantListing tenantId={8} />);
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeEnabled());
    await act(async () => resolvePrevious({ listed: true }));
    expect(screen.getByRole("checkbox")).not.toBeChecked();
  });
});
