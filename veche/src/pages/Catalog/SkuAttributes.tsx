import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Layers, Pencil, Plus, RefreshCw, Trash2 } from "lucide-react";
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
import { skuAttributeService } from "@/services/skuAttributeService";
import type { PageQueryParams, SkuAttributeValue, SkuAttributeValueInput } from "@/services/types";
import { ROLES } from "@/utils/enums";

export default function SkuAttributes() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [items, setItems] = useState<SkuAttributeValue[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Modal State
  const [modalOpen, setModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<SkuAttributeValue | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    productId: number;
    skuId: number;
    productAttributeId: number;
    attributeId: number;
    attributeValueId: number;
    tenantId: number | null;
  }>({
    productId: 0,
    skuId: 0,
    productAttributeId: 0,
    attributeId: 0,
    attributeValueId: 0,
    tenantId: user?.tenantId || null,
  });

  // Delete State
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [itemToDelete, setItemToDelete] = useState<SkuAttributeValue | null>(null);

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
      const res = await skuAttributeService.paged(params);
      setItems(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load SKU attributes";
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
      productId: 0,
      skuId: 0,
      productAttributeId: 0,
      attributeId: 0,
      attributeValueId: 0,
      tenantId: user?.tenantId || null,
    });
    setModalOpen(true);
  };

  const handleOpenEdit = (item: SkuAttributeValue) => {
    setEditingItem(item);
    setFormError(null);
    setFormData({
      productId: item.productId,
      skuId: item.skuId,
      productAttributeId: item.productAttributeId,
      attributeId: item.attributeId,
      attributeValueId: item.attributeValueId,
      tenantId: item.tenantId || null,
    });
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (
      formData.productId <= 0 ||
      formData.skuId <= 0 ||
      formData.productAttributeId <= 0 ||
      formData.attributeId <= 0 ||
      formData.attributeValueId <= 0
    ) {
      setFormError("All entity IDs must be valid positive integers.");
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      const payload: SkuAttributeValueInput = {
        productId: Number(formData.productId),
        skuId: Number(formData.skuId),
        productAttributeId: Number(formData.productAttributeId),
        attributeId: Number(formData.attributeId),
        attributeValueId: Number(formData.attributeValueId),
        tenantId: formData.tenantId,
      };

      if (editingItem?.id) {
        await skuAttributeService.update(editingItem.id, payload);
      } else {
        await skuAttributeService.create(payload);
      }
      setModalOpen(false);
      loadData();
    } catch (err: unknown) {
      setFormError(err instanceof Error ? err.message : "Failed to save SKU attribute");
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!itemToDelete?.id) return;
    try {
      await skuAttributeService.delete(itemToDelete.id);
      setDeleteConfirmOpen(false);
      setItemToDelete(null);
      loadData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to delete item");
    }
  };

  const columns: ColumnDef<SkuAttributeValue>[] = [
    {
      accessorKey: "id",
      header: "ID",
      enableSorting: true,
      cell: ({ row }) => <span className="font-mono text-xs">{row.original.id}</span>,
    },
    {
      accessorKey: "skuId",
      header: "SKU ID",
      enableSorting: true,
      cell: ({ row }) => (
        <span className="font-medium text-gray-800 dark:text-white/90">
          #{row.original.skuId}
        </span>
      ),
    },
    {
      accessorKey: "productId",
      header: "Product ID",
      enableSorting: true,
      cell: ({ row }) => (
        <span className="text-gray-700 dark:text-gray-300">
          #{row.original.productId}
        </span>
      ),
    },
    {
      accessorKey: "productAttributeId",
      header: "Product Spec ID",
      enableSorting: false,
      cell: ({ row }) => (
        <span className="text-xs text-gray-500">
          #{row.original.productAttributeId}
        </span>
      ),
    },
    {
      accessorKey: "attributeId",
      header: "Attribute ID",
      enableSorting: true,
      cell: ({ row }) => (
        <span className="text-gray-700 dark:text-gray-300">
          #{row.original.attributeId}
        </span>
      ),
    },
    {
      accessorKey: "attributeValueId",
      header: "Value ID",
      enableSorting: false,
      cell: ({ row }) => (
        <span className="font-semibold text-brand-600 dark:text-brand-400">
          #{row.original.attributeValueId}
        </span>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            accessorKey: "tenantId",
            header: "Tenant",
            enableSorting: false,
            cell: ({ row }: { row: { original: SkuAttributeValue } }) => {
              const t = tenants.find((item) => item.id === row.original.tenantId);
              return (
                <span className="text-xs text-gray-500">
                  {t?.companyName || t?.businessName || row.original.tenantId || "N/A"}
                </span>
              );
            },
          },
        ]
      : []),
    {
      id: "actions",
      header: "Actions",
      cell: ({ row }) => (
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            className="h-8 w-8 p-0"
            onClick={() => handleOpenEdit(row.original)}
            aria-label="Edit"
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
            aria-label="Delete"
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
        title="SKU Attributes | Novgorod Admin"
        description="Manage SKU variant attributes and option values"
      />
      <PageBreadcrumb pageTitle="SKU Attributes" />

      <ComponentCard
        title="SKU Variant Attributes"
        desc="Map SKU variants to specific attribute values (e.g. Size, Color, Capacity)"
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
                Refresh
              </Button>
              <Button size="sm" onClick={handleOpenAdd} startIcon={<Plus size={14} />}>
                Assign Attribute
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && items.length === 0}
          error={error}
          emptyMessage="No SKU attributes found."
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
            <Layers size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingItem ? "Edit Variant Attribute" : "Assign Variant Attribute"}
            </h3>
            <p className="text-xs text-gray-500">
              Associate SKU with catalog attribute values
            </p>
          </div>
        </div>

        {formError && (
          <div className="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-700 dark:bg-red-950/40 dark:text-red-300">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="mt-4 space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="productId">Product ID *</Label>
              <Input
                id="productId"
                type="number"
                min={1}
                required
                value={formData.productId || ""}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, productId: Number(e.target.value) }))
                }
                placeholder="e.g. 10"
              />
            </div>
            <div>
              <Label htmlFor="skuId">SKU ID *</Label>
              <Input
                id="skuId"
                type="number"
                min={1}
                required
                value={formData.skuId || ""}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, skuId: Number(e.target.value) }))
                }
                placeholder="e.g. 20"
              />
            </div>
          </div>

          <div>
            <Label htmlFor="productAttributeId">Product Attribute Spec ID *</Label>
            <Input
              id="productAttributeId"
              type="number"
              min={1}
              required
              value={formData.productAttributeId || ""}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, productAttributeId: Number(e.target.value) }))
              }
              placeholder="e.g. 1"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="attributeId">Attribute ID *</Label>
              <Input
                id="attributeId"
                type="number"
                min={1}
                required
                value={formData.attributeId || ""}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, attributeId: Number(e.target.value) }))
                }
                placeholder="e.g. 2"
              />
            </div>
            <div>
              <Label htmlFor="attributeValueId">Attribute Value ID *</Label>
              <Input
                id="attributeValueId"
                type="number"
                min={1}
                required
                value={formData.attributeValueId || ""}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, attributeValueId: Number(e.target.value) }))
                }
                placeholder="e.g. 5"
              />
            </div>
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">Tenant (SysAdmin only)</Label>
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
                <option value="">Default Tenant</option>
                {tenants.map((t) => (
                  <option key={t.id} value={t.id ?? undefined}>
                    {t.companyName || t.businessName || `Tenant #${t.id}`}
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
              Cancel
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? "Saving..." : editingItem ? "Update Assignment" : "Assign Attribute"}
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
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Confirm Deletion</h3>
        <p className="mt-2 text-sm text-gray-500">
          Are you sure you want to remove this SKU attribute assignment (SKU #{itemToDelete?.skuId} &rarr; Value #{itemToDelete?.attributeValueId})? This action cannot be undone.
        </p>
        <div className="mt-6 flex justify-end gap-3">
          <Button variant="outline" onClick={() => setDeleteConfirmOpen(false)}>
            Cancel
          </Button>
          <Button variant="primary" className="bg-red-600 hover:bg-red-700" onClick={handleDelete}>
            Delete
          </Button>
        </div>
      </Modal>
    </>
  );
}
