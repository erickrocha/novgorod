import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Clock,
  MapPin,
  Pencil,
  Plus,
  RefreshCw,
  Sparkles,
  Trash2,
  Truck,
  Warehouse as WarehouseIcon,
} from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Badge from "@/components/ui/badge/Badge";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import {
  deleteShippingRate,
  fetchShippingRatesPaged,
  saveShippingRate,
} from "@/store/operationSlice";
import { warehouseService } from "@/services/warehouseService";
import type {
  PageQueryParams,
  ShippingRate,
  ShippingRateInput,
  Warehouse,
  WarehouseInput,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

const BRAZIL_UFS = [
  "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA",
  "MG", "MS", "MT", "PA", "PB", "PE", "PI", "PR", "RJ", "RN",
  "RO", "RR", "RS", "SC", "SE", "SP", "TO",
];

function formatCep(val?: string | null): string {
  if (!val) return "";
  const digits = val.replace(/\D/g, "");
  if (digits.length === 8) {
    return `${digits.slice(0, 5)}-${digits.slice(5)}`;
  }
  return val;
}

function cleanCepDigits(val: string): string {
  return val.replace(/\D/g, "");
}

export function ShippingRates() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const { shippingRatesPaged, loading, error } = useAppSelector((s) => s.operation);
  const rates = shippingRatesPaged?.items || [];
  const total = shippingRatesPaged?.total || 0;

  // Warehouses list
  const [warehouses, setWarehouses] = useState<Warehouse[]>([]);
  const [warehousesLoading, setWarehousesLoading] = useState(false);

  // Rate Modal State
  const [modalOpen, setModalOpen] = useState(false);
  const [editingRate, setEditingRate] = useState<ShippingRate | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const [formData, setFormData] = useState<{
    originWarehouseId: number | null;
    regionName: string;
    uf: string;
    destinationCepStart: string;
    destinationCepEnd: string;
    price: string;
    transitDaysMin: number;
    transitDaysMax: number;
    maxWeightKg: string;
    extraWeightPerKg: string;
    freeShippingThreshold: string;
    tenantId: number | null;
  }>({
    originWarehouseId: null,
    regionName: "",
    uf: "SP",
    destinationCepStart: "",
    destinationCepEnd: "",
    price: "12.90",
    transitDaysMin: 1,
    transitDaysMax: 2,
    maxWeightKg: "2.0",
    extraWeightPerKg: "5.00",
    freeShippingThreshold: "",
    tenantId: user?.tenantId || null,
  });

  // Delete Rate State
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [rateToDelete, setRateToDelete] = useState<ShippingRate | null>(null);
  const [deleting, setDeleting] = useState(false);

  // Warehouses Management Modal State
  const [warehouseModalOpen, setWarehouseModalOpen] = useState(false);
  const [whFormData, setWhFormData] = useState<{
    name: string;
    originCep: string;
    city: string;
    uf: string;
    street: string;
    number: string;
    district: string;
    isDefault: boolean;
  }>({
    name: "",
    originCep: "",
    city: "São Paulo",
    uf: "SP",
    street: "",
    number: "",
    district: "",
    isDefault: false,
  });
  const [whSaving, setWhSaving] = useState(false);
  const [whError, setWhError] = useState<string | null>(null);

  // Preset loading state
  const [presetLoading, setPresetLoading] = useState(false);

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  const loadData = useCallback(() => {
    const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
    dispatch(fetchShippingRatesPaged(params));
  }, [dispatch, page, pageSize, q, sortBy, sortDir]);

  const loadWarehouses = useCallback(async () => {
    setWarehousesLoading(true);
    try {
      const data = await warehouseService.list();
      setWarehouses(data);
    } catch {
      // silently ignore or keep empty
    } finally {
      setWarehousesLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadData();
        loadWarehouses();
      }
    });
    return () => {
      active = false;
    };
  }, [loadData, loadWarehouses]);

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
    setEditingRate(null);
    setFormData({
      originWarehouseId: warehouses.find((w) => w.isDefault)?.id || null,
      regionName: "",
      uf: "SP",
      destinationCepStart: "",
      destinationCepEnd: "",
      price: "12.90",
      transitDaysMin: 1,
      transitDaysMax: 2,
      maxWeightKg: "2.0",
      extraWeightPerKg: "5.00",
      freeShippingThreshold: "",
      tenantId: user?.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (rate: ShippingRate) => {
    setEditingRate(rate);
    setFormData({
      originWarehouseId: rate.originWarehouseId || null,
      regionName: rate.regionName || "",
      uf: rate.uf,
      destinationCepStart: formatCep(rate.destinationCepStart),
      destinationCepEnd: formatCep(rate.destinationCepEnd),
      price: (rate.priceCents / 100).toFixed(2),
      transitDaysMin: rate.transitDaysMin ?? 1,
      transitDaysMax: rate.transitDaysMax ?? 2,
      maxWeightKg: rate.maxWeightG ? (rate.maxWeightG / 1000).toString() : "",
      extraWeightPerKg: rate.extraWeightPerKgCents
        ? (rate.extraWeightPerKgCents / 100).toFixed(2)
        : "",
      freeShippingThreshold: rate.freeShippingThresholdCents
        ? (rate.freeShippingThresholdCents / 100).toFixed(2)
        : "",
      tenantId: rate.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setSaving(true);
    try {
      const priceCents = Math.round(parseFloat(formData.price || "0") * 100);
      const maxWeightG = formData.maxWeightKg.trim()
        ? Math.round(parseFloat(formData.maxWeightKg) * 1000)
        : null;
      const extraWeightPerKgCents = formData.extraWeightPerKg.trim()
        ? Math.round(parseFloat(formData.extraWeightPerKg) * 100)
        : null;
      const freeShippingThresholdCents = formData.freeShippingThreshold.trim()
        ? Math.round(parseFloat(formData.freeShippingThreshold) * 100)
        : null;

      const cepStartDigits = cleanCepDigits(formData.destinationCepStart);
      const cepEndDigits = cleanCepDigits(formData.destinationCepEnd);

      const payload: ShippingRateInput = {
        originWarehouseId: formData.originWarehouseId,
        regionName: formData.regionName.trim() || null,
        uf: formData.uf.toUpperCase().trim(),
        destinationCepStart: cepStartDigits.length === 8 ? cepStartDigits : null,
        destinationCepEnd: cepEndDigits.length === 8 ? cepEndDigits : null,
        priceCents,
        transitDaysMin: formData.transitDaysMin,
        transitDaysMax: formData.transitDaysMax,
        maxWeightG,
        extraWeightPerKgCents,
        freeShippingThresholdCents,
        tenantId: formData.tenantId,
      };

      const res = await dispatch(
        saveShippingRate({ id: editingRate?.id, data: payload }),
      );
      if (res.meta.requestStatus === "fulfilled") {
        setModalOpen(false);
        loadData();
      } else {
        setFormError((res.payload as string) || "Failed to save shipping rate");
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save shipping rate";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!rateToDelete?.id) return;
    setDeleting(true);
    try {
      const res = await dispatch(deleteShippingRate(rateToDelete.id));
      if (res.meta.requestStatus === "fulfilled") {
        setDeleteConfirmOpen(false);
        setRateToDelete(null);
        loadData();
      }
    } finally {
      setDeleting(false);
    }
  };

  // Warehouse CRUD handler
  const handleSaveWarehouse = async (e: React.FormEvent) => {
    e.preventDefault();
    setWhError(null);
    setWhSaving(true);
    try {
      const originCep = cleanCepDigits(whFormData.originCep);
      if (originCep.length !== 8) {
        setWhError("O CEP de origem deve ter 8 dígitos válidos.");
        setWhSaving(false);
        return;
      }
      const payload: WarehouseInput = {
        name: whFormData.name.trim(),
        originCep,
        city: whFormData.city.trim(),
        uf: whFormData.uf.trim().toUpperCase(),
        street: whFormData.street.trim() || null,
        number: whFormData.number.trim() || null,
        district: whFormData.district.trim() || null,
        isDefault: whFormData.isDefault,
        active: true,
        tenantId: user?.tenantId || null,
      };
      await warehouseService.create(payload);
      setWhFormData({
        name: "",
        originCep: "",
        city: "São Paulo",
        uf: "SP",
        street: "",
        number: "",
        district: "",
        isDefault: false,
      });
      loadWarehouses();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Falha ao cadastrar armazém.";
      setWhError(msg);
    } finally {
      setWhSaving(false);
    }
  };

  const handleDeleteWarehouse = async (id: number) => {
    if (!confirm("Excluir este armazém?")) return;
    try {
      await warehouseService.delete(id);
      loadWarehouses();
    } catch (err) {
      alert("Erro ao excluir armazém.");
    }
  };

  // Populate SP standard example matrix
  const handleApplySpPreset = async () => {
    if (
      !confirm(
        t(
          "operations.shippingRates.presetConfirm",
          "Deseja carregar a tabela padrão sugerida (SP Capital, SP Interior, RJ/MG/ES, Sul, Centro-Oeste, Nordeste Capitais/Interior, Norte)?",
        )
      )
    ) {
      return;
    }

    setPresetLoading(true);
    try {
      const defaultWh = warehouses.find((w) => w.isDefault);
      const whId = defaultWh?.id || null;

      const presetItems: ShippingRateInput[] = [
        {
          originWarehouseId: whId,
          regionName: "SP (Capital e Grande SP)",
          uf: "SP",
          destinationCepStart: "01000000",
          destinationCepEnd: "09999999",
          priceCents: 1290,
          transitDaysMin: 1,
          transitDaysMax: 2,
          maxWeightG: 2000,
          extraWeightPerKgCents: 500,
          freeShippingThresholdCents: 19900,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "SP (Interior e Litoral)",
          uf: "SP",
          destinationCepStart: "11000000",
          destinationCepEnd: "19999999",
          priceCents: 1690,
          transitDaysMin: 2,
          transitDaysMax: 4,
          maxWeightG: 2000,
          extraWeightPerKgCents: 500,
          freeShippingThresholdCents: 24900,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "RJ, MG, ES (Sudeste)",
          uf: "RJ",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 2290,
          transitDaysMin: 3,
          transitDaysMax: 6,
          maxWeightG: 2000,
          extraWeightPerKgCents: 600,
          freeShippingThresholdCents: 29900,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "PR, SC, RS (Sul)",
          uf: "PR",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 2890,
          transitDaysMin: 4,
          transitDaysMax: 7,
          maxWeightG: 2000,
          extraWeightPerKgCents: 700,
          freeShippingThresholdCents: 34900,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "DF, GO, MT, MS (Centro-Oeste)",
          uf: "DF",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 3490,
          transitDaysMin: 6,
          transitDaysMax: 10,
          maxWeightG: 2000,
          extraWeightPerKgCents: 800,
          freeShippingThresholdCents: null,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "Capitais do Nordeste",
          uf: "BA",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 4290,
          transitDaysMin: 8,
          transitDaysMax: 12,
          maxWeightG: 2000,
          extraWeightPerKgCents: 900,
          freeShippingThresholdCents: null,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "Interior do Nordeste",
          uf: "PE",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 5490,
          transitDaysMin: 10,
          transitDaysMax: 15,
          maxWeightG: 2000,
          extraWeightPerKgCents: 1100,
          freeShippingThresholdCents: null,
          tenantId: user?.tenantId || null,
        },
        {
          originWarehouseId: whId,
          regionName: "Região Norte",
          uf: "AM",
          destinationCepStart: null,
          destinationCepEnd: null,
          priceCents: 6890,
          transitDaysMin: 12,
          transitDaysMax: 20,
          maxWeightG: 2000,
          extraWeightPerKgCents: 1400,
          freeShippingThresholdCents: null,
          tenantId: user?.tenantId || null,
        },
      ];

      for (const item of presetItems) {
        await dispatch(saveShippingRate({ data: item }));
      }
      loadData();
    } finally {
      setPresetLoading(false);
    }
  };

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const warehouseName = (id?: number | null) => {
    if (!id) return t("operations.shippingRates.allWarehouses", "Todos / Padrão");
    const found = warehouses.find((w) => w.id === id);
    return found ? `${found.name} (${formatCep(found.originCep)})` : `#${id}`;
  };

  const columns: ColumnDef<ShippingRate, unknown>[] = [
    {
      header: t("operations.shippingRates.regionCol", "Região / Nome"),
      accessorKey: "regionName",
      enableSorting: true,
      cell: ({ row }) => {
        const name = row.original.regionName;
        return (
          <div>
            <div className="font-semibold text-gray-900 dark:text-white">
              {name || "—"}
            </div>
            {row.original.originWarehouseId ? (
              <span className="text-xs text-gray-500 flex items-center gap-1 mt-0.5">
                <WarehouseIcon size={12} className="text-brand-500" />
                {warehouseName(row.original.originWarehouseId)}
              </span>
            ) : null}
          </div>
        );
      },
    },
    {
      header: t("operations.shippingRates.ufCol", "UF"),
      accessorKey: "uf",
      enableSorting: true,
      cell: ({ getValue }) => (
        <Badge variant="light" color="primary" size="sm">
          {getValue<string>()}
        </Badge>
      ),
    },
    {
      header: t("operations.shippingRates.cepRangeCol", "Faixa de CEP"),
      id: "cepRange",
      cell: ({ row }) => {
        const start = row.original.destinationCepStart;
        const end = row.original.destinationCepEnd;
        if (start && end) {
          return (
            <span className="font-mono text-xs text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 px-2 py-0.5 rounded">
              {formatCep(start)} ~ {formatCep(end)}
            </span>
          );
        }
        return (
          <span className="text-xs text-gray-400">
            {t("operations.shippingRates.allUf", "Todo o Estado")}
          </span>
        );
      },
    },
    {
      header: t("operations.shippingRates.priceCol", "Valor Base"),
      accessorKey: "priceCents",
      enableSorting: true,
      cell: ({ getValue }) => {
        const cents = getValue<number>() || 0;
        return (
          <span className="font-semibold text-brand-600 dark:text-brand-400">
            {(cents / 100).toLocaleString("pt-BR", {
              style: "currency",
              currency: "BRL",
            })}
          </span>
        );
      },
    },
    {
      header: t("operations.shippingRates.transitDaysCol", "Prazo Médio"),
      id: "transitDays",
      cell: ({ row }) => {
        const min = row.original.transitDaysMin ?? 1;
        const max = row.original.transitDaysMax ?? 2;
        return (
          <div className="flex items-center gap-1.5 text-xs text-gray-700 dark:text-gray-300">
            <Clock size={13} className="text-gray-400" />
            <span>
              {min === max ? `${min}` : `${min} a ${max}`} dias úteis
            </span>
          </div>
        );
      },
    },
    {
      header: t("operations.shippingRates.weightLimitCol", "Peso Base"),
      accessorKey: "maxWeightG",
      cell: ({ getValue, row }) => {
        const val = getValue<number | null>();
        const extraCents = row.original.extraWeightPerKgCents;
        return (
          <div className="text-xs">
            {val ? (
              <span className="font-medium text-gray-900 dark:text-gray-200">
                Até {(val / 1000).toLocaleString("pt-BR")} kg
              </span>
            ) : (
              <span className="text-gray-400">Sem limite</span>
            )}
            {extraCents ? (
              <div className="text-theme-xs text-amber-600 dark:text-amber-400 mt-0.5">
                +{(extraCents / 100).toLocaleString("pt-BR", {
                  style: "currency",
                  currency: "BRL",
                })}/kg
              </div>
            ) : null}
          </div>
        );
      },
    },
    {
      header: t("operations.shippingRates.freeShippingCol", "Frete Grátis"),
      accessorKey: "freeShippingThresholdCents",
      cell: ({ getValue }) => {
        const cents = getValue<number | null>();
        if (!cents) return <span className="text-xs text-gray-400">—</span>;
        return (
          <Badge variant="light" color="success" size="sm">
            Acima de{" "}
            {(cents / 100).toLocaleString("pt-BR", {
              style: "currency",
              currency: "BRL",
            })}
          </Badge>
        );
      },
    },
    ...(isSysAdmin
      ? [
          {
            header: t("common.tenant", "Tenant"),
            id: "tenant",
            cell: ({ row }: { row: { original: ShippingRate } }) =>
              tenantName(row.original.tenantId),
          },
        ]
      : []),
    {
      id: "actions",
      header: "",
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end gap-1">
          <button
            type="button"
            title={t("common.edit", "Edit")}
            aria-label={t("common.edit", "Edit")}
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openEditModal(row.original)}
          >
            <Pencil size={15} />
          </button>
          <button
            type="button"
            title={t("common.delete", "Delete")}
            aria-label={t("common.delete", "Delete")}
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-error-600 hover:bg-error-50 dark:text-gray-400 dark:hover:text-error-400 dark:hover:bg-error-500/10 transition-colors"
            onClick={() => {
              setRateToDelete(row.original);
              setDeleteConfirmOpen(true);
            }}
          >
            <Trash2 size={15} />
          </button>
        </div>
      ),
    },
  ];

  return (
    <>
      <PageMeta
        title={`${t("operations.shippingRates.title", "Shipping Rates")} | Veche`}
        description={t(
          "operations.shippingRates.desc",
          "Manage regional shipping rates by state/UF, CEP ranges, and weights"
        )}
      />
      <PageBreadcrumb pageTitle={t("operations.shippingRates.title", "Shipping Rates")} />
      <ComponentCard>
        <DataGrid
          data={rates}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<WarehouseIcon size={14} />}
                onClick={() => setWarehouseModalOpen(true)}
              >
                {t("operations.shippingRates.manageWarehouses", "Centros de Distribuição (CDs)")}
              </Button>
              <Button
                size="sm"
                variant="outline"
                startIcon={<Sparkles size={14} className="text-amber-500" />}
                onClick={handleApplySpPreset}
                disabled={presetLoading}
              >
                {presetLoading
                  ? t("common.loading", "Carregando…")
                  : t("operations.shippingRates.applyPreset", "Preencher Exemplo SP")}
              </Button>
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadData}
              >
                {t("common.refresh", "Refresh")}
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateModal}
              >
                {t("operations.shippingRates.addRate", "Add Rate")}
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && rates.length === 0}
          error={error}
          emptyMessage={t("operations.shippingRates.emptyMessage", "No shipping rates found.")}
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

      {/* Shipping Rate Add/Edit Modal */}
      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-2xl p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <Truck size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingRate
                ? t("operations.shippingRates.editRate", "Edit Shipping Rate")
                : t("operations.shippingRates.newRate", "Add Shipping Rate")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t(
                "operations.shippingRates.modalDesc",
                "Configure regional pricing, CEP ranges, and weight excess thresholds."
              )}
            </p>
          </div>
        </div>

        {formError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
            {formError}
          </div>
        )}

        <form onSubmit={handleSave} className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="regionName">
                {t("operations.shippingRates.regionLabel", "Nome da Região / Serviço")}
              </Label>
              <Input
                id="regionName"
                type="text"
                placeholder={t(
                  "operations.shippingRates.regionPlaceholder",
                  "Ex: SP (Capital e Grande SP)"
                )}
                value={formData.regionName}
                onChange={(e) => setFormData({ ...formData, regionName: e.target.value })}
              />
            </div>

            <div>
              <Label htmlFor="originWarehouse">
                {t("operations.shippingRates.warehouseLabel", "CD de Origem (Armazém)")}
              </Label>
              <select
                id="originWarehouse"
                value={formData.originWarehouseId || ""}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    originWarehouseId: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="">{t("operations.shippingRates.allWarehouses", "Qualquer CD / Padrão da Loja")}</option>
                {warehouses.map((wh) => (
                  <option key={wh.id} value={wh.id}>
                    {wh.name} ({formatCep(wh.originCep)} - {wh.city}/{wh.uf})
                    {wh.isDefault ? " [Padrão]" : ""}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <Label htmlFor="uf">{t("operations.shippingRates.ufLabel", "Destination State (UF)")} *</Label>
              <select
                id="uf"
                value={formData.uf}
                onChange={(e) => setFormData({ ...formData, uf: e.target.value })}
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
                required
              >
                {BRAZIL_UFS.map((uf) => (
                  <option key={uf} value={uf}>
                    {uf}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <Label htmlFor="cepStart">
                {t("operations.shippingRates.cepStartLabel", "CEP Inicial")}
              </Label>
              <Input
                id="cepStart"
                type="text"
                placeholder="01000-000"
                value={formData.destinationCepStart}
                onChange={(e) =>
                  setFormData({ ...formData, destinationCepStart: e.target.value })
                }
              />
            </div>

            <div>
              <Label htmlFor="cepEnd">
                {t("operations.shippingRates.cepEndLabel", "CEP Final")}
              </Label>
              <Input
                id="cepEnd"
                type="text"
                placeholder="05999-999"
                value={formData.destinationCepEnd}
                onChange={(e) =>
                  setFormData({ ...formData, destinationCepEnd: e.target.value })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <Label htmlFor="price">
                {t("operations.shippingRates.priceLabel", "Valor Base (R$)")} *
              </Label>
              <Input
                id="price"
                type="number"
                step={0.01}
                min="0"
                placeholder="12.90"
                value={formData.price}
                onChange={(e) => setFormData({ ...formData, price: e.target.value })}
                required
              />
            </div>

            <div>
              <Label htmlFor="transitMin">
                {t("operations.shippingRates.transitMinLabel", "Prazo Mínimo (Dias)")}
              </Label>
              <Input
                id="transitMin"
                type="number"
                min="0"
                value={formData.transitDaysMin}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    transitDaysMin: Math.max(0, parseInt(e.target.value || "1", 10)),
                  })
                }
              />
            </div>

            <div>
              <Label htmlFor="transitMax">
                {t("operations.shippingRates.transitMaxLabel", "Prazo Máximo (Dias)")}
              </Label>
              <Input
                id="transitMax"
                type="number"
                min="0"
                value={formData.transitDaysMax}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    transitDaysMax: Math.max(0, parseInt(e.target.value || "2", 10)),
                  })
                }
              />
            </div>
          </div>

          {/* Weight excess and free shipping thresholds */}
          <div className="p-4 rounded-xl bg-gray-50 dark:bg-gray-800/40 border border-gray-100 dark:border-gray-800 space-y-4">
            <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
              Regras de Peso e Frete Grátis
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <Label htmlFor="maxWeight">
                  {t("operations.shippingRates.maxWeightLabel", "Peso Base (kg)")}
                </Label>
                <Input
                  id="maxWeight"
                  type="number"
                  step={0.1}
                  min="0"
                  placeholder="2.0"
                  value={formData.maxWeightKg}
                  onChange={(e) =>
                    setFormData({ ...formData, maxWeightKg: e.target.value })
                  }
                />
              </div>

              <div>
                <Label htmlFor="extraWeight">
                  {t("operations.shippingRates.extraWeightLabel", "Adicional / kg (R$)")}
                </Label>
                <Input
                  id="extraWeight"
                  type="number"
                  step={0.01}
                  min="0"
                  placeholder="5.00"
                  value={formData.extraWeightPerKg}
                  onChange={(e) =>
                    setFormData({ ...formData, extraWeightPerKg: e.target.value })
                  }
                />
              </div>

              <div>
                <Label htmlFor="freeShipping">
                  {t("operations.shippingRates.freeShippingLabel", "Frete Grátis a partir de (R$)")}
                </Label>
                <Input
                  id="freeShipping"
                  type="number"
                  step={0.01}
                  min="0"
                  placeholder="199.00"
                  value={formData.freeShippingThreshold}
                  onChange={(e) =>
                    setFormData({ ...formData, freeShippingThreshold: e.target.value })
                  }
                />
              </div>
            </div>
          </div>

          {isSysAdmin && (
            <div>
              <Label htmlFor="tenantId">{t("common.tenant", "Tenant")}</Label>
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
                <option value="">{t("customers.defaultNone", "Default / None")}</option>
                {tenants.map((tTenant) => (
                  <option key={tTenant.id} value={tTenant.id ?? ""}>
                    {tTenant.businessName || tTenant.companyName}
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
              {t("common.cancel", "Cancel")}
            </Button>
            <Button type="submit" disabled={saving}>
              {saving
                ? t("common.saving", "Saving…")
                : editingRate
                ? t("operations.shippingRates.saveRate", "Update Rate")
                : t("operations.shippingRates.createRate", "Save Rate")}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Delete Rate Confirmation Modal */}
      <Modal
        isOpen={deleteConfirmOpen}
        onClose={() => setDeleteConfirmOpen(false)}
        className="max-w-sm p-6"
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="p-2.5 rounded-xl bg-error-50 text-error-600 dark:bg-error-500/10 dark:text-error-400">
            <Trash2 size={20} />
          </div>
          <div>
            <h3 className="text-base font-semibold text-gray-900 dark:text-white">
              {t("operations.shippingRates.deleteConfirmTitle", "Excluir Taxa de Frete")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {t(
                "operations.shippingRates.deleteConfirmDesc",
                "Tem certeza de que deseja excluir esta taxa regional de frete?"
              )}
            </p>
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <Button
            type="button"
            variant="outline"
            onClick={() => setDeleteConfirmOpen(false)}
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            type="button"
            className="bg-error-600 hover:bg-error-700 text-white"
            onClick={handleDelete}
            disabled={deleting}
          >
            {deleting ? t("common.deleting", "Excluindo…") : t("common.delete", "Delete")}
          </Button>
        </div>
      </Modal>

      {/* Warehouses Management Modal */}
      <Modal
        isOpen={warehouseModalOpen}
        onClose={() => setWarehouseModalOpen(false)}
        className="max-w-3xl p-6"
      >
        <div className="flex items-center justify-between gap-3 mb-5 border-b border-gray-100 dark:border-gray-800 pb-4">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
              <WarehouseIcon size={20} />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Centros de Distribuição & Armazéns (CDs)
              </h3>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Cadastre seus centros de distribuição com CEP de origem para cotação e expedição de produtos.
              </p>
            </div>
          </div>
        </div>

        {/* Existing Warehouses List */}
        <div className="mb-6">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-3">
            Armazéns Cadastrados ({warehouses.length})
          </h4>
          {warehousesLoading ? (
            <div className="text-sm text-gray-500">Carregando armazéns…</div>
          ) : warehouses.length === 0 ? (
            <div className="text-sm text-gray-400 italic p-4 rounded-lg bg-gray-50 dark:bg-gray-800/40 border border-dashed border-gray-200 dark:border-gray-700 text-center">
              Nenhum CD cadastrado ainda. Use o formulário abaixo para adicionar o primeiro.
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-56 overflow-y-auto">
              {warehouses.map((wh) => (
                <div
                  key={wh.id}
                  className="p-3.5 rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex items-start justify-between"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-sm text-gray-900 dark:text-white">
                        {wh.name}
                      </span>
                      {wh.isDefault ? (
                        <Badge variant="solid" color="primary" size="sm">
                          Padrão
                        </Badge>
                      ) : null}
                    </div>
                    <div className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1 font-mono">
                      <MapPin size={12} className="text-brand-500" />
                      CEP Origem: {formatCep(wh.originCep)}
                    </div>
                    <div className="text-xs text-gray-500">
                      {wh.city} - {wh.uf} {wh.street ? `(${wh.street}${wh.number ? `, ${wh.number}` : ""})` : ""}
                    </div>
                  </div>
                  <button
                    type="button"
                    title="Excluir Armazém"
                    className="text-gray-400 hover:text-error-600 p-1 transition-colors"
                    onClick={() => handleDeleteWarehouse(wh.id)}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Add New Warehouse Form */}
        <div className="p-4 rounded-xl bg-gray-50 dark:bg-gray-800/40 border border-gray-100 dark:border-gray-800">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-600 dark:text-gray-300 mb-3 flex items-center gap-1.5">
            <Plus size={14} /> Novo Centro de Distribuição
          </h4>

          {whError && (
            <div className="mb-3 rounded-lg border-error-200 bg-error-50 p-2.5 text-xs text-error-600 border dark:bg-error-500/10 dark:border-error-500/20">
              {whError}
            </div>
          )}

          <form onSubmit={handleSaveWarehouse} className="space-y-3">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div className="md:col-span-2">
                <Label htmlFor="whName">Nome do CD / Armazém *</Label>
                <Input
                  id="whName"
                  placeholder="Ex: CD Matriz São Paulo"
                  value={whFormData.name}
                  onChange={(e) => setWhFormData({ ...whFormData, name: e.target.value })}
                  required
                />
              </div>
              <div>
                <Label htmlFor="whCep">CEP de Origem (8 dígitos) *</Label>
                <Input
                  id="whCep"
                  placeholder="01310-100"
                  value={whFormData.originCep}
                  onChange={(e) => setWhFormData({ ...whFormData, originCep: e.target.value })}
                  required
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div>
                <Label htmlFor="whCity">Cidade *</Label>
                <Input
                  id="whCity"
                  placeholder="São Paulo"
                  value={whFormData.city}
                  onChange={(e) => setWhFormData({ ...whFormData, city: e.target.value })}
                  required
                />
              </div>
              <div>
                <Label htmlFor="whUf">UF *</Label>
                <select
                  id="whUf"
                  value={whFormData.uf}
                  onChange={(e) => setWhFormData({ ...whFormData, uf: e.target.value })}
                  className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
                  required
                >
                  {BRAZIL_UFS.map((uf) => (
                    <option key={uf} value={uf}>
                      {uf}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex items-center gap-2 pt-6">
                <input
                  type="checkbox"
                  id="whDefault"
                  checked={whFormData.isDefault}
                  onChange={(e) =>
                    setWhFormData({ ...whFormData, isDefault: e.target.checked })
                  }
                  className="rounded border-gray-300 text-brand-600 focus:ring-brand-500 h-4 w-4"
                />
                <Label htmlFor="whDefault" className="cursor-pointer mb-0">
                  Armazém Padrão
                </Label>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button type="submit" size="sm" disabled={whSaving}>
                {whSaving ? "Salvando…" : "Adicionar CD"}
              </Button>
            </div>
          </form>
        </div>

        <div className="mt-5 flex justify-end">
          <Button
            type="button"
            variant="outline"
            onClick={() => setWarehouseModalOpen(false)}
          >
            Fechar
          </Button>
        </div>
      </Modal>
    </>
  );
}

export default ShippingRates;
