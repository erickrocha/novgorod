import { useEffect, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Image as ImageIcon, Pencil, Plus, RefreshCw } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
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
import CatalogImportModal from "@/components/catalog/CatalogImportModal";
import { ROLES } from "@/utils/enums";
import type { PageQueryParams } from "@/services/types";

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
  const { t } = useTranslation();
  const { kind = "products" } = useParams();
  const [searchParams, setSearchParams] = useSearchParams();
  const [importOpen, setImportOpen] = useState(false);
  const dispatch = useAppDispatch();
  const state = useAppSelector((s) => s.catalog);
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const canImport = user?.role === ROLES.SYS_ADMIN || user?.role === ROLES.TENANT_OWNER;
  const defaultTitle = kind[0].toUpperCase() + kind.slice(1);
  const catalogTitle = t(`catalog.${kind}.title`, defaultTitle);
  const catalogDesc = t(`catalog.${kind}.desc`, `Manage ${defaultTitle.toLowerCase()}`);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const rows = (
    kind === "categories"
      ? state.categories
      : kind === "skus"
        ? state.skus
        : state.products
  ) as CatalogRow[];

  const total =
    kind === "categories"
      ? state.categoriesTotal
      : kind === "skus"
        ? state.skusTotal
        : state.productsTotal;

  const load = () => {
    const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
    if (kind === "categories") dispatch(fetchCategories(params));
    else if (kind === "skus") dispatch(fetchSkus(params));
    else dispatch(fetchProducts(params));
  };

  useEffect(() => {
    load();
  }, [dispatch, kind, page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    dispatch(fetchTenants());
  }, [dispatch]);

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

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<CatalogRow, unknown>[] = [
    {
      header: t("catalog.columns.nameOrCode", "Name / code"),
      id: "name",
      accessorFn: (row) => row.name || row.code || "—",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-medium">{getValue<string>()}</span>
      ),
    },
    {
      header: t("catalog.columns.details", "Details"),
      id: "details",
      accessorFn: (row) =>
        kind === "skus"
          ? `${row.variantKey} · ${((row.priceCents ?? 0) / 100).toFixed(2)}`
          : row.slug || row.brand || "—",
    },
    {
      header: t("catalog.columns.tenant", "Tenant"),
      id: "tenant",
      cell: ({ row }) => tenantName(row.original.tenantId),
    },
    {
      header: t("catalog.columns.status", "Status"),
      accessorKey: "active",
      cell: ({ row }) => (
        <Badge size="sm" color={row.original.active ? "success" : "light"}>
          {row.original.active
            ? t("common.active", "Active")
            : t("common.inactive", "Inactive")}
        </Badge>
      ),
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      enableHiding: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end gap-1 text-end">
          {kind === "products" && (
            <Link
              title={t("catalog.photos.managePhotos", "Photos")}
              aria-label={t("catalog.photos.managePhotos", "Photos")}
              className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
              to={`/catalog/products/${row.original.id}/photos`}
            >
              <ImageIcon size={15} />
            </Link>
          )}
          <Link
            title={t("common.edit", "Edit")}
            aria-label={t("common.edit", "Edit")}
            className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            to={`/catalog/${kind}/${row.original.id}/edit`}
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
        title={`${catalogTitle} | Veche`}
        description={catalogDesc}
      />
      <PageBreadcrumb pageTitle={catalogTitle} />
      <ComponentCard
        title={catalogTitle}
        desc={catalogDesc}
      >
        <DataGrid
          data={rows}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={load}
              >
                {t("catalog.refresh", t("common.refresh", "Refresh"))}
              </Button>
              {kind === "products" && canImport && (
                <Button size="sm" variant="outline" onClick={() => setImportOpen(true)}>
                  {t("catalog.importCsvXlsx", "Import CSV/XLSX")}
                </Button>
              )}
              <Link to={`/catalog/${kind}/new`}>
                <Button size="sm" startIcon={<Plus size={14} />}>
                  {t(`catalog.${kind}.addTitle`, `Add ${kind === "skus" ? "SKU" : kind.slice(0, -1)}`)}
                </Button>
              </Link>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={state.loading && rows.length === 0}
          error={state.error}
          emptyMessage={t(`catalog.${kind}.emptyMessage`, `No ${defaultTitle.toLowerCase()} found.`)}
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

