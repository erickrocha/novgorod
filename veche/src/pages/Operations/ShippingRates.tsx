import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Pencil, Plus, RefreshCw, Truck } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { shippingRateService } from "@/services/shippingRateService";
import type { PageQueryParams, ShippingRate, ShippingRateInput } from "@/services/types";
import { ROLES } from "@/utils/enums";

const BRAZIL_UFS = [
  "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA",
  "MG", "MS", "MT", "PA", "PB", "PE", "PI", "PR", "RJ", "RN",
  "RO", "RR", "RS", "SC", "SE", "SP", "TO",
];

export default function ShippingRates() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [rates, setRates] = useState<ShippingRate[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Modal State
  const [modalOpen, setModalOpen] = useState(false);
  const [editingRate, setEditingRate] = useState<ShippingRate | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    uf: string;
    price: string;
    tenantId: number | null;
  }>({ uf: "SP", price: "0.00", tenantId: user?.tenantId || null });

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await shippingRateService.paged(params);
      setRates(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load shipping rates";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadData();
      }
    });
    return () => {
      active = false;
    };
  }, [loadData]);

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
    setEditingRate(null);
    setFormData({
      uf: "SP",
      price: "0.00",
      tenantId: user?.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (rate: ShippingRate) => {
    setEditingRate(rate);
    setFormData({
      uf: rate.uf,
      price: (rate.priceCents / 100).toFixed(2),
      tenantId: rate.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setSaving(true);
    try {
      const priceCents = Math.round(parseFloat(formData.price || "0") * 100);
      const payload: ShippingRateInput = {
        uf: formData.uf.toUpperCase().trim(),
        priceCents,
        tenantId: formData.tenantId,
      };

      if (editingRate?.id) {
        await shippingRateService.update(editingRate.id, payload);
      } else {
        await shippingRateService.create(payload);
      }
      setModalOpen(false);
      loadData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save shipping rate";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<ShippingRate, unknown>[] = [
    {
      header: t("operations.shippingRates.ufCol", "State (UF)"),
      accessorKey: "uf",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-semibold text-gray-900 dark:text-white">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: t("operations.shippingRates.priceCol", "Shipping Rate"),
      accessorKey: "priceCents",
      enableSorting: true,
      cell: ({ getValue }) => {
        const cents = getValue<number>() || 0;
        return (
          <span className="font-medium text-brand-600 dark:text-brand-400">
            {(cents / 100).toLocaleString("pt-BR", { style: "currency", currency: "BRL" })}
          </span>
        );
      },
    },
    ...(isSysAdmin
      ? [
          {
            header: t("common.tenant", "Tenant"),
            id: "tenant",
            cell: ({ row }: { row: { original: ShippingRate } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      header: t("operations.shippingRates.updatedAtCol", "Last Updated"),
      accessorKey: "updatedAt",
      cell: ({ getValue }) => {
        const dateStr = getValue<string>();
        return dateStr ? new Date(dateStr).toLocaleDateString("pt-BR") : "—";
      },
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end">
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
        title={`${t("operations.shippingRates.title", "Shipping Rates")} | Veche`}
        description={t("operations.shippingRates.desc", "Manage regional shipping rates by state/UF")}
      />
      <PageBreadcrumb pageTitle={t("operations.shippingRates.title", "Shipping Rates")} />
      <ComponentCard>
        <DataGrid
          data={rates}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadData}
              >
                {t("common.refresh", "Refresh")}
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateModal}
              >
                {t("operations.shippingRates.addRate", "Add Rate")}
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && rates.length === 0}
          error={error}
          emptyMessage={t("operations.shippingRates.emptyMessage", "No shipping rates found.")}
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

      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-md p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <Truck size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingRate ? t("operations.shippingRates.editRate", "Edit Shipping Rate") : t("operations.shippingRates.newRate", "Add Shipping Rate")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t("operations.shippingRates.modalDesc", "Configure flat shipping rate for the destination state.")}
            </p>
          </div>
        </div>

        {formError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="space-y-4">
          <div>
            <Label htmlFor="uf">{t("operations.shippingRates.ufLabel", "Destination State (UF)")}</Label>
            <select
              id="uf"
              value={formData.uf}
              onChange={(e) => setFormData({ ...formData, uf: e.target.value })}
              className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              required
            >
              {BRAZIL_UFS.map((uf) => (
                <option key={uf} value={uf}>
                  {uf}
                </option>
              ))}
            </select>
          </div>

          <div>
            <Label htmlFor="price">{t("operations.shippingRates.priceLabel", "Rate Price (R$)")}</Label>
            <Input
              id="price"
              type="number"
              step={0.01}
              min="0"
              placeholder="25.00"
              value={formData.price}
              onChange={(e) => setFormData({ ...formData, price: e.target.value })}
              required
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
              {saving ? t("common.saving", "Saving…") : editingRate ? t("operations.shippingRates.saveRate", "Update Rate") : t("operations.shippingRates.createRate", "Save Rate")}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
}
