import { useEffect } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { Pencil, Plus, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import { formatPhone } from "@/utils/taxId";
import type { Tenant } from "@/services/types";

export function Tenants() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const { tenantsList, total, loading, error } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);
  const sysAdmin = user?.role === ROLES.SYS_ADMIN;

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  useEffect(() => {
    dispatch(fetchTenants({ page, pageSize, q, sortBy, sortDir }));
  }, [dispatch, page, pageSize, q, sortBy, sortDir]);

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
  const columns: ColumnDef<Tenant, unknown>[] = [
    {
      header: t("tenants.businessName", "Nome Fantasia"),
      id: "businessName",
      accessorFn: (row) => row.businessName || row.companyName || "—",
      enableSorting: true,
    },
    {
      header: t("tenants.email", "E-mail"),
      accessorKey: "email",
      enableSorting: true,
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: t("tenants.phone", "Telefone"),
      accessorKey: "phone",
      enableSorting: true,
      cell: ({ getValue }) => {
        const val = getValue<string>();
        return val ? formatPhone(val) : "—";
      },
    },
    {
      header: t("tenants.location", "Localização"),
      id: "location",
      accessorFn: (row) =>
        [row.locality || row.city, row.administrativeArea || row.province]
          .filter(Boolean)
          .join(", ") || "—",
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      enableHiding: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end text-end">
          <Link
            title={t("common.edit", "Editar")}
            aria-label={t("common.edit", "Editar")}
            className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            to={`/tenants/${row.original.id}/edit`}
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
        title={`${t("tenants.title", "Empresas")} | Veche`}
        description={t("tenants.title", "Empresas")}
      />
      <PageBreadcrumb pageTitle={t("tenants.title", "Empresas")} />
      <ComponentCard>
        <DataGrid
          data={tenantsList}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={() => dispatch(fetchTenants({ page, pageSize, q, sortBy, sortDir }))}
              >
                {t("tenants.refresh", "Atualizar")}
              </Button>
              {sysAdmin && (
                <Link to="/tenants/new">
                  <Button size="sm" startIcon={<Plus size={14} />}>
                    {t("tenants.addTenant", "Nova empresa")}
                  </Button>
                </Link>
              )}
            </div>
          }
          getRowId={(row, index) => String(row.id ?? row.uuid ?? index)}
          loading={loading && tenantsList.length === 0}
          error={error}
          emptyMessage={t("tenants.noTenantsFound", "Nenhuma empresa encontrada.")}
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
    </>
  );
}

export default Tenants;
