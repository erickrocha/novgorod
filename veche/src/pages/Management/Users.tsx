import { useEffect, useMemo } from "react";
import { Link } from "react-router-dom";
import { Plus, RefreshCw } from "lucide-react";
import type { ColumnDef } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchUsers } from "@/store/userSlice";
import { ROLES } from "@/utils/enums";
import type { User } from "@/services/types";

export default function Users() {
  const dispatch = useAppDispatch();
  const { usersList, loading, error } = useAppSelector((s) => s.user);
  const { user } = useAppSelector((s) => s.auth);
  const { tenantsList } = useAppSelector((s) => s.tenant);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;
  const tenantId = user?.tenantId ?? user?.tenant_id;
  const users = useMemo(
    () =>
      isSysAdmin ? usersList : usersList.filter((u) => u.tenantId === tenantId),
    [isSysAdmin, tenantId, usersList],
  );
  useEffect(() => {
    dispatch(fetchUsers());
  }, [dispatch]);
  const tenantName = (id?: number | null) => {
    const t = tenantsList.find((x) => x.id === id);
    return t?.businessName || t?.companyName || `#${id ?? "—"}`;
  };
  const columns: ColumnDef<User, unknown>[] = [
    {
      header: "Name",
      accessorKey: "name",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-medium">{getValue<string>() || "—"}</span>
      ),
    },
    { header: "Email", accessorKey: "email", enableSorting: true },
    {
      header: "Role",
      accessorKey: "role",
      enableSorting: true,
      cell: ({ row }) => (
        <Badge
          size="sm"
          color={
            row.original.role === ROLES.SYS_ADMIN
              ? "error"
              : row.original.role === ROLES.TENANT_OWNER
                ? "warning"
                : "info"
          }
        >
          {row.original.role}
        </Badge>
      ),
    },
    {
      header: "Tenant",
      id: "tenant",
      cell: ({ row }) => tenantName(row.original.tenantId),
    },
    {
      header: "Status",
      accessorKey: "enabled",
      cell: ({ row }) => (
        <Badge size="sm" color={row.original.enabled ? "success" : "light"}>
          {row.original.enabled ? "Enabled" : "Disabled"}
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
          to={`/users/${row.original.id}/edit`}
        >
          Edit
        </Link>
      ),
    },
  ];
  return (
    <>
      <PageMeta title="Users | Veche" description="Manage users" />
      <PageBreadcrumb pageTitle="Users" />
      <ComponentCard
        title="User management"
        desc={
          isSysAdmin
            ? "Manage users across all tenants."
            : "Manage users in your tenant."
        }
      >
        <div className="mb-5 gap-3 flex flex-wrap justify-end">
          <Button
            variant="outline"
            startIcon={<RefreshCw size={16} />}
            onClick={() => dispatch(fetchUsers())}
          >
            Refresh
          </Button>
          <Link to="/users/new">
            <Button startIcon={<Plus size={16} />}>Add user</Button>
          </Link>
        </div>
        <DataGrid
          data={users}
          columns={columns}
          getRowId={(row, index) => String(row.id ?? row.uuid ?? index)}
          loading={loading && users.length === 0}
          error={error}
          emptyMessage="No users found."
        />
      </ComponentCard>
    </>
  );
}
