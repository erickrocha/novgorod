import { useEffect } from "react";
import { Link } from "react-router-dom";
import { Pencil, Plus, RefreshCw, Users as UsersIcon } from "lucide-react";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchUsers } from "@/store/userSlice";
import { ROLES } from "@/utils/enums";

export default function Users() {
  const dispatch = useAppDispatch();
  const { usersList, loading, error } = useAppSelector((s) => s.user);
  const { user } = useAppSelector((s) => s.auth);
  const { tenantsList } = useAppSelector((s) => s.tenant);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;
  const tenantId = user?.tenantId ?? user?.tenant_id;
  const users = isSysAdmin ? usersList : usersList.filter((u) => u.tenantId === tenantId);
  useEffect(() => { dispatch(fetchUsers()); }, [dispatch]);
  const tenantName = (id?: number | null) => { const t = tenantsList.find((x) => x.id === id); return t?.businessName || t?.companyName || `#${id ?? "—"}`; };
  return <><PageMeta title="Users | Veche" description="Manage users"/><PageBreadcrumb pageTitle="Users"/><ComponentCard title="User management" desc={isSysAdmin ? "Manage users across all tenants." : "Manage users in your tenant."}><div className="mb-5 flex flex-wrap justify-end gap-3"><Button variant="outline" startIcon={<RefreshCw size={16}/>} onClick={() => dispatch(fetchUsers())}>Refresh</Button><Link to="/users/new"><Button startIcon={<Plus size={16}/>}>Add user</Button></Link></div>{error && <div className="mb-4 rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">{error}</div>}{loading && users.length === 0 ? <div className="py-12 text-center text-gray-500">Loading users…</div> : users.length === 0 ? <div className="py-12 text-center text-gray-500"><UsersIcon className="mx-auto mb-3"/>No users found.</div> : <div className="overflow-x-auto"><table className="w-full min-w-[720px] text-left text-sm"><thead><tr className="border-b border-gray-100 dark:border-gray-800"><th className="px-4 py-3 font-medium text-gray-500">Name</th><th className="px-4 py-3 font-medium text-gray-500">Email</th><th className="px-4 py-3 font-medium text-gray-500">Role</th><th className="px-4 py-3 font-medium text-gray-500">Tenant</th><th className="px-4 py-3 font-medium text-gray-500">Status</th><th className="px-4 py-3"/></tr></thead><tbody>{users.map((u) => <tr key={u.id ?? u.uuid} className="border-b border-gray-100 dark:border-gray-800"><td className="px-4 py-4 font-medium text-gray-800 dark:text-white/90">{u.name || "—"}</td><td className="px-4 py-4 text-gray-500">{u.email}</td><td className="px-4 py-4"><Badge size="sm" color={u.role === ROLES.SYS_ADMIN ? "error" : u.role === ROLES.TENANT_OWNER ? "warning" : "info"}>{u.role}</Badge></td><td className="px-4 py-4 text-gray-500">{tenantName(u.tenantId)}</td><td className="px-4 py-4"><Badge size="sm" color={u.enabled ? "success" : "light"}>{u.enabled ? "Enabled" : "Disabled"}</Badge></td><td className="px-4 py-4 text-right"><Link className="inline-flex items-center gap-1 text-brand-500 hover:text-brand-600" to={`/users/${u.id}/edit`}><Pencil size={15}/>Edit</Link></td></tr>)}</tbody></table></div>}</ComponentCard></>;
}
