import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { History, Pencil, Plus, RefreshCw, Ticket } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Switch from "@/components/form/switch/Switch";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { marketingService } from "@/services/marketingService";
import type {
  Coupon,
  CouponInput,
  CouponRedemption,
  PageQueryParams,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

export default function CouponsPage() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [coupons, setCoupons] = useState<Coupon[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Coupon Modal
  const [modalOpen, setModalOpen] = useState(false);
  const [editingCoupon, setEditingCoupon] = useState<Coupon | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    code: string;
    campaignId: string;
    couponType: string;
    value: string;
    minOrderAmount: string;
    maxUses: string;
    maxUsesPerCustomer: string;
    startsAt: string;
    expiresAt: string;
    active: boolean;
    tenantId: number | null;
  }>({
    code: "",
    campaignId: "",
    couponType: "PERCENTAGE",
    value: "10",
    minOrderAmount: "",
    maxUses: "100",
    maxUsesPerCustomer: "1",
    startsAt: "",
    expiresAt: "",
    active: true,
    tenantId: user?.tenantId || null,
  });

  // Redemptions Modal
  const [redemptionsModalOpen, setRedemptionsModalOpen] = useState(false);
  const [selectedCoupon, setSelectedCoupon] = useState<Coupon | null>(null);
  const [redemptions, setRedemptions] = useState<CouponRedemption[]>([]);
  const [loadingRedemptions, setLoadingRedemptions] = useState(false);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "desc";

  const loadCoupons = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await marketingService.couponsPaged(params);
      setCoupons(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load coupons";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadCoupons();
      }
    });
    return () => {
      active = false;
    };
  }, [loadCoupons]);

  useEffect(() => {
    if (isSysAdmin && tenants.length === 0) {
      dispatch(fetchTenants());
    }
  }, [dispatch, isSysAdmin, tenants.length]);

  const onPaginationChange = (next: PaginationState) => {
    const params = new URLSearchParams(searchParams);
    params.set("page", String(next.pageIndex + 1));
    params.set("pageSize", String(next.pageSize));
    setSearchParams(params);
  };

  const onSortingChange = (next: SortingState) => {
    const params = new URLSearchParams(searchParams);
    if (next.length > 0) {
      params.set("sortBy", next[0].id);
      params.set("sortDir", next[0].desc ? "desc" : "asc");
    } else {
      params.delete("sortBy");
      params.delete("sortDir");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const onGlobalFilterChange = (val: string) => {
    const params = new URLSearchParams(searchParams);
    if (val) {
      params.set("q", val);
    } else {
      params.delete("q");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const openCreateModal = () => {
    setEditingCoupon(null);
    setFormData({
      code: "",
      campaignId: "",
      couponType: "PERCENTAGE",
      value: "10",
      minOrderAmount: "",
      maxUses: "100",
      maxUsesPerCustomer: "1",
      startsAt: "",
      expiresAt: "",
      active: true,
      tenantId: user?.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (c: Coupon) => {
    setEditingCoupon(c);
    setFormData({
      code: c.code,
      campaignId: c.campaignId ? String(c.campaignId) : "",
      couponType: c.couponType,
      value: String(c.value),
      minOrderAmount: c.minOrderCents ? (c.minOrderCents / 100).toFixed(2) : "",
      maxUses: c.maxUses != null ? String(c.maxUses) : "",
      maxUsesPerCustomer:
        c.maxUsesPerCustomer != null ? String(c.maxUsesPerCustomer) : "1",
      startsAt: c.startsAt ? c.startsAt.slice(0, 16) : "",
      expiresAt: c.expiresAt ? c.expiresAt.slice(0, 16) : "",
      active: c.active,
      tenantId: c.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setSaving(true);
    try {
      const minOrderCents = formData.minOrderAmount
        ? Math.round(parseFloat(formData.minOrderAmount) * 100)
        : null;

      const payload: CouponInput = {
        code: formData.code.trim().toUpperCase(),
        campaignId: formData.campaignId ? Number(formData.campaignId) : null,
        couponType: formData.couponType,
        value: Number(formData.value),
        minOrderCents,
        maxUses: formData.maxUses ? Number(formData.maxUses) : null,
        maxUsesPerCustomer: formData.maxUsesPerCustomer
          ? Number(formData.maxUsesPerCustomer)
          : null,
        startsAt: formData.startsAt ? formData.startsAt + ":00" : null,
        expiresAt: formData.expiresAt ? formData.expiresAt + ":00" : null,
        active: formData.active,
        tenantId: formData.tenantId,
      };

      if (editingCoupon?.id) {
        await marketingService.updateCoupon(editingCoupon.id, payload);
      } else {
        await marketingService.createCoupon(payload);
      }
      setModalOpen(false);
      loadCoupons();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save coupon";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const openRedemptionsModal = async (c: Coupon) => {
    setSelectedCoupon(c);
    setRedemptionsModalOpen(true);
    setLoadingRedemptions(true);
    try {
      const res = await marketingService.redemptionsPaged({ couponId: c.id, pageSize: 50 });
      setRedemptions(res.items);
    } catch {
      setRedemptions([]);
    } finally {
      setLoadingRedemptions(false);
    }
  };

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<Coupon, unknown>[] = [
    {
      header: t("marketing.coupons.codeCol", "Coupon Code"),
      accessorKey: "code",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono font-bold tracking-wider text-brand-600 dark:text-brand-400 bg-brand-50/70 dark:bg-brand-500/10 px-2.5 py-1 rounded-md text-xs">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: t("marketing.coupons.discountCol", "Discount"),
      id: "discount",
      accessorFn: (row) =>
        row.couponType === "PERCENTAGE"
          ? `${row.value}% OFF`
          : `R$ ${(row.value / 100).toFixed(2)} OFF`,
      cell: ({ getValue }) => (
        <span className="font-semibold text-gray-900 dark:text-white">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: t("marketing.coupons.minOrderCol", "Min. Order"),
      accessorKey: "minOrderCents",
      cell: ({ getValue }) => {
        const c = getValue<number>();
        return c != null ? `R$ ${(c / 100).toFixed(2)}` : t("marketing.coupons.none", "None");
      },
    },
    {
      header: t("marketing.coupons.maxUsesCol", "Max Uses"),
      accessorKey: "maxUses",
      cell: ({ getValue }) => getValue<number>() ?? t("marketing.coupons.unlimited", "Unlimited"),
    },
    {
      header: t("common.status", "Status"),
      accessorKey: "active",
      cell: ({ getValue }) => (
        <Badge size="sm" color={getValue<boolean>() ? "success" : "light"}>
          {getValue<boolean>() ? t("common.active", "Active") : t("common.inactive", "Inactive")}
        </Badge>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            header: t("common.tenant", "Tenant"),
            id: "tenant",
            cell: ({ row }: { row: { original: Coupon } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      id: "actions",
      header: "",
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end gap-1">
          <button
            type="button"
            title={t("marketing.coupons.redemptionsTitle", "Redemptions History")}
            aria-label={t("marketing.coupons.redemptions", "Redemptions")}
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openRedemptionsModal(row.original)}
          >
            <History size={15} />
          </button>
          <button
            type="button"
            title={t("common.edit", "Edit")}
            aria-label={t("common.edit", "Edit")}
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openEditModal(row.original)}
          >
            <Pencil size={15} />
          </button>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title={`${t("marketing.coupons.title", "Coupons")} | Veche`}
        description={t("marketing.coupons.desc", "Create discount vouchers and track coupon usage")}
      />
      <PageBreadcrumb pageTitle={t("marketing.coupons.title", "Coupons")} />
      <ComponentCard>
        <DataGrid
          data={coupons}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadCoupons}
              >
                {t("common.refresh", "Refresh")}
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateModal}
              >
                {t("marketing.coupons.addCoupon", "Add Coupon")}
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && coupons.length === 0}
          error={error}
          emptyMessage={t("marketing.coupons.emptyMessage", "No coupons found.")}
          manualPagination
          manualSorting
          manualFiltering
          totalRows={total}
          pagination={{ pageIndex: Math.max(0, page - 1), pageSize }}
          onPaginationChange={onPaginationChange}
          sorting={[{ id: sortBy, desc: sortDir === "desc" }]}
          onSortingChange={onSortingChange}
          globalFilter={q}
          onGlobalFilterChange={onGlobalFilterChange}
        />
      </ComponentCard>

      {/* Coupon Modal */}
      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-lg p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <Ticket size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingCoupon ? t("marketing.coupons.editCoupon", "Edit Coupon") : t("marketing.coupons.newCoupon", "Add Coupon")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t("marketing.coupons.modalDesc", "Set discount code, percentage or fixed value, and redemption limits.")}
            </p>
          </div>
        </div>

        {formError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="code">{t("marketing.coupons.couponCode", "Coupon Code")}</Label>
              <Input
                id="code"
                placeholder="SUMMER20"
                className="uppercase tracking-wider font-mono font-semibold"
                value={formData.code}
                onChange={(e) =>
                  setFormData({ ...formData, code: e.target.value.toUpperCase() })
                }
                required
              />
            </div>
            <div>
              <Label htmlFor="couponType">{t("marketing.coupons.discountType", "Discount Type")}</Label>
              <select
                id="couponType"
                value={formData.couponType}
                onChange={(e) =>
                  setFormData({ ...formData, couponType: e.target.value })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="PERCENTAGE">{t("marketing.coupons.types.percentage", "Percentage (%)")}</option>
                <option value="FIXED">{t("marketing.coupons.types.fixed", "Fixed Amount (Cents)")}</option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="val">
                {formData.couponType === "PERCENTAGE" ? t("marketing.coupons.valuePercent", "Value (%)") : t("marketing.coupons.valueCents", "Value (Cents)")}
              </Label>
              <Input
                id="val"
                type="number"
                min="1"
                placeholder={formData.couponType === "PERCENTAGE" ? "10" : "1500"}
                value={formData.value}
                onChange={(e) => setFormData({ ...formData, value: e.target.value })}
                required
              />
            </div>
            <div>
              <Label htmlFor="minOrder">{t("marketing.coupons.minOrderAmount", "Min. Order Amount (R$)")}</Label>
              <Input
                id="minOrder"
                type="number"
                step={0.01}
                min="0"
                placeholder="100.00"
                value={formData.minOrderAmount}
                onChange={(e) =>
                  setFormData({ ...formData, minOrderAmount: e.target.value })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="maxUses">{t("marketing.coupons.maxUses", "Total Usage Limit")}</Label>
              <Input
                id="maxUses"
                type="number"
                min="1"
                placeholder="100"
                value={formData.maxUses}
                onChange={(e) =>
                  setFormData({ ...formData, maxUses: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="maxPerCustomer">{t("marketing.coupons.maxUsesPerCustomer", "Limit Per Customer")}</Label>
              <Input
                id="maxPerCustomer"
                type="number"
                min="1"
                placeholder="1"
                value={formData.maxUsesPerCustomer}
                onChange={(e) =>
                  setFormData({ ...formData, maxUsesPerCustomer: e.target.value })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="startsAt">{t("marketing.coupons.startsAt", "Valid From")}</Label>
              <Input
                id="startsAt"
                type="datetime-local"
                value={formData.startsAt}
                onChange={(e) =>
                  setFormData({ ...formData, startsAt: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="expiresAt">{t("marketing.coupons.expiresAt", "Expires At")}</Label>
              <Input
                id="expiresAt"
                type="datetime-local"
                value={formData.expiresAt}
                onChange={(e) =>
                  setFormData({ ...formData, expiresAt: e.target.value })
                }
              />
            </div>
          </div>

          <div className="pt-2">
            <Switch
              label={t("marketing.coupons.activeCoupon", "Active Coupon")}
              checked={formData.active}
              onChange={(checked) => setFormData({ ...formData, active: checked })}
            />
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">{t("common.tenant", "Tenant")}</Label>
              <select
                id="tenantId"
                value={formData.tenantId || ""}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    tenantId: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="">{t("customers.defaultNone", "Default / None")}</option>
                {tenants.map((tTenant) => (
                  <option key={tTenant.id} value={tTenant.id ?? ""}>
                    {tTenant.businessName || tTenant.companyName}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="mt-6 flex justify-end gap-3 pt-3 border-t border-gray-100 dark:border-gray-800">
            <Button
              type="button"
              variant="outline"
              onClick={() => setModalOpen(false)}
            >
              {t("common.cancel", "Cancel")}
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? t("common.saving", "Saving…") : t("marketing.coupons.saveCoupon", "Save Coupon")}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Redemptions Modal */}
      <Modal
        isOpen={redemptionsModalOpen}
        onClose={() => setRedemptionsModalOpen(false)}
        className="max-w-md p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <History size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {t("marketing.coupons.redemptionsTitle", "Redemptions History")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t("marketing.coupons.redemptionsDesc", "Coupon {code} usage by customers.", { code: selectedCoupon?.code })}
            </p>
          </div>
        </div>

        <div className="space-y-2 max-h-64 overflow-y-auto">
          {loadingRedemptions ? (
            <p className="text-sm text-center text-gray-500 py-6">
              {t("marketing.coupons.loadingRedemptions", "Loading redemptions…")}
            </p>
          ) : redemptions.length === 0 ? (
            <p className="text-sm text-center text-gray-500 py-6">
              {t("marketing.coupons.noRedemptions", "No redemptions recorded for this coupon yet.")}
            </p>
          ) : (
            redemptions.map((r) => (
              <div
                key={r.id}
                className="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/30 text-xs"
              >
                <div>
                  <span className="font-semibold text-gray-900 dark:text-white block">
                    {t("marketing.coupons.orderNum", "Order #{id}", { id: r.orderId })}
                  </span>
                  <span className="text-gray-500">
                    {t("marketing.coupons.customerNum", "Customer #{id}", { id: r.customerId })}
                  </span>
                </div>
                <span className="text-gray-400 text-2xs">
                  {r.createdAt ? new Date(r.createdAt).toLocaleString("pt-BR") : "—"}
                </span>
              </div>
            ))
          )}
        </div>
      </Modal>
    </>
  );
}
