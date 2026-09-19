import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { FileSpreadsheet, Pencil, Plus, RefreshCw } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import { taxRuleService } from "@/services/taxRuleService";
import type { PageQueryParams, TaxRule, TaxRuleInput } from "@/services/types";
import { ROLES } from "@/utils/enums";

const BRAZIL_UFS = [
  "ALL", "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA",
  "MG", "MS", "MT", "PA", "PB", "PE", "PI", "PR", "RJ", "RN",
  "RO", "RR", "RS", "SC", "SE", "SP", "TO",
];

export default function TaxRules() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const [rules, setRules] = useState<TaxRule[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Modal State
  const [modalOpen, setModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<TaxRule | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    name: string;
    ufOrigem: string;
    ufDestino: string;
    ncm: string;
    origemMercadoria: number;
    cst: string;
    aliquotaIcms: string;
    aliquotaIpi: string;
    aliquotaPis: string;
    aliquotaCofins: string;
    tenantId: number | null;
  }>({
    name: "",
    ufOrigem: "SP",
    ufDestino: "SP",
    ncm: "",
    origemMercadoria: 0,
    cst: "00",
    aliquotaIcms: "18.00",
    aliquotaIpi: "0.00",
    aliquotaPis: "1.65",
    aliquotaCofins: "7.60",
    tenantId: user?.tenantId || null,
  });

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
      const res = await taxRuleService.paged(params);
      setRules(res.items);
      setTotal(res.total);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load tax rules";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadData();
      }
    });
    return () => {
      active = false;
    };
  }, [loadData]);

  useEffect(() => {
    if (isSysAdmin && tenants.length === 0) {
      dispatch(fetchTenants());
    }
  }, [dispatch, isSysAdmin, tenants.length]);

  const onPaginationChange = (next: PaginationState) => {
    const params = new URLSearchParams(searchParams);
    params.set("page", String(next.pageIndex + 1));
    params.set("pageSize", String(next.pageSize));
    setSearchParams(params);
  };

  const onSortingChange = (next: SortingState) => {
    const params = new URLSearchParams(searchParams);
    if (next.length > 0) {
      params.set("sortBy", next[0].id);
      params.set("sortDir", next[0].desc ? "desc" : "asc");
    } else {
      params.delete("sortBy");
      params.delete("sortDir");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const onGlobalFilterChange = (val: string) => {
    const params = new URLSearchParams(searchParams);
    if (val) {
      params.set("q", val);
    } else {
      params.delete("q");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const openCreateModal = () => {
    setEditingRule(null);
    setFormData({
      name: "",
      ufOrigem: "SP",
      ufDestino: "SP",
      ncm: "",
      origemMercadoria: 0,
      cst: "00",
      aliquotaIcms: "18.00",
      aliquotaIpi: "0.00",
      aliquotaPis: "1.65",
      aliquotaCofins: "7.60",
      tenantId: user?.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (rule: TaxRule) => {
    setEditingRule(rule);
    setFormData({
      name: rule.name,
      ufOrigem: rule.ufOrigem || "SP",
      ufDestino: rule.ufDestino || "SP",
      ncm: rule.ncm || "",
      origemMercadoria: rule.origemMercadoria ?? 0,
      cst: rule.cst || "00",
      aliquotaIcms: rule.aliquotaIcms != null ? String(rule.aliquotaIcms) : "0.00",
      aliquotaIpi: rule.aliquotaIpi != null ? String(rule.aliquotaIpi) : "0.00",
      aliquotaPis: rule.aliquotaPis != null ? String(rule.aliquotaPis) : "0.00",
      aliquotaCofins: rule.aliquotaCofins != null ? String(rule.aliquotaCofins) : "0.00",
      tenantId: rule.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setSaving(true);
    try {
      const payload: TaxRuleInput = {
        name: formData.name.trim(),
        ufOrigem: formData.ufOrigem === "ALL" ? null : formData.ufOrigem,
        ufDestino: formData.ufDestino === "ALL" ? null : formData.ufDestino,
        ncm: formData.ncm.trim() || null,
        origemMercadoria: formData.origemMercadoria,
        cst: formData.cst.trim() || null,
        aliquotaIcms: formData.aliquotaIcms ? parseFloat(formData.aliquotaIcms) : null,
        aliquotaIpi: formData.aliquotaIpi ? parseFloat(formData.aliquotaIpi) : null,
        aliquotaPis: formData.aliquotaPis ? parseFloat(formData.aliquotaPis) : null,
        aliquotaCofins: formData.aliquotaCofins ? parseFloat(formData.aliquotaCofins) : null,
        tenantId: formData.tenantId,
      };

      if (editingRule?.id) {
        await taxRuleService.update(editingRule.id, payload);
      } else {
        await taxRuleService.create(payload);
      }
      setModalOpen(false);
      loadData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save tax rule";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<TaxRule, unknown>[] = [
    {
      header: "Rule Name",
      accessorKey: "name",
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-semibold text-gray-900 dark:text-white">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: "Origin / Destination",
      id: "route",
      accessorFn: (row) => `${row.ufOrigem || "ANY"} → ${row.ufDestino || "ANY"}`,
      cell: ({ getValue }) => (
        <span className="text-xs font-medium px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded-md text-gray-700 dark:text-gray-300">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: "NCM",
      accessorKey: "ncm",
      cell: ({ getValue }) => getValue<string>() || "—",
    },
    {
      header: "ICMS",
      accessorKey: "aliquotaIcms",
      cell: ({ getValue }) => {
        const val = getValue<number>();
        return val != null ? `${val}%` : "—";
      },
    },
    {
      header: "IPI",
      accessorKey: "aliquotaIpi",
      cell: ({ getValue }) => {
        const val = getValue<number>();
        return val != null ? `${val}%` : "—";
      },
    },
    {
      header: "PIS / COFINS",
      id: "pis_cofins",
      accessorFn: (row) =>
        `${row.aliquotaPis ?? 0}% / ${row.aliquotaCofins ?? 0}%`,
      cell: ({ getValue }) => (
        <span className="text-gray-600 dark:text-gray-400 text-xs">
          {getValue<string>()}
        </span>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            header: "Tenant",
            id: "tenant",
            cell: ({ row }: { row: { original: TaxRule } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      id: "actions",
      header: "",
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end">
          <button
            type="button"
            title="Edit"
            aria-label="Edit"
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openEditModal(row.original)}
          >
            <Pencil size={15} />
          </button>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title="Tax Rules | Veche"
        description="Configure tax rates, ICMS, IPI, PIS, and COFINS"
      />
      <PageBreadcrumb pageTitle="Tax Rules" />
      <ComponentCard>
        <DataGrid
          data={rules}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadData}
              >
                Refresh
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateModal}
              >
                Add Rule
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && rules.length === 0}
          error={error}
          emptyMessage="No tax rules found."
          manualPagination
          manualSorting
          manualFiltering
          totalRows={total}
          pagination={{ pageIndex: Math.max(0, page - 1), pageSize }}
          onPaginationChange={onPaginationChange}
          sorting={[{ id: sortBy, desc: sortDir === "desc" }]}
          onSortingChange={onSortingChange}
          globalFilter={q}
          onGlobalFilterChange={onGlobalFilterChange}
        />
      </ComponentCard>

      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-xl p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <FileSpreadsheet size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingRule ? "Edit Tax Rule" : "Add Tax Rule"}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Set fiscal parameters, origin/destination states, and tax rates.
            </p>
          </div>
        </div>

        {formError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="space-y-4">
          <div>
            <Label htmlFor="ruleName">Rule Name</Label>
            <Input
              id="ruleName"
              placeholder="e.g. Standard Interstate SP to RJ"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="ufOrigem">Origin State</Label>
              <select
                id="ufOrigem"
                value={formData.ufOrigem}
                onChange={(e) => setFormData({ ...formData, ufOrigem: e.target.value })}
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                {BRAZIL_UFS.map((uf) => (
                  <option key={uf} value={uf}>
                    {uf === "ALL" ? "All States (Default)" : uf}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <Label htmlFor="ufDestino">Destination State</Label>
              <select
                id="ufDestino"
                value={formData.ufDestino}
                onChange={(e) => setFormData({ ...formData, ufDestino: e.target.value })}
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                {BRAZIL_UFS.map((uf) => (
                  <option key={uf} value={uf}>
                    {uf === "ALL" ? "All States (Default)" : uf}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-3">
            <div>
              <Label htmlFor="ncm">NCM</Label>
              <Input
                id="ncm"
                placeholder="6109.10.00"
                value={formData.ncm}
                onChange={(e) => setFormData({ ...formData, ncm: e.target.value })}
              />
            </div>
            <div>
              <Label htmlFor="origemMercadoria">Origem (0-8)</Label>
              <select
                id="origemMercadoria"
                value={formData.origemMercadoria}
                onChange={(e) =>
                  setFormData({ ...formData, origemMercadoria: Number(e.target.value) })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value={0}>0 - Nacional</option>
                <option value={1}>1 - Estrangeira (Importação direta)</option>
                <option value={2}>2 - Estrangeira (Mercado interno)</option>
              </select>
            </div>
            <div>
              <Label htmlFor="cst">CST</Label>
              <Input
                id="cst"
                placeholder="00"
                value={formData.cst}
                onChange={(e) => setFormData({ ...formData, cst: e.target.value })}
              />
            </div>
          </div>

          <div className="grid grid-cols-4 gap-3">
            <div>
              <Label htmlFor="icms">ICMS (%)</Label>
              <Input
                id="icms"
                type="number"
                step={0.01}
                min="0"
                value={formData.aliquotaIcms}
                onChange={(e) =>
                  setFormData({ ...formData, aliquotaIcms: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="ipi">IPI (%)</Label>
              <Input
                id="ipi"
                type="number"
                step={0.01}
                min="0"
                value={formData.aliquotaIpi}
                onChange={(e) =>
                  setFormData({ ...formData, aliquotaIpi: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="pis">PIS (%)</Label>
              <Input
                id="pis"
                type="number"
                step={0.01}
                min="0"
                value={formData.aliquotaPis}
                onChange={(e) =>
                  setFormData({ ...formData, aliquotaPis: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="cofins">COFINS (%)</Label>
              <Input
                id="cofins"
                type="number"
                step={0.01}
                min="0"
                value={formData.aliquotaCofins}
                onChange={(e) =>
                  setFormData({ ...formData, aliquotaCofins: e.target.value })
                }
              />
            </div>
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">Tenant</Label>
              <select
                id="tenantId"
                value={formData.tenantId || ""}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    tenantId: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="">Default / None</option>
                {tenants.map((t) => (
                  <option key={t.id} value={t.id ?? ""}>
                    {t.businessName || t.companyName}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="mt-6 flex justify-end gap-3 pt-3 border-t border-gray-100 dark:border-gray-800">
            <Button
              type="button"
              variant="outline"
              onClick={() => setModalOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? "Saving…" : "Save Rule"}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
}
