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
import Switch from "@/components/form/switch/Switch";
import Button from "@/components/ui/button/Button";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import {
  createCustomer,
  fetchCustomerById,
  updateCustomer,
} from "@/store/customerSlice";
import { ROLES } from "@/utils/enums";
import { formatCpf, formatPhone, stripNonDigits } from "@/utils/taxId";
import type { CustomerInput } from "@/services/types";

const customerSchema = yup.object({
  name: yup.string().required("Nome é obrigatório"),
  email: yup.string().email("E-mail inválido").required("E-mail é obrigatório"),
  password: yup.string().defined().default(""),
  cpf: yup.string().nullable().defined(),
  phone: yup.string().nullable().defined(),
  active: yup.boolean().defined().default(true),
  marketingConsent: yup.boolean().defined().default(false),
  tenantId: yup.string().defined().default(""),
});

type CustomerFormData = yup.InferType<typeof customerSchema>;

export function CustomerForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const editing = Boolean(id);

  const { tenantsList } = useAppSelector((s) => s.tenant);
  const { currentCustomer, loading, error } = useAppSelector((s) => s.customer);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const pageTitle = editing
    ? t("customers.editCustomer", "Editar Cliente")
    : t("customers.addCustomer", "Novo Cliente");

  const {
    handleSubmit,
    setValue,
    watch,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<CustomerFormData>({
    resolver: yupResolver(customerSchema) as any,
    defaultValues: {
      name: "",
      email: "",
      password: "",
      cpf: "",
      phone: "",
      active: true,
      marketingConsent: false,
      tenantId: user?.tenantId ? String(user.tenantId) : "",
    },
  });

  const formValues = watch();

  useEffect(() => {
    if (isSysAdmin && tenantsList.length === 0) {
      dispatch(fetchTenants());
    }
  }, [dispatch, isSysAdmin, tenantsList.length]);

  useEffect(() => {
    if (editing && id) {
      dispatch(fetchCustomerById(Number(id)));
    }
  }, [dispatch, editing, id]);

  useEffect(() => {
    if (editing && currentCustomer && String(currentCustomer.id) === id) {
      reset({
        name: currentCustomer.name || "",
        email: currentCustomer.email || "",
        password: "",
        cpf: currentCustomer.cpf ? formatCpf(currentCustomer.cpf) : "",
        phone: currentCustomer.phone ? formatPhone(currentCustomer.phone) : "",
        active: currentCustomer.active ?? true,
        marketingConsent: currentCustomer.marketingConsent ?? false,
        tenantId: currentCustomer.tenantId ? String(currentCustomer.tenantId) : "",
      });
    }
  }, [editing, currentCustomer, id, reset]);

  const tenantOptions: ComboboxOption[] = useMemo(() => {
    const list = tenantsList
      .filter((tn) => tn.id != null)
      .map((tn) => ({
        value: String(tn.id),
        label: tn.businessName || tn.companyName || `Tenant #${tn.id}`,
      }));

    if (formValues.tenantId && !list.some((o) => o.value === formValues.tenantId)) {
      list.unshift({
        value: formValues.tenantId,
        label: `Tenant #${formValues.tenantId}`,
      });
    }
    return list;
  }, [tenantsList, formValues.tenantId]);

  const handleCpfChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setValue("cpf", formatCpf(e.target.value));
  };

  const handlePhoneChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setValue("phone", formatPhone(e.target.value));
  };

  const onSubmit = async (data: CustomerFormData) => {
    const payload: CustomerInput = {
      name: data.name.trim(),
      email: data.email.trim(),
      cpf: data.cpf ? stripNonDigits(data.cpf) || null : null,
      phone: data.phone ? stripNonDigits(data.phone) || null : null,
      active: data.active,
      marketingConsent: data.marketingConsent,
      tenantId: isSysAdmin
        ? (data.tenantId ? Number(data.tenantId) : null)
        : (user?.tenantId ?? null),
      ...(data.password ? { password: data.password } : {}),
    };

    const action = editing && id
      ? await dispatch(updateCustomer({ id: Number(id), data: payload }))
      : await dispatch(createCustomer(payload));

    if (action.meta.requestStatus === "fulfilled") {
      navigate("/customers");
    }
  };

  return (
    <>
      <PageMeta
        title={`${pageTitle} | Veche`}
        description={t("customers.customerForm", "Formulário de Cliente")}
      />
      <PageBreadcrumb
        pageTitle={pageTitle}
        items={[{ label: t("customers.title", "Clientes"), href: "/customers" }]}
      />
      <ComponentCard title={pageTitle}>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
          {error && (
            <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600 dark:bg-error-500/10 dark:border-error-500/20">
              {error}
            </div>
          )}

          {/* Row 1: Full name (100%) */}
          <div>
            <Label htmlFor="customerName">
              {t("customers.fullName", "Nome Completo")}
            </Label>
            <Input
              id="customerName"
              disabled={loading}
              value={formValues.name}
              onChange={(e) => setValue("name", e.target.value, { shouldValidate: true })}
              error={Boolean(errors.name)}
              hint={errors.name?.message}
              placeholder={t("customers.fullNamePlaceholder", "Maria Silva")}
            />
          </div>

          {/* Row 2: Email (50%) and Password (50%) */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="customerEmail">
                {t("customers.email", "E-mail")}
              </Label>
              <Input
                id="customerEmail"
                type="email"
                disabled={loading}
                value={formValues.email}
                onChange={(e) => setValue("email", e.target.value, { shouldValidate: true })}
                error={Boolean(errors.email)}
                hint={errors.email?.message}
                placeholder={t("customers.emailPlaceholder", "maria@exemplo.com")}
              />
            </div>
            <div>
              <Label htmlFor="customerPassword">
                {editing
                  ? t("customers.newPasswordOptional", "Nova Senha (Opcional)")
                  : t("customers.password", "Senha")}
              </Label>
              <Input
                id="customerPassword"
                type="password"
                disabled={loading}
                value={formValues.password}
                onChange={(e) => setValue("password", e.target.value)}
                placeholder={t("customers.passwordPlaceholder", "••••••••")}
              />
            </div>
          </div>

          {/* Row 3: CPF (50%) and Phone (50%) */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="customerCpf">
                {t("customers.cpf", "CPF")}
              </Label>
              <Input
                id="customerCpf"
                disabled={loading}
                value={formValues.cpf || ""}
                onChange={handleCpfChange}
                maxLength={14}
                placeholder={t("customers.cpfPlaceholder", "000.000.000-00")}
              />
            </div>
            <div>
              <Label htmlFor="customerPhone">
                {t("customers.phone", "Telefone")}
              </Label>
              <Input
                id="customerPhone"
                type="tel"
                disabled={loading}
                value={formValues.phone || ""}
                onChange={handlePhoneChange}
                maxLength={15}
                placeholder={t("customers.phonePlaceholder", "(00) 00000-0000")}
              />
            </div>
          </div>

          {/* Row 4: Account Active & Marketing Consent */}
          <div className="flex flex-wrap items-center gap-6 pt-2">
            <Switch
              label={t("customers.activeAccount", "Conta Ativa")}
              checked={formValues.active}
              onChange={(checked) => setValue("active", checked)}
            />
            <Switch
              label={t("customers.marketingConsent", "Consentimento de Marketing")}
              checked={formValues.marketingConsent}
              onChange={(checked) => setValue("marketingConsent", checked)}
            />
          </div>

          {/* Optional SysAdmin: Tenant selection */}
          {isSysAdmin && (
            <div>
              <Label htmlFor="customerTenant">
                {t("customers.tenant", "Empresa / Tenant")}
              </Label>
              <FilterableCombobox
                id="customerTenant"
                value={formValues.tenantId}
                options={tenantOptions}
                onChange={(val) => setValue("tenantId", val)}
                placeholder={t("customers.defaultNone", "Padrão / Nenhuma")}
                emptyText={t("tenants.noTenantsFound", "Nenhuma empresa encontrada")}
              />
            </div>
          )}

          {/* Form Actions: Cancel on left, Save on right */}
          <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-gray-800">
            <Link to="/customers">
              <Button variant="outline" type="button">
                {t("customers.cancel", "Cancelar")}
              </Button>
            </Link>
            <Button disabled={isSubmitting || loading} type="submit">
              {isSubmitting
                ? t("customers.saving", "Salvando...")
                : editing
                ? t("customers.saveChanges", "Salvar Alterações")
                : t("customers.createCustomer", "Criar Cliente")}
            </Button>
          </div>
        </form>
      </ComponentCard>
    </>
  );
}

export default CustomerForm;
