import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api";
import { tenantService } from "../tenantService";

vi.mock("../api", () => ({ api: { get: vi.fn(), put: vi.fn() } }));

describe("tenant listing service", () => {
  beforeEach(() => vi.clearAllMocks());

  it("reads only the listing endpoint", async () => {
    vi.mocked(api.get).mockResolvedValue({ data: { listed: true } });
    await expect(tenantService.getListing(7)).resolves.toEqual({
      listed: true,
    });
    expect(api.get).toHaveBeenCalledWith("/tenant/7/listing");
  });

  it.each([true, false])("writes only listed=%s", async (listed) => {
    vi.mocked(api.put).mockResolvedValue({ data: { listed } });
    await expect(tenantService.setListing(7, listed)).resolves.toEqual({
      listed,
    });
    expect(api.put).toHaveBeenCalledWith("/tenant/7/listing", { listed });
  });

  it("propagates failures without claiming success", async () => {
    vi.mocked(api.put).mockRejectedValue(new Error("unavailable"));
    await expect(tenantService.setListing(7, true)).rejects.toThrow(
      "unavailable",
    );
  });
});
