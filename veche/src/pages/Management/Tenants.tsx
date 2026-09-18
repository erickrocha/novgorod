import { useEffect } from "react";
import { Link } from "react-router-dom";
import { Building2, Pencil, Plus, RefreshCw } from "lucide-react";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";

export default function Tenants() {
  const dispatch = useAppDispatch(); const { tenantsList, loading, error } = useAppSelector((s) => s.tenant); const { user } = useAppSelector((s) => s.auth); const sysAdmin = user?.role === ROLES.SYS_ADMIN; const ownId = user?.tenantId ?? user?.tenant_id;
  useEffect(() => { dispatch(fetchTenants()); }, [dispatch]);
  const tenants = sysAdmin ? tenantsList : tenantsList.filter((t) => t.id === ownId);
  return <><PageMeta title="Tenants | Veche" description="Manage tenants"/><PageBreadcrumb pageTitle="Tenants"/><ComponentCard title="Tenant management" desc={sysAdmin ? "Manage all tenants." : "Manage your tenant information."}><div className="mb-5 flex justify-end gap-3"><Button variant="outline" startIcon={<RefreshCw size={16}/>} onClick={() => dispatch(fetchTenants())}>Refresh</Button>{sysAdmin && <Link to="/tenants/new"><Button startIcon={<Plus size={16}/>}>Add tenant</Button></Link>}</div>{error && <div className="mb-4 rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">{error}</div>}{loading && tenants.length === 0 ? <div className="py-12 text-center text-gray-500">Loading tenants…</div> : tenants.length === 0 ? <div className="py-12 text-center text-gray-500"><Building2 className="mx-auto mb-3"/>No tenant found.</div> : <div className="overflow-x-auto"><table className="w-full min-w-[680px] text-left text-sm"><thead><tr className="border-b border-gray-100 dark:border-gray-800"><th className="px-4 py-3 font-medium text-gray-500">Business name</th><th className="px-4 py-3 font-medium text-gray-500">Email</th><th className="px-4 py-3 font-medium text-gray-500">Phone</th><th className="px-4 py-3 font-medium text-gray-500">Location</th><th className="px-4 py-3"/></tr></thead><tbody>{tenants.map((t) => <tr key={t.id ?? t.uuid} className="border-b border-gray-100 dark:border-gray-800"><td className="px-4 py-4 font-medium text-gray-800 dark:text-white/90">{t.businessName || t.companyName || "—"}</td><td className="px-4 py-4 text-gray-500">{t.email || "—"}</td><td className="px-4 py-4 text-gray-500">{t.phone || "—"}</td><td className="px-4 py-4 text-gray-500">{[t.locality || t.city, t.administrativeArea || t.province].filter(Boolean).join(", ") || "—"}</td><td className="px-4 py-4 text-right"><Link className="inline-flex items-center gap-1 text-brand-500 hover:text-brand-600" to={`/tenants/${t.id}/edit`}><Pencil size={15}/>Edit</Link></td></tr>)}</tbody></table></div>}</ComponentCard></>;
}
