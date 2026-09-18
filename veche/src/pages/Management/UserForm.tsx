import { useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Select from "@/components/form/Select";
import Button from "@/components/ui/button/Button";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { createUser, fetchUsers, updateUser } from "@/store/userSlice";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import type { Role, UserInput } from "@/services/types";

export default function UserForm() {
  const { id } = useParams(); const navigate = useNavigate(); const dispatch = useAppDispatch();
  const editing = Boolean(id); const { usersList, loading, error } = useAppSelector((s) => s.user); const { tenantsList } = useAppSelector((s) => s.tenant); const { user } = useAppSelector((s) => s.auth);
  const sysAdmin = user?.role === ROLES.SYS_ADMIN; const ownTenant = user?.tenantId ?? user?.tenant_id; const existing = usersList.find((u) => String(u.id) === id);
  const [form, setForm] = useState({ name: "", email: "", password: "", role: ROLES.TENANT_USER as Role, tenantId: ownTenant ? String(ownTenant) : "", enabled: true, firstLogin: true });
  useEffect(() => { dispatch(fetchUsers()); if (sysAdmin) dispatch(fetchTenants()); }, [dispatch, sysAdmin]);
  // Form state is synchronized when the asynchronously loaded record becomes available.
  // eslint-disable-next-line react-hooks/set-state-in-effect
  useEffect(() => { if (existing) setForm({ name: existing.name || "", email: existing.email, password: "", role: existing.role, tenantId: existing.tenantId ? String(existing.tenantId) : "", enabled: existing.enabled, firstLogin: existing.firstLogin }); }, [existing]);
  const roleOptions = useMemo(() => [ROLES.TENANT_USER, ROLES.TENANT_OWNER, ...(sysAdmin ? [ROLES.SYS_ADMIN] : [])].map((r) => ({ value: r, label: r })), [sysAdmin]);
  const tenantOptions = tenantsList.filter((t) => t.id != null).map((t) => ({ value: String(t.id), label: t.businessName || t.companyName || `Tenant #${t.id}` }));
  const submit = async (e: FormEvent) => { e.preventDefault(); const payload: UserInput = { name: form.name || undefined, email: form.email, role: sysAdmin ? form.role : (existing?.role || ROLES.TENANT_USER), tenantId: sysAdmin ? (form.tenantId ? Number(form.tenantId) : null) : ownTenant, enabled: form.enabled, firstLogin: form.firstLogin, ...(form.password ? { password: form.password } : {}) }; const result = editing && existing?.id ? await dispatch(updateUser({ id: existing.id, userData: payload })) : await dispatch(createUser(payload)); if (result.meta.requestStatus === "fulfilled") { await dispatch(fetchUsers()); navigate("/users"); } };
  if (editing && existing && !sysAdmin && existing.tenantId !== ownTenant) return <div className="rounded-lg border border-error-200 bg-error-50 p-4 text-error-600">You do not have access to this user.</div>;
  return <><PageMeta title={`${editing ? "Edit" : "Add"} user | Veche`} description="User form"/><PageBreadcrumb pageTitle={editing ? "Edit user" : "Add user"}/><ComponentCard title={editing ? "Edit user" : "Add user"}><form onSubmit={submit} className="space-y-5"><div className="grid gap-5 md:grid-cols-2"><div><Label htmlFor="name">Name</Label><Input id="name" value={form.name} onChange={(e) => setForm({...form,name:e.target.value})}/></div><div><Label htmlFor="email">Email</Label><Input id="email" type="email" required value={form.email} onChange={(e) => setForm({...form,email:e.target.value})}/></div><div><Label htmlFor="password">Password {editing && "(leave blank to keep)"}</Label><Input id="password" type="password" required={!editing} value={form.password} onChange={(e) => setForm({...form,password:e.target.value})}/></div><div><Label>Role</Label><Select options={roleOptions} defaultValue={form.role} onChange={(v) => setForm({...form,role:v as Role})} className={!sysAdmin ? "pointer-events-none opacity-60" : ""}/></div><div><Label>Tenant</Label><Select options={tenantOptions} defaultValue={form.tenantId} onChange={(v) => setForm({...form,tenantId:v})} className={!sysAdmin ? "pointer-events-none opacity-60" : ""}/></div></div><label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300"><input type="checkbox" checked={form.enabled} onChange={(e) => setForm({...form,enabled:e.target.checked})}/> Enabled</label>{error && <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">{error}</div>}<div className="flex justify-end gap-3"><Link to="/users"><Button variant="outline">Cancel</Button></Link><Button disabled={loading}>{editing ? "Save changes" : "Create user"}</Button></div></form></ComponentCard></>;
}
