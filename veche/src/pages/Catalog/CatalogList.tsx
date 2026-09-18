import { useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Pencil, Plus, RefreshCw } from "lucide-react";
import type { ColumnDef } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  fetchCategories,
  fetchProducts,
  fetchSkus,
} from "@/store/catalogSlice";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import CatalogImportModal from "@/components/catalog/CatalogImportModal";

type CatalogRow = {
  id: number;
  name?: string;
  code?: string;
  slug?: string;
  brand?: string;
  variantKey?: string;
  priceCents?: number;
  tenantId?: number | null;
  active: boolean;
};

export default function CatalogList() {
  const { kind = "products" } = useParams();
  const [importOpen, setImportOpen] = useState(false);
  const dispatch = useAppDispatch();
  const state = useAppSelector((s) => s.catalog);
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const user = useAppSelector((s) => s.auth.user);
  const sys = user?.role === ROLES.SYS_ADMIN;
  const tenantId = user?.tenantId ?? user?.tenant_id;
  const title = kind[0].toUpperCase() + kind.slice(1);
  const rows = (
    kind === "categories"
      ? state.categories
      : kind === "skus"
        ? state.skus
        : state.products
  ) as CatalogRow[];
  const filtered = useMemo(
    () => (sys ? rows : rows.filter((row) => row.tenantId === tenantId)),
    [rows, sys, tenantId],
  );
  const load = () => {
    if (kind === "categories") dispatch(fetchCategories());
    else if (kind === "skus") dispatch(fetchSkus());
    else dispatch(fetchProducts());
  };
  useEffect(() => {
    load();
    dispatch(fetchTenants());
  }, [dispatch, kind]);
  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;
  const columns: ColumnDef<CatalogRow, unknown>[] = [
    {
      header: "Name / code",
      id: "name",
      accessorFn: (row) => row.name || row.code || "—",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-medium">{getValue<string>()}</span>
      ),
    },
    {
      header: "Details",
      id: "details",
      accessorFn: (row) =>
        kind === "skus"
          ? `${row.variantKey} · ${((row.priceCents ?? 0) / 100).toFixed(2)}`
          : row.slug || row.brand || "—",
    },
    {
      header: "Tenant",
      id: "tenant",
      cell: ({ row }) => tenantName(row.original.tenantId),
    },
    {
      header: "Status",
      accessorKey: "active",
      cell: ({ row }) => (
        <Badge size="sm" color={row.original.active ? "success" : "light"}>
          {row.original.active ? "Active" : "Inactive"}
        </Badge>
      ),
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      enableHiding: false,
      cell: ({ row }) => (
        <Link
          className="gap-1 text-brand-500 inline-flex items-center"
          to={`/catalog/${kind}/${row.original.id}/edit`}
        >
          <Pencil size={15} />
          Edit
        </Link>
      ),
    },
  ];
  return (
    <>
      <PageMeta
        title={`${title} | Veche`}
        description={`Manage ${title.toLowerCase()}`}
      />
      <PageBreadcrumb pageTitle={title} />
      <ComponentCard
        title={`${title} management`}
        desc={
          sys
            ? "Manage catalog data across all tenants."
            : "Manage catalog data for your tenant."
        }
      >
        <div className="mb-5 gap-3 flex justify-end">
          <Button
            variant="outline"
            startIcon={<RefreshCw size={16} />}
            onClick={load}
          >
            Refresh
          </Button>
          {kind === "products" && (
            <Button variant="outline" onClick={() => setImportOpen(true)}>
              Import CSV/XLSX
            </Button>
          )}
          <Link to={`/catalog/${kind}/new`}>
            <Button startIcon={<Plus size={16} />}>
              Add {kind === "skus" ? "SKU" : kind.slice(0, -1)}
            </Button>
          </Link>
        </div>
        <DataGrid
          data={filtered}
          columns={columns}
          getRowId={(row, index) => String(row.id ?? index)}
          loading={state.loading && filtered.length === 0}
          error={state.error}
          emptyMessage={`No ${title.toLowerCase()} found.`}
        />
      </ComponentCard>
      {importOpen && (
        <CatalogImportModal
          onClose={() => setImportOpen(false)}
          onImported={() => {
            setImportOpen(false);
            load();
          }}
        />
      )}
    </>
  );
}
