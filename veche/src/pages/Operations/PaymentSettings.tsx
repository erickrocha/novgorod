import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useForm } from "react-hook-form";
import { yupResolver } from "@hookform/resolvers/yup";
import * as yup from "yup";
import {
  CheckCircle2,
  AlertCircle,
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
  clearPaymentSettingsStatus,
  fetchPaymentSettings,
  updatePaymentSettings,
} from "@/store/paymentSettingsSlice";
import { ROLES } from "@/utils/enums";
import type { PaymentProvider, PaymentSettingsUpdate, Tenant } from "@/services/types";

const paymentSettingsSchema = yup.object({
  provider: yup.string().oneOf(["mercado_pago", "pagseguro"]).required(),
  mpAccessToken: yup.string().default(""),
  mpPublicKey: yup.string().default(""),
  mpCollectorId: yup.string().default(""),
  mpWebhookSecret: yup.string().default(""),
  psToken: yup.string().default(""),
  psPublicKey: yup.string().default(""),
  psEnvironment: yup.string().oneOf(["sandbox", "production"]).default("sandbox"),
});

type PaymentSettingsFormData = yup.InferType<typeof paymentSettingsSchema>;

export default function PaymentSettings() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { user } = useAppSelector((state) => state.auth);
  const tenants = useAppSelector((state) => state.tenant.tenantsList);
  const tenantsLoading = useAppSelector((state) => state.tenant.loading);
  const { settings, loading, saving, error, saveSuccess } = useAppSelector(
    (state) => state.paymentSettings
  );

  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;
  const [selectedTenantId, setSelectedTenantId] = useState<number | null>(
    user?.tenantId ?? null
  );

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { isDirty },
  } = useForm<PaymentSettingsFormData>({
    resolver: yupResolver(paymentSettingsSchema),
    defaultValues: {
      provider: "mercado_pago",
      mpAccessToken: "",
      mpPublicKey: "",
      mpCollectorId: "",
      mpWebhookSecret: "",
      psToken: "",
      psPublicKey: "",
      psEnvironment: "sandbox",
    },
  });

  const selectedProvider = watch("provider") as PaymentProvider;

  useEffect(() => {
    if (isSysAdmin && tenants.length === 0) {
      dispatch(fetchTenants());
    }
  }, [isSysAdmin, tenants.length, dispatch]);

  useEffect(() => {
    if (selectedTenantId) {
      dispatch(fetchPaymentSettings(selectedTenantId));
    }
  }, [selectedTenantId, dispatch]);

  useEffect(() => {
    if (settings) {
      reset({
        provider: settings.provider,
        mpAccessToken: "",
        mpPublicKey: settings.provider === "mercado_pago" ? (settings.publicKey ?? "") : "",
        mpCollectorId: "",
        mpWebhookSecret: "",
        psToken: "",
        psPublicKey: settings.provider === "pagseguro" ? (settings.publicKey ?? "") : "",
        psEnvironment: (settings.environment as "sandbox" | "production") ?? "sandbox",
      });
    }
  }, [settings, reset]);

  const onSubmit = async (data: PaymentSettingsFormData) => {
    if (!selectedTenantId) return;

    dispatch(clearPaymentSettingsStatus());

    const updatePayload: PaymentSettingsUpdate = {
      provider: data.provider as PaymentProvider,
    };

    if (data.provider === "pagseguro") {
      const hasPsChanges = Boolean(
        data.psToken.trim() || data.psPublicKey.trim() || data.psEnvironment
      );
      if (hasPsChanges) {
        updatePayload.credentials = {
          pagseguro: {
            token: data.psToken.trim() ? data.psToken.trim() : undefined,
            publicKey: data.psPublicKey.trim() ? data.psPublicKey.trim() : undefined,
            environment: data.psEnvironment || "sandbox",
          },
        };
      }
    } else {
      const hasMpChanges = Boolean(
        data.mpAccessToken.trim() ||
          data.mpPublicKey.trim() ||
          data.mpCollectorId.trim() ||
          data.mpWebhookSecret.trim()
      );
      if (hasMpChanges) {
        updatePayload.credentials = {
          mercadoPago: {
            accessToken: data.mpAccessToken.trim() ? data.mpAccessToken.trim() : undefined,
            publicKey: data.mpPublicKey.trim() ? data.mpPublicKey.trim() : undefined,
            collectorId: data.mpCollectorId.trim() ? Number(data.mpCollectorId.trim()) : undefined,
            webhookSecret: data.mpWebhookSecret.trim() ? data.mpWebhookSecret.trim() : undefined,
          },
        };
      }
    }

    await dispatch(
      updatePaymentSettings({
        tenantId: selectedTenantId,
        data: updatePayload,
      })
    );
  };

  const tenantOptions: ComboboxOption[] = (tenants || []).map((t: Tenant) => ({
    value: String(t.id),
    label: `${t.businessName || t.companyName || "Tenant"} (#${t.id})`,
  }));

  return (
    <>
      <PageMeta
        title="Configurações de Pagamento | Veche"
        description="Configure o provedor de pagamento (Mercado Pago ou PagSeguro) para o checkout multitenant."
      />
      <PageBreadcrumb pageTitle={t("paymentSettings.title", "Configurações de Pagamento")} />

      <div className="space-y-6">
        {isSysAdmin && (
          <ComponentCard title={t("paymentSettings.selectTenant", "Selecionar Tenant")}>
            <div className="max-w-md">
              <Label>{t("paymentSettings.tenant", "Tenant")}</Label>
              <FilterableCombobox
                options={tenantOptions}
                value={selectedTenantId ? String(selectedTenantId) : ""}
                onChange={(val) => {
                  setSelectedTenantId(val ? Number(val) : null);
                  dispatch(clearPaymentSettingsStatus());
                }}
                placeholder={t("paymentSettings.chooseTenant", "Selecione um tenant...")}
                disabled={tenantsLoading}
              />
            </div>
          </ComponentCard>
        )}

        {selectedTenantId && (
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            {saveSuccess && (
              <div className="flex items-center gap-3 p-4 rounded-xl bg-emerald-50 text-emerald-800 border border-emerald-200">
                <CheckCircle2 className="w-5 h-5 flex-shrink-0" />
                <p className="text-sm font-medium">
                  {t("paymentSettings.saveSuccess", "Configurações de pagamento salvas com sucesso.")}
                </p>
              </div>
            )}

            {error && (
              <div className="flex items-center gap-3 p-4 rounded-xl bg-rose-50 text-rose-800 border border-rose-200">
                <AlertCircle className="w-5 h-5 flex-shrink-0" />
                <p className="text-sm font-medium">{error}</p>
              </div>
            )}

            <ComponentCard title={t("paymentSettings.providerSelection", "Provedor de Pagamento")}>
              <div className="space-y-4">
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {t(
                    "paymentSettings.providerDescription",
                    "Escolha qual gateway de pagamento será utilizado nas vendas deste lojista."
                  )}
                </p>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <label
                    className={`flex items-start gap-4 p-4 rounded-2xl border cursor-pointer transition ${
                      selectedProvider === "mercado_pago"
                        ? "border-brand-500 bg-brand-50/20 dark:bg-brand-500/10"
                        : "border-gray-200 dark:border-gray-800 hover:border-gray-300"
                    }`}
                  >
                    <input
                      type="radio"
                      value="mercado_pago"
                      {...register("provider")}
                      className="mt-1 text-brand-600 focus:ring-brand-500"
                    />
                    <div>
                      <p className="font-semibold text-gray-900 dark:text-white">
                        Mercado Pago
                      </p>
                      <p className="text-xs text-gray-500 mt-1">
                        {t(
                          "paymentSettings.mercadoPagoDesc",
                          "Checkout transparente via cartão com conciliação automática."
                        )}
                      </p>
                    </div>
                  </label>

                  <label
                    className={`flex items-start gap-4 p-4 rounded-2xl border cursor-pointer transition ${
                      selectedProvider === "pagseguro"
                        ? "border-brand-500 bg-brand-50/20 dark:bg-brand-500/10"
                        : "border-gray-200 dark:border-gray-800 hover:border-gray-300"
                    }`}
                  >
                    <input
                      type="radio"
                      value="pagseguro"
                      {...register("provider")}
                      className="mt-1 text-brand-600 focus:ring-brand-500"
                    />
                    <div>
                      <p className="font-semibold text-gray-900 dark:text-white">
                        PagSeguro / PagBank
                      </p>
                      <p className="text-xs text-gray-500 mt-1">
                        {t(
                          "paymentSettings.pagseguroDesc",
                          "Gateway PagBank com criptografia no navegador e conciliação direta."
                        )}
                      </p>
                    </div>
                  </label>
                </div>

                <div className="flex items-center gap-3 pt-2">
                  {settings?.credentialsConfigured ? (
                    <div className="flex items-center gap-2 text-xs font-medium text-emerald-700 bg-emerald-50 px-3 py-1.5 rounded-full border border-emerald-200">
                      <ShieldCheck className="w-4 h-4" />
                      <span>{t("paymentSettings.credentialsConfigured", "Credenciais configuradas e salvas com segurança")}</span>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2 text-xs font-medium text-amber-700 bg-amber-50 px-3 py-1.5 rounded-full border border-amber-200">
                      <ShieldAlert className="w-4 h-4" />
                      <span>{t("paymentSettings.credentialsNotConfigured", "Nenhuma credencial cadastrada para este tenant (usando padrão global)")}</span>
                    </div>
                  )}
                </div>
              </div>
            </ComponentCard>

            {selectedProvider === "mercado_pago" ? (
              <ComponentCard title="Credenciais Mercado Pago">
                <div className="space-y-4 max-w-xl">
                  <div>
                    <Label htmlFor="mpAccessToken">Access Token (Chave Secreta)</Label>
                    <Input
                      id="mpAccessToken"
                      type="password"
                      placeholder={
                        settings?.credentialsConfigured && settings.provider === "mercado_pago"
                          ? "•••••••••••••••• (deixe em branco para manter)"
                          : "TEST-XXXXX ou APP_USR-XXXXX"
                      }
                      {...register("mpAccessToken")}
                    />
                  </div>

                  <div>
                    <Label htmlFor="mpPublicKey">Public Key (Chave Pública)</Label>
                    <Input
                      id="mpPublicKey"
                      type="text"
                      placeholder="TEST-XXXXX ou APP_USR-XXXXX"
                      {...register("mpPublicKey")}
                    />
                  </div>

                  <div>
                    <Label htmlFor="mpCollectorId">Collector ID (ID da Conta Mercado Pago)</Label>
                    <Input
                      id="mpCollectorId"
                      type="text"
                      placeholder="Ex: 123456789"
                      {...register("mpCollectorId")}
                    />
                  </div>

                  <div>
                    <Label htmlFor="mpWebhookSecret">Webhook Secret (Opcional)</Label>
                    <Input
                      id="mpWebhookSecret"
                      type="password"
                      placeholder="Segredo para validação de assinaturas de webhook"
                      {...register("mpWebhookSecret")}
                    />
                  </div>
                </div>
              </ComponentCard>
            ) : (
              <ComponentCard title="Credenciais PagSeguro / PagBank">
                <div className="space-y-4 max-w-xl">
                  <div>
                    <Label htmlFor="psToken">Token de Acesso PagSeguro</Label>
                    <Input
                      id="psToken"
                      type="password"
                      placeholder={
                        settings?.credentialsConfigured && settings.provider === "pagseguro"
                          ? "•••••••••••••••• (deixe em branco para manter)"
                          : "Token PagBank gerado no painel"
                      }
                      {...register("psToken")}
                    />
                  </div>

                  <div>
                    <Label htmlFor="psPublicKey">Chave Pública (Para criptografia no frontend)</Label>
                    <Input
                      id="psPublicKey"
                      type="text"
                      placeholder="Chave pública do PagBank"
                      {...register("psPublicKey")}
                    />
                  </div>

                  <div>
                    <Label htmlFor="psEnvironment">Ambiente</Label>
                    <select
                      id="psEnvironment"
                      className="w-full rounded-xl border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 p-3 text-sm focus:border-brand-500 focus:outline-none"
                      {...register("psEnvironment")}
                    >
                      <option value="sandbox">Sandbox (Testes)</option>
                      <option value="production">Produção (Live)</option>
                    </select>
                  </div>
                </div>
              </ComponentCard>
            )}

            <div className="flex items-center gap-4">
              <Button
                type="submit"
                disabled={saving || loading}
                className="flex items-center gap-2"
              >
                <Save className="w-4 h-4" />
                {saving ? t("common.saving", "Salvando...") : t("common.save", "Salvar Configurações")}
              </Button>

              <Button
                type="button"
                variant="outline"
                disabled={saving || !isDirty}
                onClick={() => {
                  if (settings) {
                    reset({
                      provider: settings.provider,
                      mpAccessToken: "",
                      mpPublicKey: settings.publicKey ?? "",
                      mpCollectorId: "",
                      mpWebhookSecret: "",
                      psToken: "",
                      psPublicKey: settings.publicKey ?? "",
                      psEnvironment: (settings.environment as "sandbox" | "production") ?? "sandbox",
                    });
                  }
                }}
                className="flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" />
                {t("common.reset", "Desfazer")}
              </Button>
            </div>
          </form>
        )}
      </div>
    </>
  );
}
