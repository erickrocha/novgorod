import { useEffect, useMemo } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useForm } from "react-hook-form";
import { yupResolver } from "@hookform/resolvers/yup";
import * as yup from "yup";
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

const userSchema = yup.object({
  name: yup.string().nullable().defined(),
  email: yup.string().email("E-mail inválido").required("E-mail é obrigatório"),
  password: yup.string().defined().default(""),
  role: yup.string().required("Função é obrigatória"),
  tenantId: yup.string().defined().default(""),
  enabled: yup.boolean().defined().default(true),
  firstLogin: yup.boolean().defined().default(true),
});

type UserFormData = yup.InferType<typeof userSchema>;

export function UserForm() {
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

  const {
    handleSubmit,
    setValue,
    watch,
    reset,
    formState: { errors },
  } = useForm<UserFormData>({
    resolver: yupResolver(userSchema) as any,
    defaultValues: {
      name: "",
      email: "",
      password: "",
      role: ROLES.TENANT_USER,
      tenantId: ownTenant ? String(ownTenant) : "",
      enabled: true,
      firstLogin: true,
    },
  });

  const formValues = watch();

  useEffect(() => {
    dispatch(fetchUsers());
    dispatch(fetchTenants());
  }, [dispatch]);

  useEffect(() => {
    if (existing) {
      reset({
        name: existing.name || "",
        email: existing.email,
        password: "",
        role: existing.role,
        tenantId: existing.tenantId ? String(existing.tenantId) : "",
        enabled: existing.enabled ?? true,
        firstLogin: existing.firstLogin ?? true,
      });
    }
  }, [existing, reset]);

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
    if (formValues.role && !roles.some((r) => r.value === formValues.role)) {
      roles.push({
        value: formValues.role,
        label: formValues.role,
      });
    }
    return roles;
  }, [sysAdmin, formValues.role, t]);

  const tenantOptions: ComboboxOption[] = useMemo(() => {
    const list = tenantsList
      .filter((t) => t.id != null)
      .map((t) => ({
        value: String(t.id),
        label: t.businessName || t.companyName || `Tenant #${t.id}`,
      }));

    if (formValues.tenantId && !list.some((o) => o.value === formValues.tenantId)) {
      list.unshift({
        value: formValues.tenantId,
        label: `Tenant #${formValues.tenantId}`,
      });
    }
    return list;
  }, [tenantsList, formValues.tenantId]);

  const onSubmit = async (data: UserFormData) => {
    const payload: UserInput = {
      name: data.name || undefined,
      email: data.email,
      role: sysAdmin ? (data.role as Role) : (existing?.role || ROLES.TENANT_USER),
      tenantId: sysAdmin
        ? (data.tenantId ? Number(data.tenantId) : null)
        : (ownTenant ?? null),
      enabled: data.enabled,
      firstLogin: data.firstLogin,
      ...(data.password ? { password: data.password } : {}),
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
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
          {/* First row: combo box to select tenant width 100%, use filterable combo box */}
          <div>
            <Label htmlFor="tenant">{t("users.tenant", "Empresa")}</Label>
            <FilterableCombobox
              id="tenant"
              options={tenantOptions}
              value={formValues.tenantId}
              onChange={(v) => setValue("tenantId", v)}
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
                value={formValues.name || ""}
                onChange={(e) => setValue("name", e.target.value)}
                placeholder={t("users.namePlaceholder", "Nome completo")}
              />
            </div>
            <div className="md:col-span-3">
              <Label htmlFor="role">{t("users.role", "Função / Perfil")}</Label>
              <FilterableCombobox
                id="role"
                options={roleOptions}
                value={formValues.role}
                onChange={(v) => setValue("role", v as Role, { shouldValidate: true })}
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
                value={formValues.email}
                onChange={(e) => setValue("email", e.target.value, { shouldValidate: true })}
                error={Boolean(errors.email)}
                hint={errors.email?.message}
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
                value={formValues.password}
                onChange={(e) => setValue("password", e.target.value)}
                placeholder={t("users.passwordPlaceholder", "Digite a senha")}
              />
            </div>
            <div className="md:col-span-2">
              <Label htmlFor="enabled">{t("users.enabled", "Status")}</Label>
              <div className="h-11 flex items-center">
                <Switch
                  id="enabled"
                  checked={formValues.enabled}
                  onChange={(checked) => setValue("enabled", checked)}
                  label={
                    formValues.enabled
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

export default UserForm;
