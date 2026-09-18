import { useEffect, useState, useMemo, useCallback } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Radio from "@/components/form/input/Radio";
import Button from "@/components/ui/button/Button";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import { BrFlagIcon } from "@/icons";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { createTenant, fetchTenants, updateTenant } from "@/store/tenantSlice";
import { locationService } from "@/services/locationService";
import { useLanguage } from "@/context/LanguageContext";
import { ROLES } from "@/utils/enums";
import type { TenantInput, Province, City } from "@/services/types";

export default function TenantForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const editing = Boolean(id);
  const { tenantsList, loading, error } = useAppSelector((s) => s.tenant);
  const { user } = useAppSelector((s) => s.auth);
  const sysAdmin = user?.role === ROLES.SYS_ADMIN;
  const ownId = user?.tenantId ?? user?.tenant_id;
  const existing = tenantsList.find((t) => String(t.id) === id);

  const { language } = useLanguage();
  const defaultCountryCode = useMemo(() => {
    if (language === "pt-BR") return "BR";
    if (language.includes("-")) return language.split("-")[1].toUpperCase();
    return "BR";
  }, [language]);

  const pageTitle = editing
    ? t("tenants.editTenant", "Editar empresa")
    : t("tenants.addTenant", "Nova empresa");

  const [form, setForm] = useState({
    businessName: "",
    companyName: "",
    taxId: "",
    email: "",
    phone: "",
    website: "",
    addressLine1: "",
    addressLine2: "",
    locality: "",
    administrativeArea: "",
    postalCode: "",
    countryCode: defaultCountryCode,
  });

  const [provinces, setProvinces] = useState<Province[]>([]);
  const [loadingProvinces, setLoadingProvinces] = useState(false);
  const [cities, setCities] = useState<City[]>([]);
  const [loadingCities, setLoadingCities] = useState(false);

  useEffect(() => {
    dispatch(fetchTenants());
  }, [dispatch]);

  // Load provinces list
  useEffect(() => {
    setLoadingProvinces(true);
    locationService
      .provinces("BR")
      .then((data) => setProvinces(data))
      .catch(() => {})
      .finally(() => setLoadingProvinces(false));
  }, []);

  const loadCities = useCallback(async (provinceId: number) => {
    setLoadingCities(true);
    try {
      const list = await locationService.citiesByProvince(provinceId);
      setCities(list);
    } catch {
      setCities([]);
    } finally {
      setLoadingCities(false);
    }
  }, []);

  // Synchronize existing record into form
  useEffect(() => {
    if (existing) {
      const provVal = existing.administrativeArea || existing.province || "";
      setForm({
        businessName: existing.businessName || "",
        companyName: existing.companyName || "",
        taxId: existing.taxId || "",
        email: existing.email || "",
        phone: existing.phone || "",
        website: existing.website || "",
        addressLine1: existing.addressLine1 || "",
        addressLine2: existing.addressLine2 || "",
        locality: existing.locality || existing.city || "",
        administrativeArea: provVal,
        postalCode: existing.postalCode || existing.zipcode || "",
        countryCode: existing.countryCode || defaultCountryCode,
      });

      if (provVal && provinces.length > 0) {
        const match = provinces.find(
          (p) =>
            p.acronym.toLowerCase() === provVal.toLowerCase() ||
            p.name.toLowerCase() === provVal.toLowerCase(),
        );
        if (match && match.id) {
          loadCities(match.id);
        }
      }
    }
  }, [existing, provinces, defaultCountryCode, loadCities]);

  const set = (key: keyof typeof form, value: string) =>
    setForm((current) => ({ ...current, [key]: value }));

  const handleProvinceChange = (val: string, option?: ComboboxOption) => {
    const prov =
      (option?.data as Province) ||
      provinces.find((p) => p.acronym === val || p.name === val);
    const chosenAcronym = prov ? prov.acronym : val;
    setForm((prev) => ({
      ...prev,
      administrativeArea: chosenAcronym,
      locality: "",
    }));

    if (prov && prov.id) {
      loadCities(prov.id);
    } else {
      setCities([]);
    }
  };

  const provinceOptions: ComboboxOption[] = useMemo(() => {
    return provinces.map((p) => ({
      value: p.acronym,
      label: `${p.name} (${p.acronym})`,
      sublabel: `UF: ${p.acronym}`,
      data: p,
    }));
  }, [provinces]);

  const cityOptions: ComboboxOption[] = useMemo(() => {
    return cities.map((c) => ({
      value: c.name,
      label: c.name,
      data: c,
    }));
  }, [cities]);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const payload = Object.fromEntries(
      Object.entries(form).map(([k, v]) => [
        k,
        typeof v === "string" ? v.trim() || null : v,
      ]),
    ) as TenantInput;

    payload.countryCode = form.countryCode || defaultCountryCode || "BR";

    const result =
      editing && existing?.id
        ? await dispatch(updateTenant({ id: existing.id, tenantData: payload }))
        : await dispatch(createTenant(payload));

    if (result.meta.requestStatus === "fulfilled") {
      await dispatch(fetchTenants());
      navigate("/tenants");
    }
  };

  if (!sysAdmin && (!editing || !existing || existing.id !== ownId)) {
    return (
      <div className="rounded-lg border border-error-200 bg-error-50 p-4 text-error-600">
        {t("tenants.noAccess", "Você não tem acesso a esta empresa.")}
      </div>
    );
  }

  return (
    <>
      <PageMeta
        title={`${pageTitle} | Veche`}
        description={t("tenants.tenantForm", "Formulário de empresa")}
      />
      <PageBreadcrumb pageTitle={pageTitle} />
      <ComponentCard title={pageTitle}>
        <form onSubmit={submit} className="space-y-5">
          {/* First row: Business name (70%) and Tax ID (30%) */}
          <div className="grid grid-cols-1 md:grid-cols-10 gap-5">
            <div className="md:col-span-7">
              <Label htmlFor="businessName">
                {t("tenants.businessName", "Nome Fantasia")}
              </Label>
              <Input
                id="businessName"
                required
                value={form.businessName}
                onChange={(e) => set("businessName", e.target.value)}
                placeholder={t("tenants.businessNamePlaceholder", "Nome fantasia")}
              />
            </div>
            <div className="md:col-span-3">
              <Label htmlFor="taxId">{t("tenants.taxId", "CNPJ")}</Label>
              <Input
                id="taxId"
                required
                value={form.taxId}
                onChange={(e) => set("taxId", e.target.value)}
                placeholder={t("tenants.taxIdPlaceholder", "CNPJ")}
              />
            </div>
          </div>

          {/* Second row: Company name (100%) */}
          <div>
            <Label htmlFor="companyName">
              {t("tenants.companyName", "Razão Social")}
            </Label>
            <Input
              id="companyName"
              value={form.companyName}
              onChange={(e) => set("companyName", e.target.value)}
              placeholder={t("tenants.companyNamePlaceholder", "Razão social")}
            />
          </div>

          {/* Third row: Phone (50%) and Website (50%) */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="phone">{t("tenants.phone", "Telefone")}</Label>
              <Input
                id="phone"
                type="tel"
                value={form.phone}
                onChange={(e) => set("phone", e.target.value)}
                placeholder={t("tenants.phonePlaceholder", "Telefone")}
              />
            </div>
            <div>
              <Label htmlFor="website">{t("tenants.website", "Website")}</Label>
              <Input
                id="website"
                type="text"
                value={form.website}
                onChange={(e) => set("website", e.target.value)}
                placeholder={t("tenants.websitePlaceholder", "Website")}
              />
            </div>
          </div>

          {/* Fourth row: Fieldset with title Address */}
          <fieldset className="rounded-xl border border-gray-200 dark:border-gray-800 p-5 space-y-5">
            <legend className="px-2 text-sm font-semibold text-gray-700 dark:text-gray-300">
              {t("tenants.address", "Endereço")}
            </legend>

            {/* Address Row 1: Address line 1 (100%) */}
            <div>
              <Label htmlFor="addressLine1">
                {t("tenants.addressLine1", "Logradouro / Endereço")}
              </Label>
              <Input
                id="addressLine1"
                value={form.addressLine1}
                onChange={(e) => set("addressLine1", e.target.value)}
                placeholder={t("tenants.addressLine1Placeholder", "Endereço")}
              />
            </div>

            {/* Address Row 2: Address line 2 (80%) and Postal code (20%) */}
            <div className="grid grid-cols-1 md:grid-cols-10 gap-5">
              <div className="md:col-span-8">
                <Label htmlFor="addressLine2">
                  {t("tenants.addressLine2", "Complemento")}
                </Label>
                <Input
                  id="addressLine2"
                  value={form.addressLine2}
                  onChange={(e) => set("addressLine2", e.target.value)}
                  placeholder={t("tenants.addressLine2Placeholder", "Complemento")}
                />
              </div>
              <div className="md:col-span-2">
                <Label htmlFor="postalCode">{t("tenants.postalCode", "CEP")}</Label>
                <Input
                  id="postalCode"
                  value={form.postalCode}
                  onChange={(e) => set("postalCode", e.target.value)}
                  placeholder={t("tenants.postalCodePlaceholder", "CEP")}
                />
              </div>
            </div>

            {/* Address Row 3: Radio button Brazil flag (20%), State / Province (30%), City (50%) */}
            <div className="grid grid-cols-1 md:grid-cols-10 gap-5 items-end">
              <div className="md:col-span-2">
                <Label>{t("tenants.country", "País")}</Label>
                <div className="flex h-11 items-center px-1">
                  <Radio
                    id="country-br"
                    name="country"
                    value="BR"
                    checked={true}
                    onChange={() => {}}
                    label={
                      <span className="flex items-center gap-2">
                        <BrFlagIcon className="size-5 rounded-full overflow-hidden shrink-0 shadow-xs" />
                        <span className="text-sm font-medium text-gray-800 dark:text-gray-200">
                          Brasil
                        </span>
                      </span>
                    }
                  />
                </div>
              </div>
              <div className="md:col-span-3">
                <Label htmlFor="stateProvince">
                  {t("tenants.stateProvince", "Estado / UF")}
                </Label>
                <FilterableCombobox
                  id="stateProvince"
                  value={form.administrativeArea}
                  options={provinceOptions}
                  onChange={handleProvinceChange}
                  placeholder={t("tenants.selectState", "Selecione o estado...")}
                  loading={loadingProvinces}
                  emptyText={t("tenants.noStateFound", "Nenhum estado encontrado")}
                />
              </div>
              <div className="md:col-span-5">
                <Label htmlFor="city">{t("tenants.city", "Cidade")}</Label>
                <FilterableCombobox
                  id="city"
                  value={form.locality}
                  options={cityOptions}
                  onChange={(val) => set("locality", val)}
                  placeholder={
                    !form.administrativeArea
                      ? t("tenants.selectStateFirst", "Selecione o estado primeiro...")
                      : loadingCities
                      ? t("tenants.loadingCities", "Carregando cidades...")
                      : t("tenants.selectOrSearchCity", "Selecione ou busque a cidade...")
                  }
                  disabled={!form.administrativeArea}
                  loading={loadingCities}
                  emptyText={t("tenants.noCityFound", "Nenhuma cidade encontrada")}
                />
              </div>
            </div>
          </fieldset>

          {error && (
            <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">
              {error}
            </div>
          )}

          {/* Button cancel aligned left, button save aligned to the right */}
          <div className="flex items-center justify-between pt-3">
            <Link to="/tenants">
              <Button variant="outline" type="button">
                {t("tenants.cancel", "Cancelar")}
              </Button>
            </Link>
            <Button disabled={loading} type="submit">
              {editing
                ? t("tenants.saveChanges", "Salvar alterações")
                : t("tenants.createTenant", "Criar empresa")}
            </Button>
          </div>
        </form>
      </ComponentCard>
    </>
  );
}
