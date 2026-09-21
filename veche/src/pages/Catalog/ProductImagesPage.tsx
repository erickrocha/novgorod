import { useEffect, useState } from "react";
import { useParams, Link, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Edit3, Package, Sparkles } from "lucide-react";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import ProductImagesManager from "@/components/catalog/ProductImagesManager";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchProducts } from "@/store/catalogSlice";
import { type Product } from "@/services/catalogService";

export default function ProductImagesPage() {
  const { id } = useParams<{ id: string }>();
  const { t } = useTranslation();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();

  const catalogState = useAppSelector((state) => state.catalog);
  const productId = Number(id);

  const [product, setProduct] = useState<Product | null>(null);

  useEffect(() => {
    if (catalogState.products.length === 0) {
      dispatch(fetchProducts());
    }
  }, [dispatch, catalogState.products.length]);

  useEffect(() => {
    if (productId && catalogState.products.length > 0) {
      const found = catalogState.products.find((p) => p.id === productId);
      if (found) {
        setProduct(found);
      }
    }
  }, [productId, catalogState.products]);

  const pageTitle = product
    ? `${product.name} — ${t("catalog.photos.title", "Product Photos")}`
    : t("catalog.photos.title", "Product Photos");

  return (
    <>
      <PageMeta
        title={`${pageTitle} | Novgorod`}
        description={t(
          "catalog.photos.desc",
          "Upload and manage high-resolution images for the product catalog."
        )}
      />

      <div className="space-y-6">
        {/* Navigation Breadcrumb */}
        <PageBreadcrumb
          pageTitle={t("catalog.photos.title", "Product Photos")}
          showTitle={false}
          items={[
            {
              label: t("menu.catalog", "Catálogo"),
              href: "/catalog/products",
            },
            {
              label: t("catalog.products.title", "Produtos"),
              href: "/catalog/products",
            },
            ...(product
              ? [
                  {
                    label: product.name,
                    href: `/catalog/products/${product.id}/edit`,
                  },
                ]
              : []),
          ]}
        />

        {/* Product Context Banner */}
        <div className="relative overflow-hidden rounded-2xl border border-gray-200 bg-white p-6 shadow-xs dark:border-gray-800 dark:bg-gray-900">
          {/* Subtle background glow */}
          <div className="pointer-events-none absolute -top-16 -right-16 h-48 w-48 rounded-full bg-brand-500/10 blur-3xl" />

          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-4">
              <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-brand-50 text-brand-600 dark:bg-brand-950/40 dark:text-brand-400">
                <Package size={24} />
              </div>
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <h1 className="text-xl font-bold tracking-tight text-gray-900 dark:text-white">
                    {product ? product.name : `${t("catalog.products.title", "Product")} #${id}`}
                  </h1>
                  {product && (
                    <Badge
                      size="sm"
                      color={product.active ? "success" : "light"}
                    >
                      {product.active
                        ? t("common.active", "Active")
                        : t("common.inactive", "Inactive")}
                    </Badge>
                  )}
                </div>

                <div className="mt-1 flex flex-wrap items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                  {product?.brand && (
                    <span className="inline-flex items-center gap-1 font-medium text-gray-700 dark:text-gray-300">
                      <Sparkles size={13} className="text-amber-500" />
                      {product.brand}
                    </span>
                  )}
                  {product?.slug && (
                    <span>
                      Slug: <code className="rounded bg-gray-100 px-1 py-0.5 font-mono text-[11px] text-gray-700 dark:bg-gray-800 dark:text-gray-300">{product.slug}</code>
                    </span>
                  )}
                  <span>ID: #{productId}</span>
                </div>
              </div>
            </div>

            {/* Quick Action Navigation Buttons */}
            <div className="flex items-center gap-2.5 self-start sm:self-auto">
              <Button
                variant="outline"
                size="sm"
                startIcon={<ArrowLeft size={14} />}
                onClick={() => navigate("/catalog/products")}
              >
                {t("catalog.photos.backToProducts", "Voltar aos Produtos")}
              </Button>
              {productId > 0 && (
                <Link to={`/catalog/products/${productId}/edit`}>
                  <Button
                    variant="outline"
                    size="sm"
                    startIcon={<Edit3 size={14} />}
                  >
                    {t("catalog.photos.editProduct", "Editar Produto")}
                  </Button>
                </Link>
              )}
            </div>
          </div>
        </div>

        {/* Dedicated Photos Management Card */}
        <ComponentCard
          title={t("catalog.photos.title", "Product Photos")}
          desc={t(
            "catalog.photos.desc",
            "Upload single or multiple images directly to S3 with CloudFront CDN distribution."
          )}
        >
          {productId > 0 ? (
            <ProductImagesManager productId={productId} />
          ) : (
            <div className="p-8 text-center text-sm text-gray-500">
              {t("catalog.photos.productNotFound", "Product not found")}
            </div>
          )}
        </ComponentCard>
      </div>
    </>
  );
}
