import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Eye, Package, RefreshCw, ShoppingCart } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { cartService } from "@/services/cartService";
import type { Cart, CartItem, PageQueryParams } from "@/services/types";
import { ROLES } from "@/utils/enums";

const CART_STATUS_COLORS: Record<string, "success" | "warning" | "info" | "light"> = {
  ACTIVE: "info",
  ABANDONED: "warning",
  CONVERTED: "success",
};

export default function CartsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [carts, setCarts] = useState<Cart[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Detail Modal State
  const [selectedCart, setSelectedCart] = useState<Cart | null>(null);
  const [items, setItems] = useState<CartItem[]>([]);
  const [loadingItems, setLoadingItems] = useState(false);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "desc";

  const loadCarts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await cartService.paged(params);
      setCarts(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load carts";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadCarts();
      }
    });
    return () => {
      active = false;
    };
  }, [loadCarts]);

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

  const openCartDetails = async (cart: Cart) => {
    setSelectedCart(cart);
    setLoadingItems(true);
    try {
      const res = await cartService.itemsPaged({ cartId: cart.id, pageSize: 50 });
      setItems(res.items);
    } catch {
      setItems([]);
    } finally {
      setLoadingItems(false);
    }
  };

  const formatCurrency = (cents: number) =>
    (cents / 100).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<Cart, unknown>[] = [
    {
      header: "Cart ID",
      accessorKey: "id",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-bold text-gray-900 dark:text-white">
          #{getValue<number>()}
        </span>
      ),
    },
    {
      header: "Customer / Session",
      id: "owner",
      cell: ({ row }) => (
        <div>
          {row.original.customerId ? (
            <span className="font-medium text-gray-900 dark:text-white">
              Customer #{row.original.customerId}
            </span>
          ) : (
            <span className="text-gray-500 font-mono text-xs">
              Guest ({row.original.sessionToken ? row.original.sessionToken.slice(0, 10) + "…" : "—"})
            </span>
          )}
        </div>
      ),
    },
    {
      header: "Status",
      accessorKey: "status",
      enableSorting: true,
      cell: ({ getValue }) => {
        const status = (getValue<string>() || "ACTIVE").toUpperCase();
        const color = CART_STATUS_COLORS[status] || "light";
        return (
          <Badge size="sm" color={color}>
            {status}
          </Badge>
        );
      },
    },
    ...(isSysAdmin
      ? [
          {
            header: "Tenant",
            id: "tenant",
            cell: ({ row }: { row: { original: Cart } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      header: "Created At",
      accessorKey: "createdAt",
      enableSorting: true,
      cell: ({ getValue }) => {
        const d = getValue<string>();
        return d ? new Date(d).toLocaleString("pt-BR") : "—";
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
            title="View Cart Items"
            aria-label="View Items"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openCartDetails(row.original)}
          >
            <Eye size={15} />
          </button>
        </div>
      ),
    },
  ];

  const cartTotalCents = items.reduce((acc, it) => acc + it.totalCents, 0);

  return (
    <>
      <PageMeta
        title="Shopping Carts | Veche"
        description="Monitor active and abandoned shopping carts"
      />
      <PageBreadcrumb pageTitle="Shopping Carts" />
      <ComponentCard>
        <DataGrid
          data={carts}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadCarts}
              >
                Refresh
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && carts.length === 0}
          error={error}
          emptyMessage="No carts found."
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

      {/* Cart Detail Modal */}
      <Modal
        isOpen={!!selectedCart}
        onClose={() => setSelectedCart(null)}
        className="max-w-2xl p-6"
      >
        {selectedCart && (
          <div className="space-y-5">
            <div className="flex items-center justify-between border-b border-gray-100 dark:border-gray-800 pb-4">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
                  <ShoppingCart size={20} />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900 dark:text-white flex items-center gap-2">
                    Cart #{selectedCart.id}
                    <Badge
                      size="sm"
                      color={
                        CART_STATUS_COLORS[selectedCart.status.toUpperCase()] || "light"
                      }
                    >
                      {selectedCart.status.toUpperCase()}
                    </Badge>
                  </h3>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {selectedCart.customerId
                      ? `Customer #${selectedCart.customerId}`
                      : "Anonymous Session"}
                  </p>
                </div>
              </div>
            </div>

            {loadingItems ? (
              <div className="py-8 text-center text-sm text-gray-500">
                Loading cart items…
              </div>
            ) : items.length === 0 ? (
              <div className="py-8 text-center text-sm text-gray-500">
                This cart has no active items.
              </div>
            ) : (
              <div className="space-y-4">
                <h4 className="text-sm font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                  <Package size={16} /> Cart Items ({items.length})
                </h4>
                <div className="rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                  <table className="w-full text-left text-xs">
                    <thead className="bg-gray-50 dark:bg-gray-800/60 text-gray-500 border-b border-gray-200 dark:border-gray-800">
                      <tr>
                        <th className="p-2.5">SKU ID</th>
                        <th className="p-2.5 text-center">Quantity</th>
                        <th className="p-2.5 text-right">Unit Price</th>
                        <th className="p-2.5 text-right">Total</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                      {items.map((it) => (
                        <tr key={it.id}>
                          <td className="p-2.5 font-medium text-gray-900 dark:text-white">
                            SKU #{it.skuId}
                          </td>
                          <td className="p-2.5 text-center font-semibold">
                            {it.quantity}
                          </td>
                          <td className="p-2.5 text-right text-gray-600 dark:text-gray-300">
                            {formatCurrency(it.unitPriceCents)}
                          </td>
                          <td className="p-2.5 text-right font-bold text-gray-900 dark:text-white">
                            {formatCurrency(it.totalCents)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                    <tfoot className="bg-gray-50/50 dark:bg-gray-800/40 border-t border-gray-200 dark:border-gray-800 font-bold text-sm">
                      <tr>
                        <td colSpan={3} className="p-3 text-right">
                          Estimated Total:
                        </td>
                        <td className="p-3 text-right text-brand-600 dark:text-brand-400">
                          {formatCurrency(cartTotalCents)}
                        </td>
                      </tr>
                    </tfoot>
                  </table>
                </div>
              </div>
            )}
          </div>
        )}
      </Modal>
    </>
  );
}
