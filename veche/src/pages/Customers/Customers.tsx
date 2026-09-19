import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { MapPin, Pencil, Plus, RefreshCw, UserCheck } from "lucide-react";
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
import { customerService } from "@/services/customerService";
import type {
  Customer,
  CustomerAddress,
  CustomerAddressInput,
  CustomerInput,
  PageQueryParams,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

export default function Customers() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [customers, setCustomers] = useState<Customer[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Customer Form Modal State
  const [customerModalOpen, setCustomerModalOpen] = useState(false);
  const [editingCustomer, setEditingCustomer] = useState<Customer | null>(null);
  const [savingCustomer, setSavingCustomer] = useState(false);
  const [customerFormError, setCustomerFormError] = useState<string | null>(null);
  const [customerForm, setCustomerForm] = useState<{
    name: string;
    email: string;
    password?: string;
    cpf: string;
    phone: string;
    active: boolean;
    marketingConsent: boolean;
    tenantId: number | null;
  }>({
    name: "",
    email: "",
    password: "",
    cpf: "",
    phone: "",
    active: true,
    marketingConsent: false,
    tenantId: user?.tenantId || null,
  });

  // Addresses Modal State
  const [addressModalOpen, setAddressModalOpen] = useState(false);
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null);
  const [addresses, setAddresses] = useState<CustomerAddress[]>([]);
  const [loadingAddresses, setLoadingAddresses] = useState(false);
  const [showAddAddress, setShowAddAddress] = useState(false);
  const [savingAddress, setSavingAddress] = useState(false);
  const [addressError, setAddressError] = useState<string | null>(null);
  const [newAddress, setNewAddress] = useState<CustomerAddressInput>({
    tenantId: user?.tenantId || null,
    customerId: 0,
    recipientName: "",
    phone: "",
    cep: "",
    street: "",
    number: "",
    complement: "",
    neighborhood: "",
    city: "",
    uf: "SP",
    isDefault: false,
  });

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const loadCustomers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await customerService.paged(params);
      setCustomers(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load customers";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadCustomers();
      }
    });
    return () => {
      active = false;
    };
  }, [loadCustomers]);

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

  const openCreateCustomerModal = () => {
    setEditingCustomer(null);
    setCustomerForm({
      name: "",
      email: "",
      password: "",
      cpf: "",
      phone: "",
      active: true,
      marketingConsent: false,
      tenantId: user?.tenantId || null,
    });
    setCustomerFormError(null);
    setCustomerModalOpen(true);
  };

  const openEditCustomerModal = (c: Customer) => {
    setEditingCustomer(c);
    setCustomerForm({
      name: c.name,
      email: c.email,
      password: "",
      cpf: c.cpf || "",
      phone: c.phone || "",
      active: c.active,
      marketingConsent: c.marketingConsent,
      tenantId: c.tenantId || null,
    });
    setCustomerFormError(null);
    setCustomerModalOpen(true);
  };

  const handleSaveCustomer = async (e: React.FormEvent) => {
    e.preventDefault();
    setCustomerFormError(null);
    setSavingCustomer(true);
    try {
      const payload: CustomerInput = {
        name: customerForm.name.trim(),
        email: customerForm.email.trim(),
        cpf: customerForm.cpf.trim() || null,
        phone: customerForm.phone.trim() || null,
        active: customerForm.active,
        marketingConsent: customerForm.marketingConsent,
        tenantId: customerForm.tenantId,
        ...(customerForm.password ? { password: customerForm.password } : {}),
      };

      if (editingCustomer?.id) {
        await customerService.update(editingCustomer.id, payload);
      } else {
        await customerService.create(payload);
      }
      setCustomerModalOpen(false);
      loadCustomers();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save customer";
      setCustomerFormError(msg);
    } finally {
      setSavingCustomer(false);
    }
  };

  const openAddressesModal = async (c: Customer) => {
    setSelectedCustomer(c);
    setShowAddAddress(false);
    setAddressError(null);
    setAddressModalOpen(true);
    setLoadingAddresses(true);
    try {
      const res = await customerService.addressesPaged({ customerId: c.id, pageSize: 50 });
      setAddresses(res.items);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load addresses";
      setAddressError(msg);
    } finally {
      setLoadingAddresses(false);
    }
  };

  const handleSaveAddress = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCustomer) return;
    setSavingAddress(true);
    setAddressError(null);
    try {
      await customerService.createAddress({
        ...newAddress,
        customerId: selectedCustomer.id,
        tenantId: selectedCustomer.tenantId,
      });
      setShowAddAddress(false);
      const res = await customerService.addressesPaged({
        customerId: selectedCustomer.id,
        pageSize: 50,
      });
      setAddresses(res.items);
      setNewAddress({
        tenantId: selectedCustomer.tenantId,
        customerId: selectedCustomer.id,
        recipientName: "",
        phone: "",
        cep: "",
        street: "",
        number: "",
        complement: "",
        neighborhood: "",
        city: "",
        uf: "SP",
        isDefault: false,
      });
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to add address";
      setAddressError(msg);
    } finally {
      setSavingAddress(false);
    }
  };

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<Customer, unknown>[] = [
    {
      header: "Customer",
      accessorKey: "name",
      enableSorting: true,
      cell: ({ row }) => (
        <div>
          <div className="font-semibold text-gray-900 dark:text-white">
            {row.original.name}
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400">
            {row.original.email}
          </div>
        </div>
      ),
    },
    {
      header: "CPF / Tax ID",
      accessorKey: "cpf",
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: "Phone",
      accessorKey: "phone",
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: "Marketing",
      accessorKey: "marketingConsent",
      cell: ({ getValue }) => (
        <Badge size="sm" color={getValue<boolean>() ? "success" : "light"}>
          {getValue<boolean>() ? "Opted-In" : "No"}
        </Badge>
      ),
    },
    {
      header: "Status",
      accessorKey: "active",
      cell: ({ getValue }) => (
        <Badge size="sm" color={getValue<boolean>() ? "success" : "light"}>
          {getValue<boolean>() ? "Active" : "Inactive"}
        </Badge>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            header: "Tenant",
            id: "tenant",
            cell: ({ row }: { row: { original: Customer } }) =>
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
            title="Manage Addresses"
            aria-label="Addresses"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openAddressesModal(row.original)}
          >
            <MapPin size={15} />
          </button>
          <button
            type="button"
            title="Edit Customer"
            aria-label="Edit"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openEditCustomerModal(row.original)}
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
        title="Customers | Veche"
        description="Manage customer accounts and delivery addresses"
      />
      <PageBreadcrumb pageTitle="Customers" />
      <ComponentCard>
        <DataGrid
          data={customers}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadCustomers}
              >
                Refresh
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateCustomerModal}
              >
                Add Customer
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && customers.length === 0}
          error={error}
          emptyMessage="No customers found."
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

      {/* Customer Form Modal */}
      <Modal
        isOpen={customerModalOpen}
        onClose={() => setCustomerModalOpen(false)}
        className="max-w-lg p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <UserCheck size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingCustomer ? "Edit Customer" : "Add Customer"}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Customer profile and communication preferences.
            </p>
          </div>
        </div>

        {customerFormError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {customerFormError}
          </div>
        )}

        <form onSubmit={handleSaveCustomer} className="space-y-4">
          <div>
            <Label htmlFor="name">Full Name</Label>
            <Input
              id="name"
              placeholder="Maria Silva"
              value={customerForm.name}
              onChange={(e) =>
                setCustomerForm({ ...customerForm, name: e.target.value })
              }
              required
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="maria@example.com"
                value={customerForm.email}
                onChange={(e) =>
                  setCustomerForm({ ...customerForm, email: e.target.value })
                }
                required
              />
            </div>
            <div>
              <Label htmlFor="password">
                {editingCustomer ? "New Password (Optional)" : "Password"}
              </Label>
              <Input
                id="password"
                type="password"
                placeholder="••••••••"
                value={customerForm.password}
                onChange={(e) =>
                  setCustomerForm({ ...customerForm, password: e.target.value })
                }
                {...(!editingCustomer ? { required: true } : {})}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="cpf">CPF</Label>
              <Input
                id="cpf"
                placeholder="123.456.789-00"
                value={customerForm.cpf}
                onChange={(e) =>
                  setCustomerForm({ ...customerForm, cpf: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="phone">Phone</Label>
              <Input
                id="phone"
                placeholder="+55 (11) 99999-9999"
                value={customerForm.phone}
                onChange={(e) =>
                  setCustomerForm({ ...customerForm, phone: e.target.value })
                }
              />
            </div>
          </div>

          <div className="flex items-center gap-6 pt-2">
            <Switch
              label="Active Account"
              checked={customerForm.active}
              onChange={(checked) =>
                setCustomerForm({ ...customerForm, active: checked })
              }
            />
            <Switch
              label="Marketing Consent"
              checked={customerForm.marketingConsent}
              onChange={(checked) =>
                setCustomerForm({ ...customerForm, marketingConsent: checked })
              }
            />
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">Tenant</Label>
              <select
                id="tenantId"
                value={customerForm.tenantId || ""}
                onChange={(e) =>
                  setCustomerForm({
                    ...customerForm,
                    tenantId: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="">Default / None</option>
                {tenants.map((t) => (
                  <option key={t.id} value={t.id ?? ""}>
                    {t.businessName || t.companyName}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="mt-6 flex justify-end gap-3 pt-3 border-t border-gray-100 dark:border-gray-800">
            <Button
              type="button"
              variant="outline"
              onClick={() => setCustomerModalOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={savingCustomer}>
              {savingCustomer ? "Saving…" : "Save Customer"}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Addresses Modal */}
      <Modal
        isOpen={addressModalOpen}
        onClose={() => setAddressModalOpen(false)}
        className="max-w-2xl p-6"
      >
        <div className="flex items-center justify-between mb-5">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
              <MapPin size={20} />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Addresses for {selectedCustomer?.name}
              </h3>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Delivery and billing locations on file.
              </p>
            </div>
          </div>
          {!showAddAddress && (
            <Button
              size="sm"
              startIcon={<Plus size={14} />}
              onClick={() => setShowAddAddress(true)}
            >
              Add Address
            </Button>
          )}
        </div>

        {addressError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {addressError}
          </div>
        )}

        {showAddAddress ? (
          <form onSubmit={handleSaveAddress} className="space-y-3 border-t border-gray-100 dark:border-gray-800 pt-4">
            <h4 className="text-sm font-semibold text-gray-800 dark:text-gray-200">
              New Address
            </h4>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <Label htmlFor="recipientName">Recipient Name</Label>
                <Input
                  id="recipientName"
                  placeholder="Recipient Name"
                  value={newAddress.recipientName}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, recipientName: e.target.value })
                  }
                  required
                />
              </div>
              <div>
                <Label htmlFor="cep">CEP</Label>
                <Input
                  id="cep"
                  placeholder="01001-000"
                  value={newAddress.cep}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, cep: e.target.value })
                  }
                  required
                />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-3">
              <div className="col-span-2">
                <Label htmlFor="street">Street Address</Label>
                <Input
                  id="street"
                  placeholder="Av. Paulista"
                  value={newAddress.street}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, street: e.target.value })
                  }
                  required
                />
              </div>
              <div>
                <Label htmlFor="number">Number</Label>
                <Input
                  id="number"
                  placeholder="1000"
                  value={newAddress.number}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, number: e.target.value })
                  }
                  required
                />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-3">
              <div>
                <Label htmlFor="complement">Complement</Label>
                <Input
                  id="complement"
                  placeholder="Apt 42"
                  value={newAddress.complement || ""}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, complement: e.target.value })
                  }
                />
              </div>
              <div>
                <Label htmlFor="neighborhood">Neighborhood</Label>
                <Input
                  id="neighborhood"
                  placeholder="Bela Vista"
                  value={newAddress.neighborhood || ""}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, neighborhood: e.target.value })
                  }
                />
              </div>
              <div>
                <Label htmlFor="city">City / UF</Label>
                <div className="flex gap-2">
                  <Input
                    id="city"
                    placeholder="São Paulo"
                    value={newAddress.city}
                    onChange={(e) =>
                      setNewAddress({ ...newAddress, city: e.target.value })
                    }
                    required
                  />
                  <Input
                    id="uf"
                    className="w-16 uppercase"
                    maxLength={2}
                    placeholder="SP"
                    value={newAddress.uf}
                    onChange={(e) =>
                      setNewAddress({ ...newAddress, uf: e.target.value.toUpperCase() })
                    }
                    required
                  />
                </div>
              </div>
            </div>

            <div className="pt-2">
              <Switch
                label="Default delivery address"
                checked={newAddress.isDefault}
                onChange={(checked) =>
                  setNewAddress({ ...newAddress, isDefault: checked })
                }
              />
            </div>

            <div className="flex justify-end gap-2 pt-3">
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowAddAddress(false)}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={savingAddress}>
                {savingAddress ? "Saving…" : "Save Address"}
              </Button>
            </div>
          </form>
        ) : (
          <div className="space-y-3 max-h-96 overflow-y-auto pr-1">
            {loadingAddresses ? (
              <div className="py-8 text-center text-sm text-gray-500">
                Loading addresses…
              </div>
            ) : addresses.length === 0 ? (
              <div className="py-8 text-center text-sm text-gray-500">
                No delivery addresses recorded for this customer.
              </div>
            ) : (
              addresses.map((addr) => (
                <div
                  key={addr.id}
                  className="p-3.5 rounded-xl border border-gray-200 dark:border-gray-800 flex items-start justify-between bg-gray-50/50 dark:bg-gray-800/30"
                >
                  <div className="space-y-0.5">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-sm text-gray-900 dark:text-white">
                        {addr.recipientName}
                      </span>
                      {addr.isDefault && (
                        <Badge size="sm" color="success">
                          Default
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-gray-600 dark:text-gray-300">
                      {addr.street}, {addr.number}
                      {addr.complement ? ` - ${addr.complement}` : ""},{" "}
                      {addr.neighborhood}
                    </p>
                    <p className="text-xs text-gray-400 dark:text-gray-500">
                      {addr.city} - {addr.uf} · CEP: {addr.cep}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </Modal>
    </>
  );
}
