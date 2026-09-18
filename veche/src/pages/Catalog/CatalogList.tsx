import { useEffect, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
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
import ProductImagesModal from "@/components/catalog/ProductImagesModal";
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
  const { kind = "products" } = useParams();
  const [searchParams, setSearchParams] = useSearchParams();
  const [importOpen, setImportOpen] = useState(false);
  const [photosProduct, setPhotosProduct] = useState<{ id: number; name?: string } | null>(null);
  const dispatch = useAppDispatch();
  const state = useAppSelector((s) => s.catalog);
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const title = kind[0].toUpperCase() + kind.slice(1);

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
        <div className="flex items-center justify-end gap-1 text-end">
          {kind === "products" && (
            <button
              type="button"
              title="Photos"
              aria-label="Photos"
              className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
              onClick={() => setPhotosProduct({ id: row.original.id, name: row.original.name })}
            >
              <ImageIcon size={15} />
            </button>
          )}
          <Link
            title="Edit"
            aria-label="Edit"
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
        title={`${title} | Veche`}
        description={`Manage ${title.toLowerCase()}`}
      />
      <PageBreadcrumb pageTitle={title} />
      <ComponentCard>
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
                Refresh
              </Button>
              {kind === "products" && (
                <Button size="sm" variant="outline" onClick={() => setImportOpen(true)}>
                  Import CSV/XLSX
                </Button>
              )}
              <Link to={`/catalog/${kind}/new`}>
                <Button size="sm" startIcon={<Plus size={14} />}>
                  Add {kind === "skus" ? "SKU" : kind.slice(0, -1)}
                </Button>
              </Link>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={state.loading && rows.length === 0}
          error={state.error}
          emptyMessage={`No ${title.toLowerCase()} found.`}
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
      {photosProduct && (
        <ProductImagesModal
          productId={photosProduct.id}
          productName={photosProduct.name}
          onClose={() => setPhotosProduct(null)}
        />
      )}
    </>
  );
}

