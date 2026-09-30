import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useForm, useFieldArray } from "react-hook-form";
import { yupResolver } from "@hookform/resolvers/yup";
import * as yup from "yup";
import {
  CheckCircle2,
  AlertCircle,
  Plus,
  Trash2,
  ShieldCheck,
  ShieldAlert,
  Save,
  RotateCcw,
} from "lucide-react";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Button from "@/components/ui/button/Button";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import {
  clearShippingSettingsStatus,
  fetchShippingSettings,
  updateShippingSettings,
} from "@/store/shippingSettingsSlice";
import { ROLES } from "@/utils/enums";
import type { ShippingMode, ShippingSettingsUpdate } from "@/services/types";

const shippingSettingsSchema = yup.object({
  mode: yup.string().oneOf(["fixed", "correios"]).required(),
  originCep: yup.string().defined().default(""),
  services: yup
    .array()
    .of(
      yup.object({
        code: yup.string().trim().required("Service code is required"),
        name: yup.string().trim().required("Service name is required"),
      })
    )
    .defined()
    .default([]),
  packaging: yup.object({
    weightG: yup
      .number()
      .transform((val, orig) => (orig === "" || isNaN(val) ? 0 : val))
      .min(0, "Must be 0 or greater")
      .required(),
    lengthMm: yup
      .number()
      .transform((val, orig) => (orig === "" || isNaN(val) ? 0 : val))
      .min(0, "Must be 0 or greater")
      .required(),
    widthMm: yup
      .number()
      .transform((val, orig) => (orig === "" || isNaN(val) ? 0 : val))
      .min(0, "Must be 0 or greater")
      .required(),
    heightMm: yup
      .number()
      .transform((val, orig) => (orig === "" || isNaN(val) ? 0 : val))
      .min(0, "Must be 0 or greater")
      .required(),
  }),
  credentials: yup.object({
    username: yup.string().defined().default(""),
    apiAccessCode: yup.string().defined().default(""),
    postingCard: yup.string().defined().default(""),
    contract: yup.string().defined().default(""),
    regionalIdentifier: yup.string().defined().default(""),
  }),
});

type ShippingSettingsFormData = yup.InferType<typeof shippingSettingsSchema>;

export function ShippingSettings() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;
  const tenants = useAppSelector((s) => s.tenant.tenantsList);

  const [selectedTenantId, setSelectedTenantId] = useState<number | null>(
    user?.tenantId || null
  );

  const { settings, loading, saving, error, saveSuccess } = useAppSelector(
    (s) => s.shippingSettings
  );

  const {
    register,
    handleSubmit,
    control,
    reset,
    watch,
    formState: { errors },
  } = useForm<ShippingSettingsFormData>({
    resolver: yupResolver(shippingSettingsSchema),
    defaultValues: {
      mode: "fixed",
      originCep: "",
      services: [
        { code: "03220", name: "SEDEX" },
        { code: "03298", name: "PAC" },
      ],
      packaging: {
        weightG: 0,
        lengthMm: 0,
        widthMm: 0,
        heightMm: 0,
      },
      credentials: {
        username: "",
        apiAccessCode: "",
        postingCard: "",
        contract: "",
        regionalIdentifier: "",
      },
    },
  });

  const { fields, append, remove } = useFieldArray({
    control,
    name: "services",
  });

  const currentMode = watch("mode") as ShippingMode;

  // Fetch tenants if SysAdmin
  useEffect(() => {
    if (isSysAdmin) {
      dispatch(fetchTenants({ page: 1, pageSize: 50 }));
    }
  }, [dispatch, isSysAdmin]);

  // Set default selected tenant for SysAdmin if none set
  useEffect(() => {
    if (isSysAdmin && !selectedTenantId && tenants.length > 0) {
      const firstId = tenants[0].id;
      if (firstId) {
        setSelectedTenantId(firstId);
      }
    }
  }, [isSysAdmin, selectedTenantId, tenants]);

  // Load shipping settings when selectedTenantId changes
  useEffect(() => {
    if (selectedTenantId) {
      dispatch(clearShippingSettingsStatus());
      dispatch(fetchShippingSettings(selectedTenantId));
    }
  }, [dispatch, selectedTenantId]);

  // Populate form when settings arrive
  useEffect(() => {
    if (settings) {
      const cfg = settings.configuration;
      reset({
        mode: cfg.mode || "fixed",
        originCep: cfg.originCep || "",
        services:
          cfg.services && cfg.services.length > 0
            ? cfg.services
            : [
                { code: "03220", name: "SEDEX" },
                { code: "03298", name: "PAC" },
              ],
        packaging: {
          weightG: cfg.packaging?.weightG || 0,
          lengthMm: cfg.packaging?.lengthMm || 0,
          widthMm: cfg.packaging?.widthMm || 0,
          heightMm: cfg.packaging?.heightMm || 0,
        },
        credentials: {
          username: "",
          apiAccessCode: "",
          postingCard: "",
          contract: "",
          regionalIdentifier: "",
        },
      });
    }
  }, [settings, reset]);

  const onSubmit = async (data: ShippingSettingsFormData) => {
    if (!selectedTenantId) return;

    const payload: ShippingSettingsUpdate = {
      configuration: {
        mode: data.mode as ShippingMode,
        originCep: data.originCep?.trim() || null,
        services: data.services || [],
        packaging: {
          weightG: Number(data.packaging.weightG) || 0,
          lengthMm: Number(data.packaging.lengthMm) || 0,
          widthMm: Number(data.packaging.widthMm) || 0,
          heightMm: Number(data.packaging.heightMm) || 0,
        },
      },
    };

    // Only send credentials object if at least one field has input
    const c = data.credentials;
    const hasAnyCred =
      Boolean(c.username?.trim()) ||
      Boolean(c.apiAccessCode?.trim()) ||
      Boolean(c.postingCard?.trim()) ||
      Boolean(c.contract?.trim()) ||
      Boolean(c.regionalIdentifier?.trim());

    if (hasAnyCred) {
      payload.credentials = {
        username: c.username?.trim() || undefined,
        apiAccessCode: c.apiAccessCode?.trim() || undefined,
        postingCard: c.postingCard?.trim() || undefined,
        contract: c.contract?.trim() || undefined,
        regionalIdentifier: c.regionalIdentifier?.trim() || undefined,
      };
    }

    dispatch(
      updateShippingSettings({
        tenantId: selectedTenantId,
        data: payload,
      })
    );
  };

  const handleReset = () => {
    if (settings) {
      const cfg = settings.configuration;
      reset({
        mode: cfg.mode || "fixed",
        originCep: cfg.originCep || "",
        services: cfg.services || [],
        packaging: {
          weightG: cfg.packaging?.weightG || 0,
          lengthMm: cfg.packaging?.lengthMm || 0,
          widthMm: cfg.packaging?.widthMm || 0,
          heightMm: cfg.packaging?.heightMm || 0,
        },
        credentials: {
          username: "",
          apiAccessCode: "",
          postingCard: "",
          contract: "",
          regionalIdentifier: "",
        },
      });
    }
    dispatch(clearShippingSettingsStatus());
  };

  const tenantOptions: ComboboxOption[] = tenants.map((t) => ({
    value: String(t.id),
    label: t.businessName || t.companyName || `Tenant #${t.id}`,
  }));

  return (
    <div className="space-y-6">
      <PageMeta
        title={`${t("operations.shippingSettings.title", "Shipping Settings")} | Veche`}
        description={t(
          "operations.shippingSettings.desc",
          "Configure shipping mode, carrier credentials, and packaging dimensions"
        )}
      />

      <PageBreadcrumb
        pageTitle={t("operations.shippingSettings.title", "Shipping Settings")}
        showTitle={true}
      />

      {/* Tenant selector for SysAdmin */}
      {isSysAdmin && (
        <ComponentCard
          title={t("operations.shippingSettings.selectTenant", "Select Seller")}
        >
          <div className="max-w-md">
            <Label htmlFor="tenantId">
              {t("operations.shippingSettings.selectTenant", "Select Seller")}
            </Label>
            <FilterableCombobox
              options={tenantOptions}
              value={selectedTenantId ? String(selectedTenantId) : ""}
              onChange={(val) => setSelectedTenantId(Number(val) || null)}
              placeholder={t(
                "operations.shippingSettings.selectTenant",
                "Select Seller"
              )}
            />
          </div>
        </ComponentCard>
      )}

      {/* Success Notification */}
      {saveSuccess && (
        <div className="flex items-center gap-3 p-4 text-sm text-green-800 bg-green-50 border border-green-200 rounded-lg dark:bg-green-950/30 dark:text-green-300 dark:border-green-800">
          <CheckCircle2 className="w-5 h-5 flex-shrink-0" />
          <span>
            {t(
              "operations.shippingSettings.savedSuccess",
              "Shipping settings saved successfully!"
            )}
          </span>
        </div>
      )}

      {/* Error Notification */}
      {error && (
        <div className="flex items-center gap-3 p-4 text-sm text-red-800 bg-red-50 border border-red-200 rounded-lg dark:bg-red-950/30 dark:text-red-300 dark:border-red-800">
          <AlertCircle className="w-5 h-5 flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Shipping Mode Card */}
        <ComponentCard
          title={t("operations.shippingSettings.mode", "Shipping Mode")}
        >
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <label
                className={`relative flex items-center p-4 border rounded-xl cursor-pointer transition-colors ${
                  currentMode === "fixed"
                    ? "border-brand-500 bg-brand-50/20 dark:bg-brand-950/20 dark:border-brand-500"
                    : "border-gray-200 hover:border-gray-300 dark:border-gray-800 dark:hover:border-gray-700"
                }`}
              >
                <input
                  type="radio"
                  value="fixed"
                  {...register("mode")}
                  className="sr-only"
                />
                <div className="flex items-center gap-3">
                  <div
                    className={`w-5 h-5 rounded-full border flex items-center justify-center ${
                      currentMode === "fixed"
                        ? "border-brand-500 bg-brand-500"
                        : "border-gray-400"
                    }`}
                  >
                    {currentMode === "fixed" && (
                      <div className="w-2 h-2 rounded-full bg-white" />
                    )}
                  </div>
                  <div>
                    <span className="block font-medium text-gray-900 dark:text-white">
                      {t(
                        "operations.shippingSettings.fixed",
                        "Fixed Rates by State (UF)"
                      )}
                    </span>
                    <span className="block text-xs text-gray-500 dark:text-gray-400">
                      {t(
                        "operations.shippingSettings.fixedHelp",
                        "Uses flat shipping rates configured per state under Shipping Rates"
                      )}
                    </span>
                  </div>
                </div>
              </label>

              <label
                className={`relative flex items-center p-4 border rounded-xl cursor-pointer transition-colors ${
                  currentMode === "correios"
                    ? "border-brand-500 bg-brand-50/20 dark:bg-brand-950/20 dark:border-brand-500"
                    : "border-gray-200 hover:border-gray-300 dark:border-gray-800 dark:hover:border-gray-700"
                }`}
              >
                <input
                  type="radio"
                  value="correios"
                  {...register("mode")}
                  className="sr-only"
                />
                <div className="flex items-center gap-3">
                  <div
                    className={`w-5 h-5 rounded-full border flex items-center justify-center ${
                      currentMode === "correios"
                        ? "border-brand-500 bg-brand-500"
                        : "border-gray-400"
                    }`}
                  >
                    {currentMode === "correios" && (
                      <div className="w-2 h-2 rounded-full bg-white" />
                    )}
                  </div>
                  <div>
                    <span className="block font-medium text-gray-900 dark:text-white">
                      {t(
                        "operations.shippingSettings.correios",
                        "Correios (Carrier API)"
                      )}
                    </span>
                    <span className="block text-xs text-gray-500 dark:text-gray-400">
                      {t(
                        "operations.shippingSettings.correiosHelp",
                        "Calculates live price and transit times via Correios API"
                      )}
                    </span>
                  </div>
                </div>
              </label>
            </div>

            {/* Origin CEP */}
            <div className="max-w-md pt-2">
              <Label htmlFor="originCep">
                {t(
                  "operations.shippingSettings.originCep",
                  "Origin Postal Code (CEP)"
                )}
              </Label>
              <Input
                id="originCep"
                placeholder="01001-000"
                {...register("originCep")}
                error={Boolean(errors.originCep)}
                hint={errors.originCep?.message}
              />
              <span className="text-xs text-gray-500 dark:text-gray-400 mt-1 block">
                {t(
                  "operations.shippingSettings.originCepHelp",
                  "Leave empty to use seller registered postal code."
                )}
              </span>
            </div>
          </div>
        </ComponentCard>

        {/* Correios Credentials Card (Active in Correios mode) */}
        {currentMode === "correios" && (
          <ComponentCard
            title={t(
              "operations.shippingSettings.credentials",
              "Correios Credentials"
            )}
            desc={t(
              "operations.shippingSettings.credentialsNote",
              "Credentials are encrypted with AES-256-GCM. Leave fields blank to preserve previously saved values."
            )}
          >
            <div className="space-y-4">
              {/* Configuration Status Badge */}
              <div className="flex items-center gap-2">
                {settings?.credentialsConfigured ? (
                  <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                    <ShieldCheck className="w-4 h-4" />
                    {t(
                      "operations.shippingSettings.credentialsConfigured",
                      "Credentials configured on server"
                    )}
                  </div>
                ) : (
                  <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
                    <ShieldAlert className="w-4 h-4" />
                    {t(
                      "operations.shippingSettings.credentialsMissing",
                      "No credentials configured"
                    )}
                  </div>
                )}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="username">
                    {t(
                      "operations.shippingSettings.username",
                      "Meu Correios Username"
                    )}
                  </Label>
                  <Input
                    id="username"
                    autoComplete="off"
                    placeholder={
                      settings?.credentialsConfigured
                        ? t(
                            "operations.shippingSettings.unchangedPlaceholder",
                            "•••••••• (unchanged)"
                          )
                        : t(
                            "operations.shippingSettings.usernamePlaceholder",
                            "Meu Correios username"
                          )
                    }
                    {...register("credentials.username")}
                  />
                </div>

                <div>
                  <Label htmlFor="apiAccessCode">
                    {t(
                      "operations.shippingSettings.apiAccessCode",
                      "API Access Code"
                    )}
                  </Label>
                  <Input
                    id="apiAccessCode"
                    type="password"
                    autoComplete="off"
                    placeholder={
                      settings?.credentialsConfigured
                        ? t(
                            "operations.shippingSettings.unchangedPlaceholder",
                            "•••••••• (unchanged)"
                          )
                        : t(
                            "operations.shippingSettings.apiAccessCodePlaceholder",
                            "API access code"
                          )
                    }
                    {...register("credentials.apiAccessCode")}
                  />
                </div>

                <div>
                  <Label htmlFor="postingCard">
                    {t(
                      "operations.shippingSettings.postingCard",
                      "Posting Card Number"
                    )}
                  </Label>
                  <Input
                    id="postingCard"
                    autoComplete="off"
                    placeholder={
                      settings?.credentialsConfigured
                        ? t(
                            "operations.shippingSettings.unchangedPlaceholder",
                            "•••••••• (unchanged)"
                          )
                        : t(
                            "operations.shippingSettings.postingCardPlaceholder",
                            "Posting card number"
                          )
                    }
                    {...register("credentials.postingCard")}
                  />
                </div>

                <div>
                  <Label htmlFor="contract">
                    {t(
                      "operations.shippingSettings.contract",
                      "Contract Number"
                    )}
                  </Label>
                  <Input
                    id="contract"
                    autoComplete="off"
                    placeholder={
                      settings?.credentialsConfigured
                        ? t(
                            "operations.shippingSettings.unchangedPlaceholder",
                            "•••••••• (unchanged)"
                          )
                        : t(
                            "operations.shippingSettings.contractPlaceholder",
                            "Contract number"
                          )
                    }
                    {...register("credentials.contract")}
                  />
                </div>

                <div>
                  <Label htmlFor="regionalIdentifier">
                    {t(
                      "operations.shippingSettings.regionalIdentifier",
                      "Regional Identifier (DR)"
                    )}
                  </Label>
                  <Input
                    id="regionalIdentifier"
                    autoComplete="off"
                    placeholder={
                      settings?.credentialsConfigured
                        ? t(
                            "operations.shippingSettings.unchangedShortPlaceholder",
                            "•• (unchanged)"
                          )
                        : t(
                            "operations.shippingSettings.regionalIdentifierPlaceholder",
                            "e.g. 72"
                          )
                    }
                    {...register("credentials.regionalIdentifier")}
                  />
                </div>
              </div>
            </div>
          </ComponentCard>
        )}

        {/* Contracted Services (Active in Correios mode) */}
        {currentMode === "correios" && (
          <ComponentCard
            title={t(
              "operations.shippingSettings.services",
              "Contracted Services"
            )}
          >
            <div className="space-y-4">
              <div className="space-y-3">
                {fields.map((field, index) => (
                  <div
                    key={field.id}
                    className="flex flex-col sm:flex-row items-center gap-3 p-3 bg-gray-50 dark:bg-gray-800/40 rounded-lg border border-gray-200 dark:border-gray-700"
                  >
                    <div className="flex-1 w-full">
                      <Label htmlFor={`services.${index}.code`}>
                        {t(
                          "operations.shippingSettings.serviceCode",
                          "Service Code"
                        )}
                      </Label>
                      <Input
                        id={`services.${index}.code`}
                        placeholder="03220"
                        {...register(`services.${index}.code` as const)}
                      />
                    </div>
                    <div className="flex-1 w-full">
                      <Label htmlFor={`services.${index}.name`}>
                        {t(
                          "operations.shippingSettings.serviceName",
                          "Service Name"
                        )}
                      </Label>
                      <Input
                        id={`services.${index}.name`}
                        placeholder="SEDEX"
                        {...register(`services.${index}.name` as const)}
                      />
                    </div>
                    <div className="sm:self-end pt-2 sm:pt-0">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => remove(index)}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => append({ code: "", name: "" })}
                className="flex items-center gap-1.5"
              >
                <Plus className="w-4 h-4" />
                {t(
                  "operations.shippingSettings.addService",
                  "Add Service"
                )}
              </Button>
            </div>
          </ComponentCard>
        )}

        {/* Packaging Allowances */}
        <ComponentCard
          title={t(
            "operations.shippingSettings.packaging",
            "Packaging Allowances"
          )}
          desc={t(
            "operations.shippingSettings.packagingDesc",
            "Additional package weight and box dimensions to add to item measurements"
          )}
        >
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <Label htmlFor="packagingWeight">
                {t(
                  "operations.shippingSettings.packagingWeight",
                  "Packaging Weight (g)"
                )}
              </Label>
              <Input
                id="packagingWeight"
                type="number"
                min="0"
                {...register("packaging.weightG")}
                error={Boolean(errors.packaging?.weightG)}
                hint={errors.packaging?.weightG?.message}
              />
            </div>

            <div>
              <Label htmlFor="packagingLength">
                {t(
                  "operations.shippingSettings.packagingLength",
                  "Length Allowance (mm)"
                )}
              </Label>
              <Input
                id="packagingLength"
                type="number"
                min="0"
                {...register("packaging.lengthMm")}
                error={Boolean(errors.packaging?.lengthMm)}
                hint={errors.packaging?.lengthMm?.message}
              />
            </div>

            <div>
              <Label htmlFor="packagingWidth">
                {t(
                  "operations.shippingSettings.packagingWidth",
                  "Width Allowance (mm)"
                )}
              </Label>
              <Input
                id="packagingWidth"
                type="number"
                min="0"
                {...register("packaging.widthMm")}
                error={Boolean(errors.packaging?.widthMm)}
                hint={errors.packaging?.widthMm?.message}
              />
            </div>

            <div>
              <Label htmlFor="packagingHeight">
                {t(
                  "operations.shippingSettings.packagingHeight",
                  "Height Allowance (mm)"
                )}
              </Label>
              <Input
                id="packagingHeight"
                type="number"
                min="0"
                {...register("packaging.heightMm")}
                error={Boolean(errors.packaging?.heightMm)}
                hint={errors.packaging?.heightMm?.message}
              />
            </div>
          </div>
        </ComponentCard>

        {/* Form Actions: Cancel on left, Save on right */}
        <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-800">
          <Button
            type="button"
            variant="outline"
            onClick={handleReset}
            disabled={saving || loading}
            className="flex items-center gap-2"
          >
            <RotateCcw className="w-4 h-4" />
            {t("operations.shippingSettings.cancel", "Cancel")}
          </Button>

          <Button
            type="submit"
            disabled={saving || loading || !selectedTenantId}
            className="flex items-center gap-2"
          >
            <Save className="w-4 h-4" />
            {saving
              ? t("operations.shippingSettings.saving", "Saving...")
              : t("operations.shippingSettings.save", "Save Settings")}
          </Button>
        </div>
      </form>
    </div>
  );
}

export default ShippingSettings;
