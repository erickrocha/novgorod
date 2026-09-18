import { useEffect, useMemo } from "react";
import { Link } from "react-router-dom";
import { Plus, RefreshCw } from "lucide-react";
import type { ColumnDef } from "@tanstack/react-table";
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
  const dispatch = useAppDispatch();
  const { tenantsList, loading, error } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);
  const sysAdmin = user?.role === ROLES.SYS_ADMIN;
  const ownId = user?.tenantId ?? user?.tenant_id;
  const tenants = useMemo(
    () => (sysAdmin ? tenantsList : tenantsList.filter((t) => t.id === ownId)),
    [ownId, sysAdmin, tenantsList],
  );
  useEffect(() => {
    dispatch(fetchTenants());
  }, [dispatch]);
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
            onClick={() => dispatch(fetchTenants())}
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
          data={tenants}
          columns={columns}
          getRowId={(row, index) => String(row.id ?? row.uuid ?? index)}
          loading={loading && tenants.length === 0}
          error={error}
          emptyMessage="No tenants found."
        />
      </ComponentCard>
    </>
  );
}
