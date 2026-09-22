import { useEffect } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Pencil, Plus, RefreshCw } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
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

export function Users() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const { usersList, total, loading, error } = useAppSelector((s) => s.user);
  const { tenantsList } = useAppSelector((s) => s.tenant);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  useEffect(() => {
    dispatch(fetchUsers({ page, pageSize, q, sortBy, sortDir }));
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
  const tenantName = (id?: number | null) => {
    const t = tenantsList.find((x) => x.id === id);
    return t?.businessName || t?.companyName || `#${id ?? "—"}`;
  };
  const columns: ColumnDef<User, unknown>[] = [
    {
      header: t("users.name", "Nome"),
      accessorKey: "name",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-medium">{getValue<string>() || "—"}</span>
      ),
    },
    { header: t("users.email", "E-mail"), accessorKey: "email", enableSorting: true },
    {
      header: t("users.role", "Função / Perfil"),
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
      header: t("users.tenant", "Empresa"),
      id: "tenant",
      cell: ({ row }) => tenantName(row.original.tenantId),
    },
    {
      header: t("users.enabled", "Status"),
      accessorKey: "enabled",
      cell: ({ row }) => (
        <Badge size="sm" color={row.original.enabled ? "success" : "light"}>
          {row.original.enabled ? t("users.active", "Ativo") : t("users.disabled", "Inativo")}
        </Badge>
      ),
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      enableHiding: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end text-end">
          <Link
            title={t("users.editUser", "Editar usuário")}
            aria-label={t("users.editUser", "Editar usuário")}
            className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            to={`/users/${row.original.id}/edit`}
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
        title={`${t("users.title", "Usuários")} | Veche`}
        description={t("users.manageUsers", "Gerenciar usuários")}
      />
      <PageBreadcrumb pageTitle={t("users.title", "Usuários")} />
      <ComponentCard>
        <DataGrid
          data={usersList}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={() => dispatch(fetchUsers({ page, pageSize, q, sortBy, sortDir }))}
              >
                {t("users.refresh", "Atualizar")}
              </Button>
              <Link to="/users/new">
                <Button size="sm" startIcon={<Plus size={14} />}>
                  {t("users.addUser", "Novo usuário")}
                </Button>
              </Link>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? row.uuid ?? index)}
          loading={loading && usersList.length === 0}
          error={error}
          emptyMessage={t("users.noUsersFound", "Nenhum usuário encontrado.")}
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

export default Users;
