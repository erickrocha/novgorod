import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Eye, Package, RefreshCw, ShoppingBag, Truck } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { orderService } from "@/services/orderService";
import { customerService } from "@/services/customerService";
import type {
  Customer,
  OrderItem,
  Orders,
  OrderStatusHistory,
  PageQueryParams,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

const ORDER_STATUS_COLORS: Record<string, "warning" | "success" | "info" | "error" | "light"> = {
  PENDING: "warning",
  PAID: "success",
  PROCESSING: "info",
  SHIPPED: "info",
  DELIVERED: "success",
  CANCELLED: "error",
};

export default function OrdersPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [orders, setOrders] = useState<Orders[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Detail Modal State
  const [selectedOrder, setSelectedOrder] = useState<Orders | null>(null);
  const [orderCustomer, setOrderCustomer] = useState<Customer | null>(null);
  const [items, setItems] = useState<OrderItem[]>([]);
  const [histories, setHistories] = useState<OrderStatusHistory[]>([]);
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [updatingStatus, setUpdatingStatus] = useState(false);
  const [newStatus, setNewStatus] = useState("PAID");
  const [statusNote, setStatusNote] = useState("");
  const [statusError, setStatusError] = useState<string | null>(null);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "desc";

  const loadOrders = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await orderService.paged(params);
      setOrders(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load orders";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadOrders();
      }
    });
    return () => {
      active = false;
    };
  }, [loadOrders]);

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

  const openOrderDetails = async (order: Orders) => {
    setSelectedOrder(order);
    setNewStatus(order.status);
    setStatusNote("");
    setStatusError(null);
    setLoadingDetails(true);
    try {
      const [itemsRes, historyRes, customerRes] = await Promise.all([
        orderService.itemsPaged({ orderId: order.id, pageSize: 50 }),
        orderService.statusHistoriesPaged({ orderId: order.id, pageSize: 50 }),
        customerService.getById(order.customerId).catch(() => null),
      ]);
      setItems(itemsRes.items);
      setHistories(historyRes.items);
      setOrderCustomer(customerRes);
    } catch {
      // Gracefully handle detail failures
    } finally {
      setLoadingDetails(false);
    }
  };

  const handleUpdateStatus = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedOrder) return;
    setUpdatingStatus(true);
    setStatusError(null);
    try {
      await orderService.addStatusHistory({
        tenantId: selectedOrder.tenantId,
        orderId: selectedOrder.id,
        fromStatus: selectedOrder.status,
        toStatus: newStatus,
        actorType: user?.role || "SYSADMIN",
        actorId: user?.userId,
        note: statusNote.trim() || null,
      });

      await orderService.update(selectedOrder.id, {
        tenantId: selectedOrder.tenantId,
        orderNumber: selectedOrder.orderNumber,
        customerId: selectedOrder.customerId,
        status: newStatus,
        subtotalCents: selectedOrder.subtotalCents,
        discountCents: selectedOrder.discountCents,
        shippingCents: selectedOrder.shippingCents,
        totalCents: selectedOrder.totalCents,
        couponId: selectedOrder.couponId,
      });

      setSelectedOrder({ ...selectedOrder, status: newStatus });
      const updatedHistories = await orderService.statusHistoriesPaged({
        orderId: selectedOrder.id,
        pageSize: 50,
      });
      setHistories(updatedHistories.items);
      setStatusNote("");
      loadOrders();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to update status";
      setStatusError(msg);
    } finally {
      setUpdatingStatus(false);
    }
  };

  const formatCurrency = (cents: number) =>
    (cents / 100).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<Orders, unknown>[] = [
    {
      header: "Order #",
      accessorKey: "orderNumber",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-bold text-gray-900 dark:text-white">
          #{getValue<string>()}
        </span>
      ),
    },
    {
      header: "Customer ID",
      accessorKey: "customerId",
      cell: ({ getValue }) => `Customer #${getValue<number>()}`,
    },
    {
      header: "Status",
      accessorKey: "status",
      enableSorting: true,
      cell: ({ getValue }) => {
        const status = getValue<string>() || "PENDING";
        const color = ORDER_STATUS_COLORS[status.toUpperCase()] || "light";
        return (
          <Badge size="sm" color={color}>
            {status.toUpperCase()}
          </Badge>
        );
      },
    },
    {
      header: "Total",
      accessorKey: "totalCents",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-semibold text-gray-900 dark:text-white">
          {formatCurrency(getValue<number>() || 0)}
        </span>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            header: "Tenant",
            id: "tenant",
            cell: ({ row }: { row: { original: Orders } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      header: "Date",
      accessorKey: "createdAt",
      enableSorting: true,
      cell: ({ getValue }) => {
        const dateStr = getValue<string>();
        return dateStr ? new Date(dateStr).toLocaleString("pt-BR") : "—";
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
            title="View Details"
            aria-label="View Details"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openOrderDetails(row.original)}
          >
            <Eye size={15} />
          </button>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title="Orders | Veche"
        description="Monitor order fulfillments, line items, and delivery statuses"
      />
      <PageBreadcrumb pageTitle="Orders" />
      <ComponentCard>
        <DataGrid
          data={orders}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadOrders}
              >
                Refresh
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && orders.length === 0}
          error={error}
          emptyMessage="No orders found."
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

      {/* Order Detail Modal */}
      <Modal
        isOpen={!!selectedOrder}
        onClose={() => setSelectedOrder(null)}
        className="max-w-3xl p-6"
      >
        {selectedOrder && (
          <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between border-b border-gray-100 dark:border-gray-800 pb-4">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
                  <ShoppingBag size={20} />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900 dark:text-white flex items-center gap-2">
                    Order #{selectedOrder.orderNumber}
                    <Badge
                      size="sm"
                      color={
                        ORDER_STATUS_COLORS[selectedOrder.status.toUpperCase()] ||
                        "light"
                      }
                    >
                      {selectedOrder.status.toUpperCase()}
                    </Badge>
                  </h3>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    Placed on {selectedOrder.createdAt ? new Date(selectedOrder.createdAt).toLocaleString("pt-BR") : "—"}
                  </p>
                </div>
              </div>
            </div>

            {loadingDetails ? (
              <div className="py-12 text-center text-sm text-gray-500">
                Loading order details…
              </div>
            ) : (
              <>
                {/* Customer & Breakdown Grid */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="p-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/30">
                    <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                      Customer Info
                    </h4>
                    <p className="text-sm font-semibold text-gray-900 dark:text-white">
                      {orderCustomer?.name || `Customer #${selectedOrder.customerId}`}
                    </p>
                    {orderCustomer?.email && (
                      <p className="text-xs text-gray-500">{orderCustomer.email}</p>
                    )}
                    {orderCustomer?.phone && (
                      <p className="text-xs text-gray-500">{orderCustomer.phone}</p>
                    )}
                  </div>

                  <div className="p-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/30">
                    <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                      Financial Summary
                    </h4>
                    <div className="space-y-1 text-xs text-gray-600 dark:text-gray-300">
                      <div className="flex justify-between">
                        <span>Subtotal:</span>
                        <span>{formatCurrency(selectedOrder.subtotalCents)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Shipping:</span>
                        <span>{formatCurrency(selectedOrder.shippingCents)}</span>
                      </div>
                      <div className="flex justify-between text-error-600 dark:text-error-400">
                        <span>Discount:</span>
                        <span>- {formatCurrency(selectedOrder.discountCents)}</span>
                      </div>
                      <div className="flex justify-between font-bold text-sm text-gray-900 dark:text-white pt-1 border-t border-gray-200 dark:border-gray-700">
                        <span>Total:</span>
                        <span>{formatCurrency(selectedOrder.totalCents)}</span>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Items Table */}
                <div>
                  <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-2.5 flex items-center gap-2">
                    <Package size={16} /> Line Items ({items.length})
                  </h4>
                  <div className="rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                    <table className="w-full text-left text-xs">
                      <thead className="bg-gray-50 dark:bg-gray-800/60 text-gray-500 border-b border-gray-200 dark:border-gray-800">
                        <tr>
                          <th className="p-2.5">SKU / Product</th>
                          <th className="p-2.5 text-center">Qty</th>
                          <th className="p-2.5 text-right">Unit Price</th>
                          <th className="p-2.5 text-right">Total</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                        {items.map((item) => (
                          <tr key={item.id}>
                            <td className="p-2.5 font-medium text-gray-900 dark:text-white">
                              {item.productName}
                              {item.skuCode && (
                                <span className="block text-gray-400 text-2xs">
                                  SKU: {item.skuCode}
                                </span>
                              )}
                            </td>
                            <td className="p-2.5 text-center">{item.quantity}</td>
                            <td className="p-2.5 text-right">
                              {formatCurrency(item.unitPriceCents)}
                            </td>
                            <td className="p-2.5 text-right font-semibold">
                              {formatCurrency(item.totalCents)}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Status Timeline */}
                <div>
                  <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-2.5 flex items-center gap-2">
                    <Truck size={16} /> Status Timeline
                  </h4>
                  <div className="space-y-2 max-h-40 overflow-y-auto pr-1">
                    {histories.length === 0 ? (
                      <p className="text-xs text-gray-400">No status events recorded yet.</p>
                    ) : (
                      histories.map((h) => (
                        <div
                          key={h.id}
                          className="flex items-center justify-between p-2 rounded-lg bg-gray-50 dark:bg-gray-800/40 text-xs"
                        >
                          <div className="flex items-center gap-2">
                            <span className="font-semibold text-gray-800 dark:text-gray-200">
                              {h.fromStatus ? `${h.fromStatus} → ` : ""}
                              {h.toStatus}
                            </span>
                            {h.note && (
                              <span className="text-gray-500 italic">“{h.note}”</span>
                            )}
                          </div>
                          <span className="text-gray-400 text-2xs">
                            {h.createdAt ? new Date(h.createdAt).toLocaleString("pt-BR") : "—"}
                          </span>
                        </div>
                      ))
                    )}
                  </div>
                </div>

                {/* Status Transition Control */}
                <form
                  onSubmit={handleUpdateStatus}
                  className="p-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-gray-50/70 dark:bg-gray-800/50 space-y-3"
                >
                  <h4 className="text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Update Order Status
                  </h4>
                  {statusError && (
                    <p className="text-xs text-error-600">{statusError}</p>
                  )}
                  <div className="grid grid-cols-3 gap-3">
                    <div>
                      <Label htmlFor="targetStatus">New Status</Label>
                      <select
                        id="targetStatus"
                        value={newStatus}
                        onChange={(e) => setNewStatus(e.target.value)}
                        className="h-10 rounded-lg border-gray-300 px-3 text-xs w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
                      >
                        <option value="PENDING">PENDING</option>
                        <option value="PAID">PAID</option>
                        <option value="PROCESSING">PROCESSING</option>
                        <option value="SHIPPED">SHIPPED</option>
                        <option value="DELIVERED">DELIVERED</option>
                        <option value="CANCELLED">CANCELLED</option>
                      </select>
                    </div>
                    <div className="col-span-2">
                      <Label htmlFor="statusNote">Status Note</Label>
                      <div className="flex gap-2">
                        <Input
                          id="statusNote"
                          placeholder="e.g. Tracking code BR123456789 or payment confirmed"
                          value={statusNote}
                          onChange={(e) => setStatusNote(e.target.value)}
                        />
                        <Button
                          type="submit"
                          size="sm"
                          disabled={updatingStatus || newStatus === selectedOrder.status}
                        >
                          {updatingStatus ? "Updating…" : "Update"}
                        </Button>
                      </div>
                    </div>
                  </div>
                </form>
              </>
            )}
          </div>
        )}
      </Modal>
    </>
  );
}
