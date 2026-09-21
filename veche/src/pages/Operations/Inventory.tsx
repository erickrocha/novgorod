import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Boxes, Pencil, Plus, RefreshCw, Trash2 } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import { Modal } from "@/components/ui/modal";
import Badge from "@/components/ui/badge/Badge";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { skuStockService } from "@/services/skuStockService";
import type { PageQueryParams, SkuStock, SkuStockInput } from "@/services/types";
import { ROLES } from "@/utils/enums";

export default function Inventory() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [items, setItems] = useState<SkuStock[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Modal State
  const [modalOpen, setModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<SkuStock | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    skuId: number;
    quantity: number;
    reserved: number;
    tenantId: number | null;
  }>({ skuId: 0, quantity: 0, reserved: 0, tenantId: user?.tenantId || null });

  // Delete State
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [itemToDelete, setItemToDelete] = useState<SkuStock | null>(null);

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
      const res = await skuStockService.paged(params);
      setItems(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load inventory stock records";
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

  const updateSearch = (patch: Record<string, string | number>) => {
    const next = new URLSearchParams(searchParams);
    Object.entries(patch).forEach(([k, v]) => {
      if (v === "" || v === null || v === undefined) {
        next.delete(k);
      } else {
        next.set(k, String(v));
      }
    });
    setSearchParams(next);
  };

  const handleOpenAdd = () => {
    setEditingItem(null);
    setFormError(null);
    setFormData({
      skuId: 0,
      quantity: 0,
      reserved: 0,
      tenantId: user?.tenantId || null,
    });
    setModalOpen(true);
  };

  const handleOpenEdit = (item: SkuStock) => {
    setEditingItem(item);
    setFormError(null);
    setFormData({
      skuId: item.skuId,
      quantity: item.quantity,
      reserved: item.reserved,
      tenantId: item.tenantId || null,
    });
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.skuId <= 0 || formData.quantity < 0 || formData.reserved < 0) {
      setFormError("SKU ID must be valid and quantities cannot be negative.");
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      const payload: SkuStockInput = {
        skuId: Number(formData.skuId),
        quantity: Number(formData.quantity),
        reserved: Number(formData.reserved),
        tenantId: formData.tenantId,
      };

      if (editingItem?.id) {
        await skuStockService.update(editingItem.id, payload);
      } else {
        await skuStockService.create(payload);
      }
      setModalOpen(false);
      loadData();
    } catch (err: unknown) {
      setFormError(err instanceof Error ? err.message : "Failed to save inventory stock");
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!itemToDelete?.id) return;
    try {
      await skuStockService.delete(itemToDelete.id);
      setDeleteConfirmOpen(false);
      setItemToDelete(null);
      loadData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to delete item");
    }
  };

  const columns: ColumnDef<SkuStock>[] = [
    {
      accessorKey: "id",
      header: t("operations.inventory.idCol", "ID"),
      enableSorting: true,
      cell: ({ row }) => <span className="font-mono text-xs">{row.original.id}</span>,
    },
    {
      accessorKey: "skuId",
      header: t("operations.inventory.skuIdCol", "SKU ID"),
      enableSorting: true,
      cell: ({ row }) => (
        <span className="font-medium text-gray-800 dark:text-white/90">
          #{row.original.skuId}
        </span>
      ),
    },
    {
      accessorKey: "quantity",
      header: t("operations.inventory.totalStockCol", "Total Stock"),
      enableSorting: true,
      cell: ({ row }) => (
        <span className="font-semibold text-gray-900 dark:text-white">
          {t("operations.inventory.units", "{count} units", { count: row.original.quantity })}
        </span>
      ),
    },
    {
      accessorKey: "reserved",
      header: t("operations.inventory.reservedCol", "Reserved"),
      enableSorting: true,
      cell: ({ row }) => (
        <span className="text-gray-500">
          {t("operations.inventory.units", "{count} units", { count: row.original.reserved })}
        </span>
      ),
    },
    {
      id: "available",
      header: t("operations.inventory.availableBalanceCol", "Available Balance"),
      cell: ({ row }) => {
        const available = row.original.quantity - row.original.reserved;
        return available > 10 ? (
          <Badge variant="solid" color="success">
            {t("operations.inventory.availableBadge", "{count} Available", { count: available })}
          </Badge>
        ) : available > 0 ? (
          <Badge variant="solid" color="warning">
            {t("operations.inventory.lowStockBadge", "{count} Low Stock", { count: available })}
          </Badge>
        ) : (
          <Badge variant="solid" color="error">
            {t("operations.inventory.outOfStockBadge", "Out of Stock")}
          </Badge>
        );
      },
    },
    ...(isSysAdmin
      ? [
          {
            accessorKey: "tenantId",
            header: t("common.tenant", "Tenant"),
            enableSorting: false,
            cell: ({ row }: { row: { original: SkuStock } }) => {
              const tTenant = tenants.find((item) => item.id === row.original.tenantId);
              return (
                <span className="text-xs text-gray-500">
                  {tTenant?.companyName || tTenant?.businessName || row.original.tenantId || "N/A"}
                </span>
              );
            },
          },
        ]
      : []),
    {
      id: "actions",
      header: t("common.actions", "Actions"),
      cell: ({ row }) => (
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            className="h-8 w-8 p-0"
            onClick={() => handleOpenEdit(row.original)}
            aria-label={t("common.edit", "Edit")}
          >
            <Pencil size={14} />
          </Button>
          <Button
            size="sm"
            variant="outline"
            className="h-8 w-8 p-0 text-red-600 hover:text-red-700"
            onClick={() => {
              setItemToDelete(row.original);
              setDeleteConfirmOpen(true);
            }}
            aria-label={t("common.delete", "Delete")}
          >
            <Trash2 size={14} />
          </Button>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title={`${t("operations.inventory.title", "Inventory Stock")} | Veche`}
        description={t("operations.inventory.desc", "Manage SKU inventory stock balances, reserves, and availability")}
      />
      <PageBreadcrumb pageTitle={t("operations.inventory.title", "Inventory Stock")} />

      <ComponentCard
        title={t("operations.inventory.cardTitle", "Warehouse & Inventory Stock")}
        desc={t("operations.inventory.desc", "Monitor on-hand inventory balances, open order reserves, and stock thresholds")}
      >
        <DataGrid
          data={items}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                variant="outline"
                size="sm"
                onClick={loadData}
                disabled={loading}
                startIcon={<RefreshCw size={14} className={loading ? "animate-spin" : ""} />}
              >
                {t("common.refresh", "Refresh")}
              </Button>
              <Button size="sm" onClick={handleOpenAdd} startIcon={<Plus size={14} />}>
                {t("operations.inventory.addRecord", "Set Stock")}
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && items.length === 0}
          error={error}
          emptyMessage={t("operations.inventory.emptyMessage", "No inventory stock records found.")}
          manualPagination
          manualSorting
          manualFiltering
          totalRows={total}
          pagination={{ pageIndex: Math.max(0, page - 1), pageSize }}
          onPaginationChange={(next: PaginationState) => {
            updateSearch({ page: next.pageIndex + 1, pageSize: next.pageSize });
          }}
          sorting={[{ id: sortBy, desc: sortDir === "desc" }]}
          onSortingChange={(next: SortingState) => {
            if (next.length > 0) {
              updateSearch({ sortBy: next[0].id, sortDir: next[0].desc ? "desc" : "asc", page: 1 });
            } else {
              updateSearch({ sortBy: "id", sortDir: "asc", page: 1 });
            }
          }}
          globalFilter={q}
          onGlobalFilterChange={(val: string) => updateSearch({ q: val, page: 1 })}
        />
      </ComponentCard>

      {/* Add / Edit Modal */}
      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-lg p-6"
      >
        <div className="flex items-center gap-3 border-b border-gray-100 pb-4 dark:border-gray-800">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-brand-50 text-brand-600 dark:bg-brand-950/50">
            <Boxes size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingItem ? t("operations.inventory.editStock", "Update Stock Balance") : t("operations.inventory.newStock", "Set Inventory Stock")}
            </h3>
            <p className="text-xs text-gray-500">
              {t("operations.inventory.modalDesc", "Set available units and reservation balances for a SKU")}
            </p>
          </div>
        </div>

        {formError && (
          <div className="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-700 dark:bg-red-950/40 dark:text-red-300">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="mt-4 space-y-4">
          <div>
            <Label htmlFor="skuId">{t("operations.inventory.skuIdLabel", "SKU ID *")}</Label>
            <Input
              id="skuId"
              type="number"
              min={1}
              required
              value={formData.skuId || ""}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, skuId: Number(e.target.value) }))
              }
              placeholder="e.g. 201"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="quantity">{t("operations.inventory.quantityLabel", "Total On-Hand Quantity *")}</Label>
              <Input
                id="quantity"
                type="number"
                min={0}
                required
                value={formData.quantity}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, quantity: Number(e.target.value) }))
                }
                placeholder="0"
              />
            </div>
            <div>
              <Label htmlFor="reserved">{t("operations.inventory.reservedLabel", "Reserved Quantity")}</Label>
              <Input
                id="reserved"
                type="number"
                min={0}
                value={formData.reserved}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, reserved: Number(e.target.value) }))
                }
                placeholder="0"
              />
            </div>
          </div>

          <div className="rounded-lg bg-gray-50 p-3 text-xs text-gray-600 dark:bg-gray-800/50 dark:text-gray-300">
            {t("operations.inventory.availableToSell", "Available to Sell:")}{" "}
            <span className="font-bold text-brand-600 dark:text-brand-400">
              {t("operations.inventory.units", "{count} units", { count: Math.max(0, formData.quantity - formData.reserved) })}
            </span>
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">{t("operations.inventory.tenantLabel", "Tenant (SysAdmin only)")}</Label>
              <select
                id="tenantId"
                className="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-800 dark:border-gray-700 dark:bg-gray-900 dark:text-white/90"
                value={formData.tenantId || ""}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    tenantId: e.target.value ? Number(e.target.value) : null,
                  }))
                }
              >
                <option value="">{t("operations.inventory.defaultTenant", "Default Tenant")}</option>
                {tenants.map((tTenant) => (
                  <option key={tTenant.id} value={tTenant.id ?? undefined}>
                    {tTenant.companyName || tTenant.businessName || `Tenant #${tTenant.id}`}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="flex justify-end gap-3 pt-4 border-t border-gray-100 dark:border-gray-800">
            <Button
              type="button"
              variant="outline"
              onClick={() => setModalOpen(false)}
              disabled={saving}
            >
              {t("common.cancel", "Cancel")}
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? t("common.saving", "Saving...") : editingItem ? t("operations.inventory.updateStock", "Update Stock") : t("operations.inventory.saveStock", "Save Stock")}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={deleteConfirmOpen}
        onClose={() => setDeleteConfirmOpen(false)}
        className="max-w-md p-6"
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{t("operations.inventory.deleteConfirmTitle", "Confirm Deletion")}</h3>
        <p className="mt-2 text-sm text-gray-500">
          {t("operations.inventory.deleteConfirmMsg", "Are you sure you want to remove the inventory balance for SKU #{skuId}? This action cannot be undone.", { skuId: itemToDelete?.skuId })}
        </p>
        <div className="mt-6 flex justify-end gap-3">
          <Button variant="outline" onClick={() => setDeleteConfirmOpen(false)}>
            {t("common.cancel", "Cancel")}
          </Button>
          <Button variant="primary" className="bg-red-600 hover:bg-red-700" onClick={handleDelete}>
            {t("common.delete", "Delete")}
          </Button>
        </div>
      </Modal>
    </>
  );
}
