import { useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import Switch from "@/components/form/switch/Switch";
import Button from "@/components/ui/button/Button";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { createUser, fetchUsers, updateUser } from "@/store/userSlice";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";
import type { Role, UserInput } from "@/services/types";

export default function UserForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();

  const editing = Boolean(id);
  const { usersList, loading, error } = useAppSelector((s) => s.user);
  const { tenantsList } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);

  const sysAdmin = user?.role === ROLES.SYS_ADMIN;
  const ownTenant = user?.tenantId ?? user?.tenant_id;
  const existing = usersList.find((u) => String(u.id) === id);

  const [form, setForm] = useState({
    name: "",
    email: "",
    password: "",
    role: ROLES.TENANT_USER as Role,
    tenantId: ownTenant ? String(ownTenant) : "",
    enabled: true,
    firstLogin: true,
  });

  useEffect(() => {
    dispatch(fetchUsers());
    dispatch(fetchTenants());
  }, [dispatch]);

  // Form state is synchronized when the asynchronously loaded record becomes available.
  // eslint-disable-next-line react-hooks/set-state-in-effect
  useEffect(() => {
    if (existing) {
      setForm({
        name: existing.name || "",
        email: existing.email,
        password: "",
        role: existing.role,
        tenantId: existing.tenantId ? String(existing.tenantId) : "",
        enabled: existing.enabled ?? true,
        firstLogin: existing.firstLogin ?? true,
      });
    }
  }, [existing]);

  const roleOptions: ComboboxOption[] = useMemo(() => {
    const roles: ComboboxOption[] = [
      {
        value: ROLES.TENANT_USER,
        label: t("users.roles.tenantUser", "Usuário da Empresa"),
      },
      {
        value: ROLES.TENANT_OWNER,
        label: t("users.roles.tenantOwner", "Administrador da Empresa"),
      },
      ...(sysAdmin
        ? [
            {
              value: ROLES.SYS_ADMIN,
              label: t("users.roles.sysAdmin", "SysAdmin"),
            },
          ]
        : []),
    ];
    if (form.role && !roles.some((r) => r.value === form.role)) {
      roles.push({
        value: form.role,
        label: form.role,
      });
    }
    return roles;
  }, [sysAdmin, form.role, t]);

  const tenantOptions: ComboboxOption[] = useMemo(() => {
    const list = tenantsList
      .filter((t) => t.id != null)
      .map((t) => ({
        value: String(t.id),
        label: t.businessName || t.companyName || `Tenant #${t.id}`,
      }));

    if (form.tenantId && !list.some((o) => o.value === form.tenantId)) {
      list.unshift({
        value: form.tenantId,
        label: `Tenant #${form.tenantId}`,
      });
    }
    return list;
  }, [tenantsList, form.tenantId]);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const payload: UserInput = {
      name: form.name || undefined,
      email: form.email,
      role: sysAdmin ? form.role : (existing?.role || ROLES.TENANT_USER),
      tenantId: sysAdmin
        ? (form.tenantId ? Number(form.tenantId) : null)
        : (ownTenant ?? null),
      enabled: form.enabled,
      firstLogin: form.firstLogin,
      ...(form.password ? { password: form.password } : {}),
    };

    const result =
      editing && existing?.id
        ? await dispatch(updateUser({ id: existing.id, userData: payload }))
        : await dispatch(createUser(payload));

    if (result.meta.requestStatus === "fulfilled") {
      await dispatch(fetchUsers());
      navigate("/users");
    }
  };

  const pageTitle = editing
    ? t("users.editUser", "Editar usuário")
    : t("users.addUser", "Novo usuário");

  if (editing && existing && !sysAdmin && existing.tenantId !== ownTenant) {
    return (
      <div className="rounded-lg border border-error-200 bg-error-50 p-4 text-error-600">
        {t("users.noAccess", "Você não tem acesso a este usuário.")}
      </div>
    );
  }

  return (
    <>
      <PageMeta
        title={`${pageTitle} | Veche`}
        description={t("users.userForm", "Formulário de usuário")}
      />
      <PageBreadcrumb pageTitle={pageTitle} />
      <ComponentCard title={pageTitle}>
        <form onSubmit={submit} className="space-y-6">
          {/* First row: combo box to select tenant width 100%, use filterable combo box */}
          <div>
            <Label htmlFor="tenant">{t("users.tenant", "Empresa")}</Label>
            <FilterableCombobox
              id="tenant"
              options={tenantOptions}
              value={form.tenantId}
              onChange={(v) => setForm({ ...form, tenantId: v })}
              placeholder={t("users.selectTenant", "Selecione a empresa...")}
              disabled={!sysAdmin}
              emptyText={t("users.noTenantFound", "Nenhuma empresa encontrada")}
              className="w-full"
            />
          </div>

          {/* Second row: Name width 70%, role combo box width 30% */}
          <div className="grid grid-cols-1 md:grid-cols-10 gap-5">
            <div className="md:col-span-7">
              <Label htmlFor="name">{t("users.name", "Nome")}</Label>
              <Input
                id="name"
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder={t("users.namePlaceholder", "Nome completo")}
              />
            </div>
            <div className="md:col-span-3">
              <Label htmlFor="role">{t("users.role", "Função / Perfil")}</Label>
              <FilterableCombobox
                id="role"
                options={roleOptions}
                value={form.role}
                onChange={(v) => setForm({ ...form, role: v as Role })}
                placeholder={t("users.selectRole", "Selecione a função...")}
                disabled={!sysAdmin}
                className="w-full"
              />
            </div>
          </div>

          {/* Third row: email width 40%, password width 40%, enabled use slider width 20% */}
          <div className="grid grid-cols-1 md:grid-cols-10 gap-5 items-start">
            <div className="md:col-span-4">
              <Label htmlFor="email">{t("users.email", "E-mail")}</Label>
              <Input
                id="email"
                type="email"
                required
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
                placeholder={t("users.emailPlaceholder", "nome@exemplo.com")}
              />
            </div>
            <div className="md:col-span-4">
              <Label htmlFor="password">
                {t("users.password", "Senha")}{" "}
                {editing && (
                  <span className="text-xs font-normal text-gray-500 dark:text-gray-400">
                    {t("users.passwordHint", "(deixe em branco para manter)")}
                  </span>
                )}
              </Label>
              <Input
                id="password"
                type="password"
                required={!editing}
                value={form.password}
                onChange={(e) => setForm({ ...form, password: e.target.value })}
                placeholder={t("users.passwordPlaceholder", "Digite a senha")}
              />
            </div>
            <div className="md:col-span-2">
              <Label htmlFor="enabled">{t("users.enabled", "Status")}</Label>
              <div className="h-11 flex items-center">
                <Switch
                  id="enabled"
                  checked={form.enabled}
                  onChange={(checked) => setForm({ ...form, enabled: checked })}
                  label={
                    form.enabled
                      ? t("users.active", "Ativo")
                      : t("users.inactive", "Inativo")
                  }
                />
              </div>
            </div>
          </div>

          {error && (
            <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">
              {error}
            </div>
          )}

          {/* Cancel button aligned to the left, save aligned to the right */}
          <div className="flex items-center justify-between pt-3">
            <Link to="/users">
              <Button variant="outline" type="button">
                {t("users.cancel", "Cancelar")}
              </Button>
            </Link>
            <Button disabled={loading} type="submit">
              {editing
                ? t("users.saveChanges", "Salvar alterações")
                : t("users.createUser", "Criar usuário")}
            </Button>
          </div>
        </form>
      </ComponentCard>
    </>
  );
}
