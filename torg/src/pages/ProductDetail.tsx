import React, { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchProductBySlug, fetchFeaturedProducts } from '../store/slices/catalogSlice';
import { addToCart } from '../store/slices/cartSlice';
import { addToast } from '../store/slices/uiSlice';
import ProductCard from '../components/product/ProductCard';
import Button from '../components/common/Button';
import Badge from '../components/common/Badge';
import { Breadcrumb } from '../components/common/Breadcrumb';
import {
  Star,
  ShoppingBag,
  Truck,
  ShieldCheck,
  RotateCcw,
  Store,
  MapPin,
  CheckCircle2,
  PackageCheck,
  Wine,
} from 'lucide-react';

export const ProductDetail: React.FC = () => {
  const { slug } = useParams<{ slug: string }>();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const { selectedProduct, featuredProducts, isLoadingProduct } = useAppSelector(
    (state) => state.catalog
  );

  const [activeImageIndex, setActiveImageIndex] = useState(0);
  const [selectedVariantIndex, setSelectedVariantIndex] = useState(0);
  const [quantity, setQuantity] = useState(1);

  useEffect(() => {
    if (slug) {
      dispatch(fetchProductBySlug(slug));
      window.scrollTo(0, 0);
      setActiveImageIndex(0);
      setSelectedVariantIndex(0);
      setQuantity(1);
    }
  }, [slug, dispatch]);

  useEffect(() => {
    if (featuredProducts.length === 0) {
      dispatch(fetchFeaturedProducts());
    }
  }, [dispatch, featuredProducts.length]);

  if (isLoadingProduct) {
    return (
      <div className="max-w-7xl mx-auto px-4 py-20 text-center animate-pulse">
        <div className="h-8 bg-slate-200 rounded-lg w-1/3 mx-auto mb-4" />
        <div className="h-4 bg-slate-200 rounded-lg w-1/4 mx-auto mb-12" />
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 max-w-4xl mx-auto">
          <div className="aspect-square bg-slate-200 rounded-3xl" />
          <div className="space-y-4 text-left">
            <div className="h-6 bg-slate-200 rounded w-3/4" />
            <div className="h-10 bg-slate-200 rounded w-1/2" />
            <div className="h-24 bg-slate-200 rounded w-full" />
          </div>
        </div>
      </div>
    );
  }

  if (!selectedProduct) {
    return (
      <div className="max-w-7xl mx-auto px-4 py-20 text-center">
        <div className="inline-flex p-4 rounded-full bg-amber-50 text-amber-700 mb-4">
          <Wine className="w-10 h-10" />
        </div>
        <h2 className="text-2xl font-bold text-slate-800">Produto não encontrado</h2>
        <p className="text-slate-500 mt-2 mb-6">O vinho solicitado não está disponível em nossa adega.</p>
        <Link to="/catalogo">
          <Button variant="primary">Voltar para a Vitrine</Button>
        </Link>
      </div>
    );
  }

  // Active SKU / Variant selection
  const currentVariant = selectedProduct.variants?.[selectedVariantIndex];
  const activeSku = selectedProduct.skus?.[selectedVariantIndex];
  const activePrice = currentVariant ? currentVariant.price : selectedProduct.price;
  const activeOriginalPrice = currentVariant?.originalPrice ?? selectedProduct.originalPrice;
  const activeStock = activeSku ? activeSku.stock : (currentVariant?.stock ?? selectedProduct.stock);
  const isOutOfStock = activeStock <= 0;

  // Resolve technical specifications from product and active SKU attributes
  const combinedSpecs: Record<string, string> = {};
  if (selectedProduct.attributes) {
    for (const [key, val] of Object.entries(selectedProduct.attributes)) {
      if (val !== undefined && val !== null) {
        combinedSpecs[key] = String(val);
      }
    }
  }
  if (activeSku?.attributes) {
    for (const attr of activeSku.attributes) {
      combinedSpecs[attr.name] = attr.value;
    }
  }

  const handleAddToCart = () => {
    if (isOutOfStock) return;
    dispatch(
      addToCart({
        product: selectedProduct,
        variant: currentVariant,
        quantity,
      })
    );
    dispatch(
      addToast({
        type: 'success',
        message: `${selectedProduct.name} foi adicionado ao carrinho!`,
      })
    );
  };

  const handleBuyNow = () => {
    if (isOutOfStock) return;
    dispatch(
      addToCart({
        product: selectedProduct,
        variant: currentVariant,
        quantity,
      })
    );
    navigate('/checkout');
  };

  // Seller / Tenant info
  const seller = selectedProduct.seller;
  const sellerLocation = [seller?.locality, seller?.administrativeArea].filter(Boolean).join(' - ');

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-12">
      {/* Breadcrumb Navigation */}
      <Breadcrumb
        items={[
          { label: 'Catálogo', href: '/catalogo' },
          { label: selectedProduct.category, href: `/catalogo?categoria=${selectedProduct.categorySlug}` },
          { label: selectedProduct.name },
        ]}
      />

      {/* Main Product Hero Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-start">
        {/* Left Column: Multi-Image Gallery */}
        <div className="lg:col-span-6 space-y-4">
          <div className="relative aspect-square w-full bg-slate-100 rounded-3xl overflow-hidden border border-slate-200/80 shadow-sm group">
            <img
              src={selectedProduct.images[activeImageIndex] || selectedProduct.thumbnail}
              alt={selectedProduct.name}
              className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
            {activeOriginalPrice && (
              <div className="absolute top-4 left-4">
                <Badge variant="rose" size="md">
                  Oferta de Mercado
                </Badge>
              </div>
            )}
          </div>

          {/* Interactive Thumbnails Rail */}
          {selectedProduct.images.length > 1 && (
            <div className="flex gap-3 overflow-x-auto pb-2">
              {selectedProduct.images.map((img, idx) => (
                <button
                  key={idx}
                  onClick={() => setActiveImageIndex(idx)}
                  className={`w-20 h-20 rounded-xl overflow-hidden border-2 transition-all cursor-pointer shrink-0 ${
                    activeImageIndex === idx
                      ? 'border-amber-600 scale-95 shadow-md ring-2 ring-amber-500/20'
                      : 'border-slate-200 opacity-70 hover:opacity-100 hover:border-slate-300'
                  }`}
                  aria-label={`Ver imagem ${idx + 1}`}
                >
                  <img src={img} alt={`Miniatura ${idx + 1}`} className="w-full h-full object-cover" />
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Right Column: Product Info, Seller Box, SKU & Actions */}
        <div className="lg:col-span-6 space-y-6">
          <div>
            <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-amber-700 mb-2">
              <span>{selectedProduct.category}</span>
              {selectedProduct.attributes?.brand && (
                <>
                  <span>•</span>
                  <span>{selectedProduct.attributes.brand}</span>
                </>
              )}
            </div>

            <h1 className="text-3xl sm:text-4xl font-black text-slate-950 tracking-tight leading-tight">
              {selectedProduct.name}
            </h1>

            {/* Rating and Reviews */}
            <div className="flex items-center gap-3 mt-4">
              <div className="flex items-center text-amber-500">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Star
                    key={i}
                    className={`w-4 h-4 ${
                      i < Math.floor(selectedProduct.rating)
                        ? 'fill-amber-400 text-amber-400'
                        : 'text-slate-300'
                    }`}
                  />
                ))}
              </div>
              <span className="text-sm font-bold text-slate-800">
                {selectedProduct.rating} / 5.0
              </span>
              <span className="text-xs text-slate-400">
                ({selectedProduct.reviewCount} avaliações de clientes verificados)
              </span>
            </div>
          </div>

          {/* Prominent Seller / Tenant Card */}
          <div className="p-4 bg-gradient-to-r from-amber-50/60 to-orange-50/40 rounded-2xl border border-amber-200/80 flex items-center justify-between gap-4">
            <div className="flex items-center gap-3.5">
              <div className="w-12 h-12 rounded-xl bg-amber-600 text-white flex items-center justify-center shrink-0 shadow-xs">
                <Store className="w-6 h-6" />
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-semibold text-slate-500 uppercase tracking-wide">
                    Vendido e entregue por
                  </span>
                  <CheckCircle2 className="w-4 h-4 text-emerald-600" />
                </div>
                <h2 className="text-base font-bold text-slate-900 leading-tight">
                  {seller ? seller.businessName : 'Novgorod Vinhos & Adega'}
                </h2>
                {sellerLocation && (
                  <div className="flex items-center gap-1 text-xs text-slate-500 mt-0.5">
                    <MapPin className="w-3.5 h-3.5 text-amber-700" />
                    <span>{sellerLocation}</span>
                  </div>
                )}
              </div>
            </div>

            <div className="hidden sm:flex flex-col items-end text-right">
              <Badge variant="emerald" size="sm">
                Vendedor Verificado
              </Badge>
              <span className="text-[11px] text-slate-400 mt-1">Envio direto da vinícola</span>
            </div>
          </div>

          {/* Pricing & Stock Card */}
          <div className="p-6 bg-slate-50 rounded-2xl border border-slate-200/80 flex items-baseline gap-4">
            <div className="text-3xl sm:text-4xl font-black text-slate-950">
              R$ {activePrice.toFixed(2).replace('.', ',')}
            </div>
            {activeOriginalPrice && (
              <div className="text-sm line-through text-slate-400">
                R$ {activeOriginalPrice.toFixed(2).replace('.', ',')}
              </div>
            )}
            <span
              className={`text-xs font-semibold px-3 py-1 rounded-full ml-auto ${
                isOutOfStock
                  ? 'text-rose-700 bg-rose-100/80'
                  : 'text-emerald-700 bg-emerald-100/80'
              }`}
            >
              {isOutOfStock ? 'Esgotado' : `Em Estoque (${activeStock} garrafas disponíveis)`}
            </span>
          </div>

          {/* Description Excerpt */}
          {selectedProduct.description && (
            <p className="text-sm sm:text-base text-slate-600 leading-relaxed">
              {selectedProduct.description}
            </p>
          )}

          {/* SKU / Variants Selector */}
          {selectedProduct.variants && selectedProduct.variants.length > 0 && (
            <div className="space-y-3 pt-2">
              <div className="flex items-center justify-between">
                <label className="text-xs uppercase font-bold text-slate-600 tracking-wider">
                  Opção / Formato:
                </label>
                {activeSku?.code && (
                  <span className="text-xs text-slate-400 font-mono">
                    SKU: {activeSku.code}
                  </span>
                )}
              </div>
              <div className="flex flex-wrap gap-2.5">
                {selectedProduct.variants.map((variant, idx) => (
                  <button
                    key={variant.id}
                    onClick={() => {
                      setSelectedVariantIndex(idx);
                      setQuantity(1);
                    }}
                    className={`px-4 py-2.5 rounded-xl text-xs font-bold border transition-all cursor-pointer ${
                      selectedVariantIndex === idx
                        ? 'border-amber-600 bg-amber-50 text-amber-900 shadow-xs ring-1 ring-amber-500/30'
                        : 'border-slate-200 text-slate-700 hover:border-slate-300 bg-white'
                    }`}
                  >
                    <span>{variant.name}</span>
                    <span className="ml-2 font-normal text-slate-500">
                      R$ {variant.price.toFixed(2).replace('.', ',')}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Quantity Selector and Purchase Actions */}
          <div className="space-y-3 pt-4 border-t border-slate-200">
            <div className="flex items-center gap-4">
              <div className="flex items-center border border-slate-300 rounded-xl overflow-hidden bg-white">
                <button
                  onClick={() => setQuantity(Math.max(1, quantity - 1))}
                  disabled={isOutOfStock || quantity <= 1}
                  className="px-4 py-3 text-slate-600 hover:bg-slate-100 font-bold transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  -
                </button>
                <span className="px-4 text-sm font-bold text-slate-900">
                  {quantity}
                </span>
                <button
                  onClick={() => setQuantity(Math.min(activeStock, quantity + 1))}
                  disabled={isOutOfStock || quantity >= activeStock}
                  className="px-4 py-3 text-slate-600 hover:bg-slate-100 font-bold transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  +
                </button>
              </div>

              <Button
                onClick={handleAddToCart}
                variant="primary"
                size="lg"
                disabled={isOutOfStock}
                className="flex-1 gap-2 text-base font-bold shadow-md cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ShoppingBag className="w-5 h-5" />
                <span>{isOutOfStock ? 'Item Indisponível' : 'Adicionar ao Carrinho'}</span>
              </Button>
            </div>

            <Button
              onClick={handleBuyNow}
              variant="secondary"
              size="lg"
              disabled={isOutOfStock}
              className="w-full text-base font-bold cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Comprar Agora com 1 Clique
            </Button>
          </div>

          {/* Value Badges */}
          <div className="grid grid-cols-3 gap-4 pt-4 text-xs text-slate-500 border-t border-slate-100">
            <div className="flex items-center gap-2">
              <Truck className="w-4 h-4 text-amber-600 shrink-0" />
              <span>Envio Climatizado</span>
            </div>
            <div className="flex items-center gap-2">
              <ShieldCheck className="w-4 h-4 text-emerald-600 shrink-0" />
              <span>Selo Novgorod</span>
            </div>
            <div className="flex items-center gap-2">
              <RotateCcw className="w-4 h-4 text-slate-600 shrink-0" />
              <span>Garantia 7 dias</span>
            </div>
          </div>
        </div>
      </div>

      {/* Specifications & Technical Details (Resolved from product_attribute and sku_attribute_value) */}
      {Object.keys(combinedSpecs).length > 0 && (
        <div className="bg-white rounded-3xl border border-slate-200/80 p-8 shadow-xs">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2.5 rounded-xl bg-amber-50 text-amber-800">
              <PackageCheck className="w-6 h-6" />
            </div>
            <div>
              <h3 className="text-xl font-bold text-slate-900">
                Ficha Técnica & Especificações
              </h3>
              <p className="text-xs text-slate-500">
                Características detalhadas certificadas pelo produtor
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-3 text-sm">
            {Object.entries(combinedSpecs).map(([key, val]) => (
              <div key={key} className="flex justify-between py-2.5 border-b border-slate-100">
                <span className="font-medium text-slate-500 capitalize">{key}:</span>
                <span className="font-bold text-slate-800 text-right">{String(val)}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Related Products from Showcase */}
      <div className="space-y-6 pt-4">
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-black text-slate-900 tracking-tight">
            Você também pode gostar
          </h3>
          <Link to="/catalogo" className="text-sm font-bold text-amber-700 hover:text-amber-800">
            Ver todo o catálogo →
          </Link>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {featuredProducts
            .filter((p) => p.id !== selectedProduct.id)
            .slice(0, 4)
            .map((p) => (
              <ProductCard key={p.id} product={p} />
            ))}
        </div>
      </div>
    </div>
  );
};

export default ProductDetail;
