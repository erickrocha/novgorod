import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Button from "@/components/ui/button/Button";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { createTenant, fetchTenants, updateTenant } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import type { TenantInput } from "@/services/types";

export default function TenantForm() {
  const { id } = useParams(); const navigate = useNavigate(); const dispatch = useAppDispatch(); const editing = Boolean(id); const { tenantsList, loading, error } = useAppSelector((s) => s.tenant); const { user } = useAppSelector((s) => s.auth); const sysAdmin = user?.role === ROLES.SYS_ADMIN; const ownId = user?.tenantId ?? user?.tenant_id; const existing = tenantsList.find((t) => String(t.id) === id);
  const [form, setForm] = useState({ name: "", businessName: "", companyName: "", taxId: "", email: "", phone: "", website: "", addressLine1: "", addressLine2: "", locality: "", administrativeArea: "", postalCode: "", countryCode: "BR" });
  useEffect(() => { dispatch(fetchTenants()); }, [dispatch]);
  // Form state is synchronized when the asynchronously loaded record becomes available.
  // eslint-disable-next-line react-hooks/set-state-in-effect
  useEffect(() => { if (existing) setForm({ name: "", businessName: existing.businessName || "", companyName: existing.companyName || "", taxId: existing.taxId || "", email: existing.email || "", phone: existing.phone || "", website: existing.website || "", addressLine1: existing.addressLine1 || "", addressLine2: existing.addressLine2 || "", locality: existing.locality || existing.city || "", administrativeArea: existing.administrativeArea || existing.province || "", postalCode: existing.postalCode || existing.zipcode || "", countryCode: existing.countryCode || "BR" }); }, [existing]);
  const set = (key: keyof typeof form, value: string) => setForm((current) => ({ ...current, [key]: value }));
  const submit = async (e: FormEvent) => { e.preventDefault(); const payload = Object.fromEntries(Object.entries(form).map(([k, v]) => [k, v.trim() || null])) as TenantInput; const result = editing && existing?.id ? await dispatch(updateTenant({ id: existing.id, tenantData: payload })) : await dispatch(createTenant(payload)); if (result.meta.requestStatus === "fulfilled") { await dispatch(fetchTenants()); navigate("/tenants"); } };
  const fields: [keyof typeof form, string, string?][] = [["name", "Name"], ["businessName", "Business name"], ["companyName", "Company name"], ["taxId", "Tax ID"], ["email", "Email", "email"], ["phone", "Phone"], ["website", "Website"], ["addressLine1", "Address"], ["addressLine2", "Address line 2"], ["locality", "City / locality"], ["administrativeArea", "State / province"], ["postalCode", "Postal code"], ["countryCode", "Country code"]];
  if (!sysAdmin && (!editing || !existing || existing.id !== ownId)) return <div className="rounded-lg border border-error-200 bg-error-50 p-4 text-error-600">You do not have access to this tenant.</div>;
  return <><PageMeta title={`${editing ? "Edit" : "Add"} tenant | Veche`} description="Tenant form"/><PageBreadcrumb pageTitle={editing ? "Edit tenant" : "Add tenant"}/><ComponentCard title={editing ? "Edit tenant" : "Add tenant"}><form onSubmit={submit} className="space-y-5"><div className="grid gap-5 md:grid-cols-2">{fields.map(([key, label, type]) => <div key={key}><Label htmlFor={key}>{label}</Label><Input id={key} type={type || "text"} required={key === "name" || key === "businessName"} disabled={!sysAdmin && key === "countryCode"} value={form[key]} onChange={(e) => set(key, e.target.value)}/></div>)}</div>{error && <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">{error}</div>}<div className="flex justify-end gap-3"><Link to="/tenants"><Button variant="outline">Cancel</Button></Link><Button disabled={loading}>{editing ? "Save changes" : "Create tenant"}</Button></div></form></ComponentCard></>;
}
