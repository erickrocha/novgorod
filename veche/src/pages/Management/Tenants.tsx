import { useEffect } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { Plus, RefreshCw } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import type { Tenant } from "@/services/types";

export default function Tenants() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const { tenantsList, total, loading, error } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);
  const sysAdmin = user?.role === ROLES.SYS_ADMIN;

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "25");
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
      header: "Business name",
      id: "businessName",
      accessorFn: (row) => row.businessName || row.companyName || "—",
      enableSorting: true,
    },
    {
      header: "Email",
      accessorKey: "email",
      enableSorting: true,
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: "Phone",
      accessorKey: "phone",
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: "Location",
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
        <Link
          className="text-brand-500"
          to={`/tenants/${row.original.id}/edit`}
        >
          Edit
        </Link>
      ),
    },
  ];
  return (
    <>
      <PageMeta title="Tenants | Veche" description="Manage tenants" />
      <PageBreadcrumb pageTitle="Tenants" />
      <ComponentCard
        title="Tenant management"
        desc={
          sysAdmin ? "Manage all tenants." : "Manage your tenant information."
        }
      >
        <div className="mb-5 gap-3 flex justify-end">
          <Button
            variant="outline"
            startIcon={<RefreshCw size={16} />}
            onClick={() => dispatch(fetchTenants({ page, pageSize, q, sortBy, sortDir }))}
          >
            Refresh
          </Button>
          {sysAdmin && (
            <Link to="/tenants/new">
              <Button startIcon={<Plus size={16} />}>Add tenant</Button>
            </Link>
          )}
        </div>
        <DataGrid
          data={tenantsList}
          columns={columns}
          getRowId={(row, index) => String(row.id ?? row.uuid ?? index)}
          loading={loading && tenantsList.length === 0}
          error={error}
          emptyMessage="No tenants found."
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
