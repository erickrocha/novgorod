import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { MapPin, Pencil, Plus, RefreshCw } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Radio from "@/components/form/input/Radio";
import Switch from "@/components/form/switch/Switch";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import { BrFlagIcon } from "@/icons";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import {
  createCustomerAddress,
  fetchCustomerAddresses,
  fetchCustomersPaged,
} from "@/store/customerSlice";
import { locationService } from "@/services/locationService";
import { formatPostalCode, stripNonDigits } from "@/utils/taxId";
import type {
  Customer,
  CustomerAddressInput,
  PageQueryParams,
  Province,
  City,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

export function Customers() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const { customersList: customers, paged, loading, error, addresses } = useAppSelector((s) => s.customer);
  const total = paged?.total || 0;
  const loadingAddresses = loading;

  // Addresses Modal State
  const [addressModalOpen, setAddressModalOpen] = useState(false);
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null);
  const [showAddAddress, setShowAddAddress] = useState(false);
  const [savingAddress, setSavingAddress] = useState(false);
  const [addressError, setAddressError] = useState<string | null>(null);
  const [newAddress, setNewAddress] = useState<CustomerAddressInput>({
    tenantId: user?.tenantId || null,
    customerId: 0,
    recipient: "",
    phone: "",
    postalCode: "",
    addressLine1: "",
    addressLine2: "",
    locality: "",
    administrativeArea: "",
    countryCode: "BR",
    isDefault: false,
  });

  const [provinces, setProvinces] = useState<Province[]>([]);
  const [loadingProvinces, setLoadingProvinces] = useState(false);
  const [cities, setCities] = useState<City[]>([]);
  const [loadingCities, setLoadingCities] = useState(false);
  const activeProvinceIdRef = useRef<number | null>(null);

  // Load provinces list
  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (!active) return;
      setLoadingProvinces(true);
      locationService
        .provinces("BR")
        .then((data) => {
          if (active) setProvinces(data);
        })
        .catch((err) => {
          console.error("Failed to load provinces", err);
        })
        .finally(() => {
          if (active) setLoadingProvinces(false);
        });
    });
    return () => {
      active = false;
    };
  }, []);

  const loadCities = useCallback(async (provinceId: number) => {
    activeProvinceIdRef.current = provinceId;
    setLoadingCities(true);
    try {
      const list = await locationService.citiesByProvince(provinceId);
      if (activeProvinceIdRef.current === provinceId) {
        setCities(Array.isArray(list) ? list : []);
      }
    } catch (err) {
      if (activeProvinceIdRef.current === provinceId) {
        console.error("Failed to load cities for province", provinceId, err);
        setCities([]);
      }
    } finally {
      if (activeProvinceIdRef.current === provinceId) {
        setLoadingCities(false);
      }
    }
  }, []);

  // Automatically load cities when administrativeArea changes
  useEffect(() => {
    if (!newAddress.administrativeArea) {
      activeProvinceIdRef.current = null;
      queueMicrotask(() => {
        setCities([]);
      });
      return;
    }
    if (provinces.length === 0) return;

    const target = newAddress.administrativeArea.trim().toLowerCase();
    const prov = provinces.find(
      (p) =>
        p.acronym.toLowerCase() === target || p.name.toLowerCase() === target,
    );
    if (prov && prov.id != null) {
      queueMicrotask(() => {
        loadCities(Number(prov.id));
      });
    } else {
      activeProvinceIdRef.current = null;
      queueMicrotask(() => {
        setCities([]);
      });
    }
  }, [newAddress.administrativeArea, provinces, loadCities]);

  const handleProvinceChange = (val: string, option?: ComboboxOption) => {
    const searchVal = val.trim().toLowerCase();
    const prov =
      (option?.data as Province) ||
      provinces.find(
        (p) =>
          p.acronym.toLowerCase() === searchVal ||
          p.name.toLowerCase() === searchVal,
      );
    const chosenAcronym = prov ? prov.acronym : val;
    setNewAddress((prev) => ({
      ...prev,
      administrativeArea: chosenAcronym,
      locality: "",
    }));
  };

  const provinceOptions: ComboboxOption[] = useMemo(() => {
    return provinces.map((p) => ({
      value: p.acronym,
      label: `${p.name} (${p.acronym})`,
      sublabel: `UF: ${p.acronym}`,
      data: p,
    }));
  }, [provinces]);

  const cityOptions: ComboboxOption[] = useMemo(() => {
    return cities.map((c) => ({
      value: c.name,
      label: c.name,
      data: c,
    }));
  }, [cities]);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const loadCustomers = useCallback(() => {
    const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
    dispatch(fetchCustomersPaged(params));
  }, [dispatch, page, pageSize, q, sortBy, sortDir]);

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

  const openAddressesModal = (c: Customer) => {
    setSelectedCustomer(c);
    setShowAddAddress(false);
    setAddressError(null);
    setNewAddress({
      tenantId: c.tenantId,
      customerId: c.id,
      recipient: "",
      phone: "",
      postalCode: "",
      addressLine1: "",
      addressLine2: "",
      locality: "",
      administrativeArea: "",
      countryCode: "BR",
      isDefault: false,
    });
    setAddressModalOpen(true);
    dispatch(fetchCustomerAddresses({ customerId: c.id, pageSize: 50 }));
  };

  const handleSaveAddress = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCustomer) return;
    setSavingAddress(true);
    setAddressError(null);
    try {
      const res = await dispatch(
        createCustomerAddress({
          ...newAddress,
          postalCode: stripNonDigits(newAddress.postalCode || ""),
          customerId: selectedCustomer.id,
          tenantId: selectedCustomer.tenantId,
        }),
      );
      if (res.meta.requestStatus === "fulfilled") {
        setShowAddAddress(false);
        dispatch(
          fetchCustomerAddresses({
            customerId: selectedCustomer.id,
            pageSize: 50,
          }),
        );
        setNewAddress({
          tenantId: selectedCustomer.tenantId,
          customerId: selectedCustomer.id,
          recipient: "",
          phone: "",
          postalCode: "",
          addressLine1: "",
          addressLine2: "",
          locality: "",
          administrativeArea: "",
          countryCode: "BR",
          isDefault: false,
        });
      } else {
        setAddressError((res.payload as string) || "Failed to add address");
      }
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
            title={t("customers.manageAddresses", "Manage Addresses")}
            aria-label="Addresses"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-brand-50 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openAddressesModal(row.original)}
          >
            <MapPin size={15} />
          </button>
          <Link
            to={`/customers/${row.original.id}/edit`}
            title={t("customers.editCustomer", "Edit Customer")}
            aria-label="Edit"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-brand-50 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
          >
            <Pencil size={15} />
          </Link>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title={`${t("customers.title", "Customers")} | Veche`}
        description="Manage customer accounts and delivery addresses"
      />
      <PageBreadcrumb pageTitle={t("customers.title", "Customers")} />
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
                {t("customers.refresh", "Refresh")}
              </Button>
              <Link to="/customers/new">
                <Button
                  size="sm"
                  startIcon={<Plus size={14} />}
                >
                  {t("customers.addCustomer", "Add Customer")}
                </Button>
              </Link>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && customers.length === 0}
          error={error}
          emptyMessage={t("customers.noCustomersFound", "No customers found.")}
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
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <Label htmlFor="recipient">Recipient Name</Label>
                <Input
                  id="recipient"
                  placeholder="Recipient Name"
                  value={newAddress.recipient}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, recipient: e.target.value })
                  }
                  required
                />
              </div>
              <div>
                <Label htmlFor="postalCode">Postal Code / CEP</Label>
                <Input
                  id="postalCode"
                  placeholder="00000-000"
                  maxLength={9}
                  value={newAddress.postalCode || ""}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, postalCode: formatPostalCode(e.target.value) })
                  }
                  required
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <Label htmlFor="addressLine1">Address Line 1</Label>
                <Input
                  id="addressLine1"
                  placeholder="Street name, number"
                  value={newAddress.addressLine1 || ""}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, addressLine1: e.target.value })
                  }
                  required
                />
              </div>
              <div>
                <Label htmlFor="addressLine2">Address Line 2 (Optional)</Label>
                <Input
                  id="addressLine2"
                  placeholder="Apt, suite, neighborhood"
                  value={newAddress.addressLine2 || ""}
                  onChange={(e) =>
                    setNewAddress({ ...newAddress, addressLine2: e.target.value })
                  }
                />
              </div>
            </div>



            {/* Country (20%), State/UF (30%), City (50%) matching Tenant & Person Address standard */}
            <div className="grid grid-cols-1 md:grid-cols-10 gap-3 items-end">
              <div className="md:col-span-2">
                <Label>Country</Label>
                <div className="flex h-11 items-center px-1">
                  <Radio
                    id="customer-country-br"
                    name="customer-country"
                    value="BR"
                    checked={true}
                    onChange={() => {}}
                    label={
                      <span className="flex items-center gap-2">
                        <BrFlagIcon className="size-5 rounded-full overflow-hidden shrink-0 shadow-xs" />
                        <span className="text-sm font-medium text-gray-800 dark:text-gray-200">
                          Brasil
                        </span>
                      </span>
                    }
                  />
                </div>
              </div>
              <div className="md:col-span-3">
                <Label htmlFor="customerStateProvince">State / Province</Label>
                <FilterableCombobox
                  id="customerStateProvince"
                  value={newAddress.administrativeArea || ""}
                  options={provinceOptions}
                  onChange={handleProvinceChange}
                  placeholder="Select state..."
                  loading={loadingProvinces}
                  emptyText="No state found"
                  required
                />
              </div>
              <div className="md:col-span-5">
                <Label htmlFor="customerCity">City</Label>
                <FilterableCombobox
                  id="customerCity"
                  value={newAddress.locality || ""}
                  options={cityOptions}
                  onChange={(val) =>
                    setNewAddress((prev) => ({ ...prev, locality: val }))
                  }
                  placeholder={
                    !newAddress.administrativeArea
                      ? "Select state first..."
                      : loadingCities
                      ? "Loading cities..."
                      : "Select or search city..."
                  }
                  disabled={!newAddress.administrativeArea}
                  loading={loadingCities}
                  emptyText={
                    loadingCities
                      ? "Loading cities..."
                      : "No city found"
                  }
                  required
                />
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
                        {addr.recipient}
                      </span>
                      {addr.isDefault && (
                        <Badge size="sm" color="success">
                          Default
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-gray-600 dark:text-gray-300">
                      {addr.addressLine1}
                      {addr.addressLine2 ? ` - ${addr.addressLine2}` : ""}
                    </p>
                    <p className="text-xs text-gray-400 dark:text-gray-500">
                      {addr.locality} - {addr.administrativeArea} · {addr.postalCode}
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

export default Customers;
