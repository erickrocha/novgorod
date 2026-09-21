import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
import {
  addPersonAddress,
  fetchPersonAddresses,
  fetchProfile,
  updatePersonAddress,
} from "@/store/authSlice";
import { locationService } from "@/services/locationService";
import { formatPostalCode, stripNonDigits } from "@/utils/taxId";
import type { Province, City, PersonAddressInput } from "@/services/types";

export default function PersonAddressForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const editing = Boolean(id);

  const { person, addresses } = useAppSelector((state) => state.auth);

  const [form, setForm] = useState({
    addressLine1: "",
    addressLine2: "",
    locality: "",
    administrativeArea: "",
    postalCode: "",
    countryCode: "BR",
  });

  const [provinces, setProvinces] = useState<Province[]>([]);
  const [loadingProvinces, setLoadingProvinces] = useState(false);
  const [cities, setCities] = useState<City[]>([]);
  const [loadingCities, setLoadingCities] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const pageTitle = editing
    ? t("profile.address.editAddress", "Editar Endereço")
    : t("profile.address.addAddress", "Adicionar Endereço");

  useEffect(() => {
    if (!person) {
      dispatch(fetchProfile());
    } else if (addresses.length === 0 && person.id) {
      dispatch(fetchPersonAddresses(person.id));
    }
  }, [dispatch, person, addresses.length]);

  const activeProvinceIdRef = useRef<number | null>(null);

  // Load provinces
  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (!active) return;
      setLoadingProvinces(true);
      locationService
        .provinces("BR")
        .then((data) => {
          if (active) setProvinces(data);
        })
        .catch((err) => {
          console.error("Failed to load provinces", err);
        })
        .finally(() => {
          if (active) setLoadingProvinces(false);
        });
    });
    return () => {
      active = false;
    };
  }, []);

  const loadCities = useCallback(async (provinceId: number) => {
    activeProvinceIdRef.current = provinceId;
    setLoadingCities(true);
    try {
      const list = await locationService.citiesByProvince(provinceId);
      if (activeProvinceIdRef.current === provinceId) {
        setCities(Array.isArray(list) ? list : []);
      }
    } catch (err) {
      if (activeProvinceIdRef.current === provinceId) {
        console.error("Failed to load cities for province", provinceId, err);
        setCities([]);
      }
    } finally {
      if (activeProvinceIdRef.current === provinceId) {
        setLoadingCities(false);
      }
    }
  }, []);

  // Prepopulate form when editing
  useEffect(() => {
    if (editing && id && addresses.length > 0) {
      const existing = addresses.find((a) => String(a.id) === id);
      if (existing) {
        queueMicrotask(() => {
          setForm({
            addressLine1: existing.addressLine1 || "",
            addressLine2: existing.addressLine2 || "",
            locality: existing.locality || "",
            administrativeArea: existing.administrativeArea || "",
            postalCode: existing.postalCode ? formatPostalCode(existing.postalCode) : "",
            countryCode: existing.countryCode || "BR",
          });
        });
      }
    }
  }, [editing, id, addresses]);

  // Automatically load cities when administrativeArea is present and provinces are loaded
  useEffect(() => {
    if (!form.administrativeArea) {
      activeProvinceIdRef.current = null;
      queueMicrotask(() => {
        setCities([]);
      });
      return;
    }
    if (provinces.length === 0) {
      return;
    }
    const target = form.administrativeArea.trim().toLowerCase();
    const prov = provinces.find(
      (p) =>
        p.acronym.toLowerCase() === target || p.name.toLowerCase() === target,
    );
    if (prov && prov.id != null) {
      queueMicrotask(() => {
        loadCities(Number(prov.id));
      });
    } else {
      activeProvinceIdRef.current = null;
      queueMicrotask(() => {
        setCities([]);
      });
    }
  }, [form.administrativeArea, provinces, loadCities]);

  const set = (key: keyof typeof form, value: string) =>
    setForm((current) => ({ ...current, [key]: value }));

  const handleProvinceChange = (val: string, option?: ComboboxOption) => {
    const searchVal = val.trim().toLowerCase();
    const prov =
      (option?.data as Province) ||
      provinces.find(
        (p) =>
          p.acronym.toLowerCase() === searchVal ||
          p.name.toLowerCase() === searchVal,
      );
    const chosenAcronym = prov ? prov.acronym : val;
    setForm((prev) => ({
      ...prev,
      administrativeArea: chosenAcronym,
      locality: "",
    }));
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

  const handlePostalCodeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    set("postalCode", formatPostalCode(e.target.value));
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!person?.id) {
      setError(t("profile.profileUpdateError", "Erro ao obter perfil do usuário"));
      return;
    }

    try {
      setIsSaving(true);
      setError(null);

      const input: PersonAddressInput = {
        tenantId: person.tenantId,
        personId: person.id,
        postalCode: form.postalCode ? stripNonDigits(form.postalCode) || null : null,
        addressLine1: form.addressLine1.trim() || null,
        addressLine2: form.addressLine2.trim() || null,
        locality: form.locality.trim() || null,
        administrativeArea: form.administrativeArea.trim() || null,
        countryCode: form.countryCode.trim().toUpperCase() || "BR",
      };

      if (editing && id) {
        await dispatch(
          updatePersonAddress({ id: Number(id), input })
        ).unwrap();
      } else {
        await dispatch(addPersonAddress(input)).unwrap();
      }

      await dispatch(fetchPersonAddresses(person.id));
      navigate("/profile");
    } catch (err: unknown) {
      console.error("Failed to save person address", err);
      setError(typeof err === "string" ? err : t("profile.profileUpdateError", "Erro ao salvar endereço"));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <>
      <PageMeta
        title={`${pageTitle} | Veche`}
        description={t("profile.address.editAddressDesc", "Gerenciamento de endereço")}
      />
      <PageBreadcrumb
        pageTitle={pageTitle}
        items={[{ label: t("profile.title", "Perfil"), href: "/profile" }]}
      />
      <ComponentCard title={pageTitle}>
        <form onSubmit={submit} className="space-y-5">
          {error && (
            <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600 dark:bg-error-500/10 dark:border-error-500/20">
              {error}
            </div>
          )}

          {/* Address Line 1: 100% */}
          <div>
            <Label htmlFor="addressLine1">
              {t("profile.address.street", "Logradouro / Endereço")}
            </Label>
            <Input
              id="addressLine1"
              value={form.addressLine1}
              onChange={(e) => set("addressLine1", e.target.value)}
              placeholder={t("tenants.addressLine1Placeholder", "Rua, avenida, número, bairro")}
            />
          </div>

          {/* Address Line 2: 80% and Postal Code: 20% */}
          <div className="grid grid-cols-1 md:grid-cols-10 gap-5">
            <div className="md:col-span-8">
              <Label htmlFor="addressLine2">
                {t("profile.address.complement", "Complemento")}
              </Label>
              <Input
                id="addressLine2"
                value={form.addressLine2}
                onChange={(e) => set("addressLine2", e.target.value)}
                placeholder={t("tenants.addressLine2Placeholder", "Sala, bloco, conjunto, andar...")}
              />
            </div>
            <div className="md:col-span-2">
              <Label htmlFor="postalCode">
                {t("profile.address.postalCode", "CEP")}
              </Label>
              <Input
                id="postalCode"
                value={form.postalCode}
                onChange={handlePostalCodeChange}
                maxLength={9}
                placeholder="00000-000"
              />
            </div>
          </div>

          {/* Country (20%), State / Province (30%), City (50%) matching tenant address */}
          <div className="grid grid-cols-1 md:grid-cols-10 gap-5 items-end">
            <div className="md:col-span-2">
              <Label>{t("profile.address.countryCode", "País")}</Label>
              <div className="flex h-11 items-center px-1">
                <Radio
                  id="country-br"
                  name="country"
                  value="BR"
                  checked={form.countryCode === "BR"}
                  onChange={() => set("countryCode", "BR")}
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
              <Label htmlFor="city">
                {t("tenants.city", "Cidade")}
              </Label>
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
                emptyText={
                  loadingCities
                    ? t("tenants.loadingCities", "Carregando cidades...")
                    : t("tenants.noCityFound", "Nenhuma cidade encontrada")
                }
              />
            </div>
          </div>

          {/* Actions: Cancel on left, Save on right */}
          <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-gray-800">
            <Link to="/profile">
              <Button variant="outline" type="button">
                {t("profile.cancel", "Cancelar")}
              </Button>
            </Link>
            <Button disabled={isSaving} type="submit">
              {isSaving
                ? t("profile.address.saving", "Salvando...")
                : t("profile.address.saveAddress", "Salvar Endereço")}
            </Button>
          </div>
        </form>
      </ComponentCard>
    </>
  );
}
