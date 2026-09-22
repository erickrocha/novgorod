import { useCallback, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Megaphone, Pencil, Plus, RefreshCw, Target } from "lucide-react";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Switch from "@/components/form/switch/Switch";
import { Modal } from "@/components/ui/modal";
import DataGrid from "@/components/data-grid/DataGrid";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchTenants } from "@/store/tenantSlice";
import {
  createTarget,
  fetchCampaignsPaged,
  fetchTargetsPaged,
  saveCampaign,
} from "@/store/marketingSlice";
import type {
  Campaign,
  CampaignInput,
  CampaignTargetInput,
  PageQueryParams,
} from "@/services/types";
import { ROLES } from "@/utils/enums";

export function CampaignsPage() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const tenants = useAppSelector((s) => s.tenant.tenantsList);
  const { user } = useAppSelector((s) => s.auth);
  const isSysAdmin = user?.role === ROLES.SYS_ADMIN;

  const { campaignsPaged, targetsPaged, loading, error } = useAppSelector(
    (s) => s.marketing,
  );
  const campaigns = campaignsPaged?.items || [];
  const total = campaignsPaged?.total || 0;
  const targets = targetsPaged?.items || [];
  const loadingTargets = loading;

  // Campaign Form Modal
  const [modalOpen, setModalOpen] = useState(false);
  const [editingCampaign, setEditingCampaign] = useState<Campaign | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formData, setFormData] = useState<{
    name: string;
    description: string;
    campaignType: string;
    startsAt: string;
    endsAt: string;
    budgetLimit: string;
    active: boolean;
    tenantId: number | null;
  }>({
    name: "",
    description: "",
    campaignType: "DISCOUNT",
    startsAt: "",
    endsAt: "",
    budgetLimit: "",
    active: true,
    tenantId: user?.tenantId || null,
  });

  // Target Modal State
  const [targetsModalOpen, setTargetsModalOpen] = useState(false);
  const [selectedCampaign, setSelectedCampaign] = useState<Campaign | null>(null);
  const [showAddTarget, setShowAddTarget] = useState(false);
  const [newTarget, setNewTarget] = useState<{ targetType: string; targetId: number }>({
    targetType: "CATEGORY",
    targetId: 1,
  });

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "desc";

  const loadCampaigns = useCallback(() => {
    const params: PageQueryParams = { page, pageSize, q, sortBy, sortDir };
    dispatch(fetchCampaignsPaged(params));
  }, [dispatch, page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    let active = true;
    queueMicrotask(() => {
      if (active) {
        loadCampaigns();
      }
    });
    return () => {
      active = false;
    };
  }, [loadCampaigns]);

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
    setEditingCampaign(null);
    setFormData({
      name: "",
      description: "",
      campaignType: "DISCOUNT",
      startsAt: "",
      endsAt: "",
      budgetLimit: "",
      active: true,
      tenantId: user?.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (camp: Campaign) => {
    setEditingCampaign(camp);
    setFormData({
      name: camp.name,
      description: camp.description || "",
      campaignType: camp.campaignType,
      startsAt: camp.startsAt ? camp.startsAt.slice(0, 16) : "",
      endsAt: camp.endsAt ? camp.endsAt.slice(0, 16) : "",
      budgetLimit: camp.budgetLimitCents ? (camp.budgetLimitCents / 100).toFixed(2) : "",
      active: camp.active,
      tenantId: camp.tenantId || null,
    });
    setFormError(null);
    setModalOpen(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setSaving(true);
    try {
      const budgetLimitCents = formData.budgetLimit
        ? Math.round(parseFloat(formData.budgetLimit) * 100)
        : null;

      const payload: CampaignInput = {
        name: formData.name.trim(),
        description: formData.description.trim() || null,
        campaignType: formData.campaignType,
        startsAt: formData.startsAt ? formData.startsAt + ":00" : null,
        endsAt: formData.endsAt ? formData.endsAt + ":00" : null,
        budgetLimitCents,
        active: formData.active,
        tenantId: formData.tenantId,
      };

      const res = await dispatch(
        saveCampaign({ id: editingCampaign?.id, data: payload }),
      );
      if (res.meta.requestStatus === "fulfilled") {
        setModalOpen(false);
        loadCampaigns();
      } else {
        setFormError((res.payload as string) || "Failed to save campaign");
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to save campaign";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const openTargetsModal = (camp: Campaign) => {
    setSelectedCampaign(camp);
    setShowAddTarget(false);
    setTargetsModalOpen(true);
    dispatch(fetchTargetsPaged({ campaignId: camp.id, pageSize: 50 }));
  };

  const handleAddTarget = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCampaign) return;
    try {
      const payload: CampaignTargetInput = {
        tenantId: selectedCampaign.tenantId,
        campaignId: selectedCampaign.id,
        targetType: newTarget.targetType,
        targetId: Number(newTarget.targetId),
      };
      const res = await dispatch(createTarget(payload));
      if (res.meta.requestStatus === "fulfilled") {
        setShowAddTarget(false);
        dispatch(
          fetchTargetsPaged({
            campaignId: selectedCampaign.id,
            pageSize: 50,
          }),
        );
      } else {
        alert((res.payload as string) || "Failed to add target");
      }
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : "Failed to add target");
    }
  };

  const formatCurrency = (cents: number) =>
    (cents / 100).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });

  const tenantName = (id?: number | null) =>
    tenants.find((t) => t.id === id)?.businessName ||
    tenants.find((t) => t.id === id)?.companyName ||
    `#${id ?? "—"}`;

  const columns: ColumnDef<Campaign, unknown>[] = [
    {
      header: t("marketing.campaigns.nameCol", "Campaign Name"),
      accessorKey: "name",
      enableSorting: true,
      cell: ({ row }) => (
        <div>
          <div className="font-semibold text-gray-900 dark:text-white">
            {row.original.name}
          </div>
          {row.original.description && (
            <div className="text-xs text-gray-500 truncate max-w-xs">
              {row.original.description}
            </div>
          )}
        </div>
      ),
    },
    {
      header: t("marketing.campaigns.typeCol", "Type"),
      accessorKey: "campaignType",
      cell: ({ getValue }) => (
        <span className="text-xs font-mono px-2 py-0.5 rounded bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300">
          {getValue<string>()}
        </span>
      ),
    },
    {
      header: t("marketing.campaigns.budgetCol", "Budget Limit"),
      accessorKey: "budgetLimitCents",
      cell: ({ getValue }) => {
        const c = getValue<number>();
        return c != null ? formatCurrency(c) : "No Limit";
      },
    },
    {
      header: t("marketing.campaigns.statusCol", "Active"),
      accessorKey: "active",
      cell: ({ getValue }) => (
        <Badge size="sm" color={getValue<boolean>() ? "success" : "light"}>
          {getValue<boolean>() ? t("common.active", "Active") : t("common.inactive", "Inactive")}
        </Badge>
      ),
    },
    ...(isSysAdmin
      ? [
          {
            header: t("marketing.campaigns.tenantCol", "Tenant"),
            id: "tenant",
            cell: ({ row }: { row: { original: Campaign } }) =>
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
            title={t("marketing.campaigns.targets", "Targets")}
            aria-label={t("marketing.campaigns.targets", "Targets")}
            className="h-8 w-8 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
            onClick={() => openTargetsModal(row.original)}
          >
            <Target size={15} />
          </button>
          <button
            type="button"
            title={t("common.edit", "Edit")}
            aria-label={t("common.edit", "Edit")}
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
        title={`${t("marketing.campaigns.title", "Marketing Campaigns")} | Veche`}
        description={t("marketing.campaigns.desc", "Create and manage seasonal sales and promotional campaigns")}
      />
      <PageBreadcrumb pageTitle={t("marketing.campaigns.title", "Marketing Campaigns")} />
      <ComponentCard
        title={t("marketing.campaigns.title", "Marketing Campaigns")}
        desc={t("marketing.campaigns.desc", "Create and manage seasonal sales and promotional campaigns")}
      >
        <DataGrid
          data={campaigns}
          columns={columns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={loadCampaigns}
              >
                {t("marketing.campaigns.refresh", t("common.refresh", "Refresh"))}
              </Button>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={openCreateModal}
              >
                {t("marketing.campaigns.addCampaign", "Add Campaign")}
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? index)}
          loading={loading && campaigns.length === 0}
          error={error}
          emptyMessage={t("marketing.campaigns.emptyMessage", "No marketing campaigns found.")}
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

      {/* Campaign Create/Edit Modal */}
      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        className="max-w-lg p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
            <Megaphone size={20} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {editingCampaign
                ? t("marketing.campaigns.editCampaign", "Edit Campaign")
                : t("marketing.campaigns.newCampaign", "Add Campaign")}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t("marketing.campaigns.desc", "Configure promotional campaign limits and timeline.")}
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
            <Label htmlFor="campName">{t("marketing.campaigns.nameCol", "Campaign Name")}</Label>
            <Input
              id="campName"
              placeholder={t("marketing.campaigns.namePlaceholder", "e.g. Black Friday 2026")}
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
          </div>

          <div>
            <Label htmlFor="campDesc">{t("common.description", "Description")}</Label>
            <Input
              id="campDesc"
              placeholder={t("marketing.campaigns.descPlaceholder", "Seasonal discount on selected catalog categories")}
              value={formData.description}
              onChange={(e) =>
                setFormData({ ...formData, description: e.target.value })
              }
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="campType">{t("marketing.campaigns.campaignType", "Campaign Type")}</Label>
              <select
                id="campType"
                value={formData.campaignType}
                onChange={(e) =>
                  setFormData({ ...formData, campaignType: e.target.value })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="DISCOUNT">{t("marketing.campaigns.types.discount", "Direct Discount")}</option>
                <option value="FLASH_SALE">{t("marketing.campaigns.types.flashSale", "Flash Sale")}</option>
                <option value="COUPON_PROGRAM">{t("marketing.campaigns.types.couponProgram", "Coupon Program")}</option>
                <option value="BUNDLE">{t("marketing.campaigns.types.bundle", "Bundle")}</option>
              </select>
            </div>
            <div>
              <Label htmlFor="campBudget">{t("marketing.campaigns.budgetLimit", "Budget Limit (R$)")}</Label>
              <Input
                id="campBudget"
                type="number"
                step={0.01}
                min="0"
                placeholder="5000.00"
                value={formData.budgetLimit}
                onChange={(e) =>
                  setFormData({ ...formData, budgetLimit: e.target.value })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="startsAt">{t("marketing.campaigns.startsAt", "Start Date & Time")}</Label>
              <Input
                id="startsAt"
                type="datetime-local"
                value={formData.startsAt}
                onChange={(e) =>
                  setFormData({ ...formData, startsAt: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="endsAt">{t("marketing.campaigns.endsAt", "End Date & Time")}</Label>
              <Input
                id="endsAt"
                type="datetime-local"
                value={formData.endsAt}
                onChange={(e) =>
                  setFormData({ ...formData, endsAt: e.target.value })
                }
              />
            </div>
          </div>

          <div className="pt-2">
            <Switch
              label={t("marketing.campaigns.campaignActive", "Campaign Active")}
              checked={formData.active}
              onChange={(checked) => setFormData({ ...formData, active: checked })}
            />
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
              {t("common.cancel", "Cancel")}
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? t("common.saving", "Saving…") : t("marketing.campaigns.saveChanges", "Save Campaign")}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Targets Modal */}
      <Modal
        isOpen={targetsModalOpen}
        onClose={() => setTargetsModalOpen(false)}
        className="max-w-md p-6"
      >
        <div className="flex items-center justify-between mb-5">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-500/10 dark:text-brand-400">
              <Target size={20} />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                {t("marketing.campaigns.targetsTitle", "Campaign Targets")}
              </h3>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {t("marketing.campaigns.targetsDesc", "Entities targeted by {name}.", { name: selectedCampaign?.name })}
              </p>
            </div>
          </div>
          {!showAddTarget && (
            <Button
              size="sm"
              startIcon={<Plus size={14} />}
              onClick={() => setShowAddTarget(true)}
            >
              {t("marketing.campaigns.addTarget", "Add Target")}
            </Button>
          )}
        </div>

        {showAddTarget ? (
          <form onSubmit={handleAddTarget} className="space-y-3 pt-2 border-t border-gray-100 dark:border-gray-800">
            <div>
              <Label htmlFor="targetType">{t("marketing.campaigns.targetEntityType", "Target Entity Type")}</Label>
              <select
                id="targetType"
                value={newTarget.targetType}
                onChange={(e) =>
                  setNewTarget({ ...newTarget, targetType: e.target.value })
                }
                className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border bg-white dark:bg-gray-800 dark:border-gray-700 dark:text-white"
              >
                <option value="CATEGORY">{t("marketing.campaigns.targetTypes.category", "Category")}</option>
                <option value="PRODUCT">{t("marketing.campaigns.targetTypes.product", "Product")}</option>
                <option value="CUSTOMER_TIER">{t("marketing.campaigns.targetTypes.customerTier", "Customer Tier")}</option>
              </select>
            </div>
            <div>
              <Label htmlFor="targetId">{t("marketing.campaigns.entityId", "Entity ID")}</Label>
              <Input
                id="targetId"
                type="number"
                min="1"
                value={newTarget.targetId}
                onChange={(e) =>
                  setNewTarget({ ...newTarget, targetId: Number(e.target.value) })
                }
                required
              />
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowAddTarget(false)}
              >
                {t("common.cancel", "Cancel")}
              </Button>
              <Button type="submit">{t("marketing.campaigns.saveTarget", "Save Target")}</Button>
            </div>
          </form>
        ) : (
          <div className="space-y-2 max-h-60 overflow-y-auto">
            {loadingTargets ? (
              <p className="text-sm text-center text-gray-500 py-6">{t("marketing.campaigns.loadingTargets", "Loading targets…")}</p>
            ) : targets.length === 0 ? (
              <p className="text-sm text-center text-gray-500 py-6">
                {t("marketing.campaigns.noTargets", "No targets specified (applies globally).")}
              </p>
            ) : (
              targets.map((tTarget) => (
                <div
                  key={tTarget.id}
                  className="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/30"
                >
                  <span className="text-sm font-semibold text-gray-900 dark:text-white">
                    {tTarget.targetType} #{tTarget.targetId}
                  </span>
                  <span className="text-2xs text-gray-400">
                    {t("marketing.campaigns.addedDate", "Added {date}", {
                      date: tTarget.createdAt ? new Date(tTarget.createdAt).toLocaleDateString("pt-BR") : "—"
                    })}
                  </span>
                </div>
              ))
            )}
          </div>
        )}
      </Modal>
    </>
  );
}

export default CampaignsPage;
