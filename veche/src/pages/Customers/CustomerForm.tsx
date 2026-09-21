import { useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
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
import { customerService } from "@/services/customerService";
import { ROLES } from "@/utils/enums";
import { formatCpf, formatPhone, stripNonDigits } from "@/utils/taxId";
import type { CustomerInput } from "@/services/types";

export default function CustomerForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const editing = Boolean(id);

  const { tenantsList } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [form, setForm] = useState({
    name: "",
    email: "",
    password: "",
    cpf: "",
    phone: "",
    active: true,
    marketingConsent: false,
    tenantId: user?.tenantId ? String(user.tenantId) : "",
  });

  const pageTitle = editing
    ? t("customers.editCustomer", "Editar Cliente")
    : t("customers.addCustomer", "Novo Cliente");

  useEffect(() => {
    if (isSysAdmin && tenantsList.length === 0) {
      dispatch(fetchTenants());
    }
  }, [dispatch, isSysAdmin, tenantsList.length]);

  useEffect(() => {
    if (editing && id) {
      let active = true;
      queueMicrotask(() => {
        if (!active) return;
        setLoading(true);
        customerService
          .getById(Number(id))
          .then((c) => {
            if (!active) return;
            setForm({
              name: c.name || "",
              email: c.email || "",
              password: "",
              cpf: c.cpf ? formatCpf(c.cpf) : "",
              phone: c.phone ? formatPhone(c.phone) : "",
              active: c.active ?? true,
              marketingConsent: c.marketingConsent ?? false,
              tenantId: c.tenantId ? String(c.tenantId) : "",
            });
          })
          .catch((err) => {
            if (!active) return;
            const msg =
              err instanceof Error ? err.message : t("customers.saveError", "Falha ao carregar cliente");
            setError(msg);
          })
          .finally(() => {
            if (active) setLoading(false);
          });
      });

      return () => {
        active = false;
      };
    }
  }, [editing, id, t]);

  const set = (key: keyof typeof form, value: unknown) =>
    setForm((current) => ({ ...current, [key]: value }));

  const tenantOptions: ComboboxOption[] = useMemo(() => {
    const list = tenantsList
      .filter((tn) => tn.id != null)
      .map((tn) => ({
        value: String(tn.id),
        label: tn.businessName || tn.companyName || `Tenant #${tn.id}`,
      }));

    if (form.tenantId && !list.some((o) => o.value === form.tenantId)) {
      list.unshift({
        value: form.tenantId,
        label: `Tenant #${form.tenantId}`,
      });
    }
    return list;
  }, [tenantsList, form.tenantId]);

  const handleCpfChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    set("cpf", formatCpf(e.target.value));
  };

  const handlePhoneChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    set("phone", formatPhone(e.target.value));
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSaving(true);

    try {
      const payload: CustomerInput = {
        name: form.name.trim(),
        email: form.email.trim(),
        cpf: form.cpf ? stripNonDigits(form.cpf) || null : null,
        phone: form.phone ? stripNonDigits(form.phone) || null : null,
        active: form.active,
        marketingConsent: form.marketingConsent,
        tenantId: isSysAdmin
          ? (form.tenantId ? Number(form.tenantId) : null)
          : (user?.tenantId ?? null),
        ...(form.password ? { password: form.password } : {}),
      };

      if (editing && id) {
        await customerService.update(Number(id), payload);
      } else {
        await customerService.create(payload);
      }

      navigate("/customers");
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : t("customers.saveError", "Falha ao salvar cliente");
      setError(msg);
    } finally {
      setSaving(false);
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
        <form onSubmit={submit} className="space-y-5">
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
              required
              disabled={loading}
              value={form.name}
              onChange={(e) => set("name", e.target.value)}
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
                required
                disabled={loading}
                value={form.email}
                onChange={(e) => set("email", e.target.value)}
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
                value={form.password}
                onChange={(e) => set("password", e.target.value)}
                placeholder={t("customers.passwordPlaceholder", "••••••••")}
                {...(!editing ? { required: true } : {})}
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
                value={form.cpf}
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
                value={form.phone}
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
              checked={form.active}
              onChange={(checked) => set("active", checked)}
            />
            <Switch
              label={t("customers.marketingConsent", "Consentimento de Marketing")}
              checked={form.marketingConsent}
              onChange={(checked) => set("marketingConsent", checked)}
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
                value={form.tenantId}
                options={tenantOptions}
                onChange={(val) => set("tenantId", val)}
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
            <Button disabled={saving || loading} type="submit">
              {saving
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
