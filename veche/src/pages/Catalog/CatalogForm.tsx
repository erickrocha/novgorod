import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Button from "@/components/ui/button/Button";
import ProductImagesManager from "@/components/catalog/ProductImagesManager";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  fetchCategories,
  fetchProducts,
  saveCategory,
  saveProduct,
  saveSku,
} from "@/store/catalogSlice";
import { fetchTenants } from "@/store/tenantSlice";
import { ROLES } from "@/utils/enums";

export default function CatalogForm() {
  const { kind = "products", id } = useParams();
  const editing = Boolean(id);
  const nav = useNavigate();
  const dispatch = useAppDispatch();
  const catalogState = useAppSelector((s) => s.catalog);
  const tenantsList = useAppSelector((s) => s.tenant.tenantsList);
  const currentUser = useAppSelector((s) => s.auth.user);

  const sys = currentUser?.role === ROLES.SYS_ADMIN;
  const ownTenant = currentUser?.tenantId ?? currentUser?.tenant_id;
  const title = kind[0].toUpperCase() + kind.slice(1);

  const existing: any =
    kind === "categories"
      ? catalogState.categories.find((x) => String(x.id) === id)
      : kind === "skus"
        ? catalogState.skus.find((x) => String(x.id) === id)
        : catalogState.products.find((x) => String(x.id) === id);

  const [form, setForm] = useState<any>({
    tenantId: ownTenant ? String(ownTenant) : "",
    name: "",
    slug: "",
    description: "",
    brand: "",
    active: true,
    ncm: "",
    cest: "",
    origemMercadoria: 0,
    productId: "",
    code: "",
    variantKey: "",
    priceCents: "0",
  });

  useEffect(() => {
    dispatch(fetchTenants());
    if (kind === "categories") dispatch(fetchCategories());
    if (kind === "products" || kind === "skus") dispatch(fetchProducts());
  }, [dispatch, kind]);

  useEffect(() => {
    if (existing) {
      setForm({
        ...form,
        ...existing,
        tenantId: existing.tenantId ? String(existing.tenantId) : "",
        productId: existing.productId ? String(existing.productId) : "",
        priceCents: String(existing.priceCents ?? 0),
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [existing]);

  const setField = (key: string, value: any) =>
    setForm((prev: any) => ({ ...prev, [key]: value }));

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const tenant = sys ? Number(form.tenantId) : ownTenant;
    let result: any;

    if (kind === "categories") {
      result = await dispatch(
        saveCategory({
          id: existing?.id,
          data: {
            tenantId: tenant,
            name: form.name,
            slug: form.slug,
            parentId: form.parentId ? Number(form.parentId) : null,
            active: form.active,
          },
        })
      );
    } else if (kind === "skus") {
      result = await dispatch(
        saveSku({
          id: existing?.id,
          data: {
            tenantId: tenant,
            productId: Number(form.productId),
            code: form.code,
            variantKey: form.variantKey,
            priceCents: Number(form.priceCents),
            compareAtPriceCents: form.compareAtPriceCents
              ? Number(form.compareAtPriceCents)
              : null,
            active: form.active,
          } as any,
        })
      );
    } else {
      result = await dispatch(
        saveProduct({
          id: existing?.id,
          data: {
            tenantId: tenant,
            name: form.name,
            slug: form.slug,
            description: form.description || null,
            brand: form.brand || null,
            active: form.active,
            ncm: form.ncm,
            cest: form.cest || null,
            origemMercadoria: Number(form.origemMercadoria),
          },
        })
      );
    }

    if (result.meta.requestStatus === "fulfilled") {
      nav(`/catalog/${kind}`);
    }
  };

  const tenantOptions = tenantsList.filter((t) => t.id != null);

  return (
    <>
      <PageMeta
        title={`${editing ? "Edit" : "Add"} ${title} | Veche`}
        description="Catalog form"
      />
      <PageBreadcrumb pageTitle={`${editing ? "Edit" : "Add"} ${title}`} />

      <div className="space-y-6">
        <ComponentCard title={`${editing ? "Edit" : "Add"} ${title}`}>
          <form onSubmit={submit} className="space-y-5">
            <div className="grid gap-5 md:grid-cols-2">
              {sys && (
                <div>
                  <Label>Tenant</Label>
                  <input
                    list="catalog-tenants"
                    required
                    value={form.tenantId}
                    onChange={(e) => setField("tenantId", e.target.value)}
                    className="h-11 w-full rounded-lg border border-gray-300 px-4 text-sm dark:border-gray-700 dark:bg-gray-800"
                  />
                  <datalist id="catalog-tenants">
                    {tenantOptions.map((t) => (
                      <option key={t.id} value={String(t.id)}>
                        {t.businessName || t.companyName}
                      </option>
                    ))}
                  </datalist>
                </div>
              )}

              {kind !== "skus" ? (
                <>
                  <div>
                    <Label>Name</Label>
                    <Input
                      required
                      value={form.name}
                      onChange={(e) => setField("name", e.target.value)}
                    />
                  </div>
                  <div>
                    <Label>Slug</Label>
                    <Input
                      required
                      value={form.slug}
                      onChange={(e) => setField("slug", e.target.value)}
                    />
                  </div>
                  {kind === "products" && (
                    <>
                      <div>
                        <Label>Brand</Label>
                        <Input
                          value={form.brand}
                          onChange={(e) => setField("brand", e.target.value)}
                        />
                      </div>
                      <div>
                        <Label>NCM</Label>
                        <Input
                          required
                          value={form.ncm}
                          onChange={(e) => setField("ncm", e.target.value)}
                        />
                      </div>
                      <div className="md:col-span-2">
                        <Label>Description</Label>
                        <Input
                          value={form.description}
                          onChange={(e) => setField("description", e.target.value)}
                        />
                      </div>
                    </>
                  )}
                </>
              ) : (
                <>
                  <div>
                    <Label>Product ID</Label>
                    <Input
                      required
                      type="number"
                      value={form.productId}
                      onChange={(e) => setField("productId", e.target.value)}
                    />
                  </div>
                  <div>
                    <Label>Code</Label>
                    <Input
                      required
                      value={form.code}
                      onChange={(e) => setField("code", e.target.value)}
                    />
                  </div>
                  <div>
                    <Label>Variant key</Label>
                    <Input
                      required
                      value={form.variantKey}
                      onChange={(e) => setField("variantKey", e.target.value)}
                    />
                  </div>
                  <div>
                    <Label>Price (cents)</Label>
                    <Input
                      required
                      type="number"
                      min="0"
                      value={form.priceCents}
                      onChange={(e) => setField("priceCents", e.target.value)}
                    />
                  </div>
                </>
              )}
            </div>

            <label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
              <input
                type="checkbox"
                checked={form.active}
                onChange={(e) => setField("active", e.target.checked)}
              />
              Active
            </label>

            {catalogState.error && (
              <div className="rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600">
                {catalogState.error}
              </div>
            )}

            <div className="flex justify-end gap-3">
              <Link to={`/catalog/${kind}`}>
                <Button variant="outline">Cancel</Button>
              </Link>
              <Button disabled={catalogState.loading}>
                {editing ? "Save changes" : "Create"}
              </Button>
            </div>
          </form>
        </ComponentCard>

        {/* Product Photos Section (Available when editing an existing product) */}
        {kind === "products" && editing && existing?.id && (
          <ComponentCard title="Product Media & Photos">
            <ProductImagesManager productId={Number(existing.id)} />
          </ComponentCard>
        )}
      </div>
    </>
  );
}
