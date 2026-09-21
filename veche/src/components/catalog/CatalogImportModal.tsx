import { useEffect, useMemo, useRef, useState } from "react";
import Papa from "papaparse";
import * as XLSX from "xlsx";
import { Upload, X, Building2, CheckCircle2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import FilterableCombobox, {
  type ComboboxOption,
} from "@/components/form/FilterableCombobox";
import { catalogService } from "@/services/catalogService";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenantById, fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";

type ImportRow = {
  product_key: string;
  sku_key: string;
  category_key: string;
  product_name: string;
  slug: string;
  description: string;
  brand: string;
  ncm: string;
  cest: string;
  origem_mercadoria: number;
  code: string;
  variant_key: string;
  price_cents: number;
  compare_at_price_cents: string;
  weight_g: number;
  width_mm: number;
  height_mm: number;
  length_mm: number;
  active: boolean;
  color: string;
};

const blank = (): ImportRow => ({
  product_key: "",
  sku_key: "",
  category_key: "",
  product_name: "",
  slug: "",
  description: "",
  brand: "",
  ncm: "22042100",
  cest: "0301200",
  origem_mercadoria: 0,
  code: "",
  variant_key: "750ml",
  price_cents: 0,
  compare_at_price_cents: "",
  weight_g: 0,
  width_mm: 0,
  height_mm: 0,
  length_mm: 0,
  active: true,
  color: "",
});

const flatWorkbook = (book: XLSX.WorkBook): ImportRow[] => {
  const read = (name: string) =>
    book.Sheets[name]
      ? XLSX.utils.sheet_to_json<Record<string, unknown>>(book.Sheets[name], {
          defval: "",
        })
      : [];
  const products = read("Products");
  const skus = read("SKUs");
  const links = read("ProductCategories");
  const categories = new Map(
    read("Categories").map((row) => [
      String(row.external_key),
      String(row.slug || row.external_key),
    ])
  );
  const productMap = new Map(
    products.map((row) => [String(row.external_key), row])
  );
  return skus.map((row) => {
    const p = productMap.get(String(row.product_key)) || {};
    const link = links.find(
      (item) => String(item.product_key) === String(row.product_key)
    );
    const result = blank();
    Object.assign(result, {
      product_key: String(row.product_key),
      sku_key: String(row.external_key),
      category_key: categories.get(String(link?.category_key || "")) || "",
      product_name: String(p.name || ""),
      slug: String(p.slug || ""),
      description: String(p.description || ""),
      brand: String(p.brand || ""),
      ncm: String(p.ncm || "22042100"),
      cest: String(p.cest || "0301200"),
      origem_mercadoria: Number(p.origem_mercadoria || 0),
      code: String(row.code || ""),
      variant_key: String(row.variant_key || "750ml"),
      price_cents: Number(row.price_cents || 0),
      weight_g: Number(row.weight_g || 0),
      width_mm: Number(row.width_mm || 0),
      height_mm: Number(row.height_mm || 0),
      length_mm: Number(row.length_mm || 0),
    });
    return result;
  });
};

export default function CatalogImportModal({
  onClose,
  onImported,
}: {
  onClose: () => void;
  onImported: () => void;
}) {
  const { t } = useTranslation();
  const input = useRef<HTMLInputElement>(null);
  const [rows, setRows] = useState<ImportRow[]>([]);
  const [file, setFile] = useState<File | null>(null);
  const [tenantId, setTenantId] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState<{
    total: number;
    created: number;
    updated: number;
    unchanged: number;
  } | null>(null);

  const dispatch = useAppDispatch();
  const { user } = useAppSelector((s) => s.auth);
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const activeTenant = useAppSelector((s) => s.tenant.activeTenant);

  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const ownTenantId = user?.tenantId ?? user?.tenant_id ?? activeTenant?.id;
  const ownTenant =
    (activeTenant?.id === ownTenantId ? activeTenant : null) ||
    tenants.find((t) => t.id === ownTenantId) ||
    activeTenant;
  const ownTenantName =
    ownTenant?.businessName ||
    ownTenant?.companyName ||
    (ownTenantId ? `#${ownTenantId}` : "");

  useEffect(() => {
    if (isSysAdmin && tenants.length === 0) {
      dispatch(fetchTenants());
    } else if (!isSysAdmin && ownTenantId && !activeTenant) {
      dispatch(fetchTenantById(Number(ownTenantId)));
    }
  }, [dispatch, isSysAdmin, tenants.length, ownTenantId, activeTenant]);

  const effectiveTenant = isSysAdmin
    ? tenantId
      ? Number(tenantId)
      : null
    : ownTenantId
    ? Number(ownTenantId)
    : null;

  const tenantOptions: ComboboxOption[] = useMemo(
    () =>
      tenants
        .filter((t) => t.id != null)
        .map((t) => ({
          value: String(t.id),
          label: t.businessName || t.companyName || `Tenant #${t.id}`,
        })),
    [tenants]
  );

  const errors = useMemo(
    () =>
      rows
        .flatMap((row, index) => [
          !row.product_name.trim() &&
            `Row ${index + 1}: product name is required`,
          !row.slug.trim() && `Row ${index + 1}: slug is required`,
          !row.code.trim() && `Row ${index + 1}: SKU code is required`,
          row.price_cents < 0 && `Row ${index + 1}: price cannot be negative`,
        ])
        .filter(Boolean) as string[],
    [rows]
  );

  const parse = (selected: File) => {
    setFile(selected);
    setError("");
    if (selected.name.toLowerCase().endsWith(".csv")) {
      Papa.parse<ImportRow>(selected, {
        header: true,
        skipEmptyLines: true,
        complete: (res) =>
          setRows(
            res.data.map((row) => ({
              ...blank(),
              ...row,
              price_cents: Number(row.price_cents || 0),
              origem_mercadoria: Number(row.origem_mercadoria || 0),
              weight_g: Number(row.weight_g || 0),
              width_mm: Number(row.width_mm || 0),
              height_mm: Number(row.height_mm || 0),
              length_mm: Number(row.length_mm || 0),
              active: String(row.active) !== "false",
            }))
          ),
        error: (e) => setError(e.message),
      });
    } else {
      selected
        .arrayBuffer()
        .then((buffer) => setRows(flatWorkbook(XLSX.read(buffer, { type: "array" }))))
        .catch((e) => setError(String(e)));
    }
  };

  const update = (index: number, key: keyof ImportRow, value: string) =>
    setRows((current) =>
      current.map((row, i) =>
        i === index
          ? {
              ...row,
              [key]: [
                "price_cents",
                "weight_g",
                "width_mm",
                "height_mm",
                "length_mm",
              ].includes(key)
                ? Number(value)
                : value,
            }
          : row
      )
    );

  const canSubmit = Boolean(
    file &&
    !busy &&
    errors.length === 0 &&
    (isSysAdmin ? Boolean(effectiveTenant) : true)
  );

  const submit = async () => {
    if (!canSubmit || !file) return;
    setBusy(true);
    setError("");
    try {
      const res = await catalogService.importCatalog(
        file,
        isSysAdmin ? effectiveTenant : (effectiveTenant ?? undefined),
        rows
      );
      setResult(res);
      onImported();
    } catch (e: any) {
      setError(
        e?.response?.data?.message ||
          t("catalog.importFailed", "Falha na importação do catálogo.")
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-99999 flex items-center justify-center bg-black/50 p-4">
      <div className="max-h-[90vh] w-full max-w-7xl overflow-hidden rounded-xl bg-white p-6 shadow-theme-lg dark:bg-gray-900">
        {/* Header */}
        <div className="mb-4 flex items-center justify-between border-b border-gray-100 pb-4 dark:border-gray-800">
          <div>
            <h2 className="text-xl font-semibold text-gray-800 dark:text-white">
              {t("catalog.importTitle", "Importar catálogo de vinhos")}
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {t(
                "catalog.importSubtitle",
                "Envie um arquivo CSV/XLSX, revise cada linha, edite os valores se necessário e importe."
              )}
            </p>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="rounded-lg p-1.5 text-gray-400 hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-white/5 dark:hover:text-gray-200"
          >
            <X size={20} />
          </button>
        </div>

        {/* Success Result View */}
        {result ? (
          <div className="space-y-4 py-6 text-center">
            <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-success-50 text-success-600 dark:bg-success-950/30">
              <CheckCircle2 size={32} />
            </div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              {t("catalog.importSuccess", "Catálogo importado com sucesso!")}
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-300">
              {t("catalog.importSummary", {
                total: result.total,
                created: result.created,
                updated: result.updated,
                unchanged: result.unchanged,
                defaultValue: `Importadas ${result.total} linhas: ${result.created} criadas, ${result.updated} atualizadas, ${result.unchanged} inalteradas.`,
              })}
            </p>
            <div className="pt-2">
              <Button onClick={onClose}>{t("catalog.close", "Fechar")}</Button>
            </div>
          </div>
        ) : (
          <>
            {/* Top Controls: File input & Tenant info / selection */}
            <div className="mb-4 flex flex-wrap items-end gap-5">
              <div className="min-w-[260px]">
                <Label>{t("catalog.catalogFile", "Arquivo do catálogo")}</Label>
                <input
                  ref={input}
                  type="file"
                  accept=".csv,.xlsx"
                  className="block w-full text-sm text-gray-500 file:me-4 file:rounded-lg file:border-0 file:bg-brand-50 file:px-4 file:py-2.5 file:text-sm file:font-semibold file:text-brand-700 hover:file:bg-brand-100 dark:file:bg-white/5 dark:file:text-gray-300"
                  onChange={(e) => e.target.files?.[0] && parse(e.target.files[0])}
                />
              </div>

              {/* SysAdmin: target tenant selector */}
              {isSysAdmin && (
                <div className="min-w-[280px] flex-1">
                  <Label htmlFor="targetTenant">
                    {t("catalog.targetTenant", "Empresa de destino")}
                  </Label>
                  <FilterableCombobox
                    id="targetTenant"
                    options={tenantOptions}
                    value={tenantId}
                    onChange={(v) => setTenantId(v)}
                    placeholder={t("catalog.selectTenant", "Selecione a empresa...")}
                  />
                </div>
              )}

              {/* Informative banner showing tenant name for non-sysadmin users */}
              {!isSysAdmin && (
                <div className="flex items-center gap-2 rounded-lg border border-brand-200 bg-brand-50 px-4 py-2.5 text-sm font-medium text-brand-800 dark:border-brand-800 dark:bg-brand-950/30 dark:text-brand-300">
                  <Building2 size={18} className="shrink-0 text-brand-600 dark:text-brand-400" />
                  <span>
                    {t("catalog.tenantOwnerNotice", {
                      tenantName: ownTenantName || (ownTenantId ? `#${ownTenantId}` : ""),
                      defaultValue: `Tudo será importado para a sua empresa: ${ownTenantName || (ownTenantId ? `#${ownTenantId}` : "")}`,
                    })}
                  </span>
                </div>
              )}

              <span className="text-sm text-gray-500 dark:text-gray-400 pb-2">
                {file
                  ? `${file.name} · ${t("catalog.rowsCount", { count: rows.length, defaultValue: `${rows.length} linhas` })}`
                  : t("catalog.noFileSelected", "Nenhum arquivo selecionado")}
              </span>
            </div>

            {/* Error alerts */}
            {error && (
              <div className="mb-3 flex items-center gap-2 rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600 dark:border-error-800 dark:bg-error-950/30">
                <AlertCircle size={18} className="shrink-0" />
                <span>{error}</span>
              </div>
            )}
            {errors.length > 0 && (
              <div className="mb-3 rounded-lg border border-warning-200 bg-warning-50 p-3 text-sm text-warning-700 dark:border-warning-800 dark:bg-warning-950/30">
                <p className="font-medium">
                  {t("catalog.validationErrors", {
                    count: errors.length,
                    defaultValue: `${errors.length} erro(s) de validação. Corrija-os antes de importar.`,
                  })}
                </p>
                <ul className="mt-1 list-disc list-inside text-xs space-y-0.5 max-h-24 overflow-y-auto">
                  {errors.slice(0, 5).map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                  {errors.length > 5 && (
                    <li>... {errors.length - 5} mais erro(s)</li>
                  )}
                </ul>
              </div>
            )}

            {/* Editable Preview Table */}
            <div className="max-h-[50vh] overflow-auto rounded-lg border border-gray-200 dark:border-gray-800">
              <table className="w-full min-w-[1100px] text-start text-xs">
                <thead className="bg-gray-50 dark:bg-gray-800/60 sticky top-0 z-10">
                  <tr className="border-b border-gray-200 dark:border-gray-800 text-gray-600 dark:text-gray-300">
                    <th className="p-2 text-start">#</th>
                    <th className="p-2 text-start">{t("catalog.import.productCol", "Product")}</th>
                    <th className="p-2 text-start">{t("catalog.import.slugCol", "Slug")}</th>
                    <th className="p-2 text-start">{t("catalog.import.skuCol", "SKU code")}</th>
                    <th className="p-2 text-start">{t("catalog.import.variantCol", "Variant")}</th>
                    <th className="p-2 text-start">{t("catalog.import.priceCol", "Price cents")}</th>
                    <th className="p-2 text-start">{t("catalog.import.categoryCol", "Category")}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                  {rows.length === 0 ? (
                    <tr>
                      <td colSpan={7} className="py-8 text-center text-gray-400">
                        {t("catalog.selectFile", "Selecione um arquivo para pré-visualizar.")}
                      </td>
                    </tr>
                  ) : (
                    rows.map((row, index) => (
                      <tr
                        key={`${row.sku_key}-${index}`}
                        className="hover:bg-gray-50/50 dark:hover:bg-white/5 transition-colors"
                      >
                        <td className="p-2 text-gray-400">{index + 1}</td>
                        <td className="p-1">
                          <Input
                            value={row.product_name}
                            onChange={(e) =>
                              update(index, "product_name", e.target.value)
                            }
                          />
                        </td>
                        <td className="p-1">
                          <Input
                            value={row.slug}
                            onChange={(e) =>
                              update(index, "slug", e.target.value)
                            }
                          />
                        </td>
                        <td className="p-1">
                          <Input
                            value={row.code}
                            onChange={(e) =>
                              update(index, "code", e.target.value)
                            }
                          />
                        </td>
                        <td className="p-1">
                          <Input
                            value={row.variant_key}
                            onChange={(e) =>
                              update(index, "variant_key", e.target.value)
                            }
                          />
                        </td>
                        <td className="p-1">
                          <Input
                            type="number"
                            value={row.price_cents}
                            onChange={(e) =>
                              update(index, "price_cents", e.target.value)
                            }
                          />
                        </td>
                        <td className="p-2 text-gray-600 dark:text-gray-400">
                          {row.category_key}
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>

            {/* Footer Buttons */}
            <div className="mt-5 flex justify-end gap-3">
              <Button variant="outline" onClick={onClose} type="button">
                {t("catalog.cancel", "Cancelar")}
              </Button>
              <Button
                disabled={!canSubmit}
                startIcon={<Upload size={16} />}
                onClick={submit}
                type="button"
              >
                {busy
                  ? t("catalog.importing", "Importando…")
                  : t("catalog.importButton", "Importar catálogo")}
              </Button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
