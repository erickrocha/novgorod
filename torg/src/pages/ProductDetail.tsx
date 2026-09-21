import React, { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchProductBySlug, fetchFeaturedProducts } from '../store/slices/catalogSlice';
import { addToCart } from '../store/slices/cartSlice';
import { addToast } from '../store/slices/uiSlice';
import ProductCard from '../components/product/ProductCard';
import Button from '../components/common/Button';
import Badge from '../components/common/Badge';
import {
  Star,
  ShoppingBag,
  Truck,
  ShieldCheck,
  RotateCcw,
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
        <div className="h-4 bg-slate-200 rounded-lg w-1/4 mx-auto" />
      </div>
    );
  }

  if (!selectedProduct) {
    return (
      <div className="max-w-7xl mx-auto px-4 py-20 text-center">
        <h2 className="text-2xl font-bold text-slate-800">Produto não encontrado</h2>
        <p className="text-slate-500 mt-2 mb-6">O item solicitado não está disponível no mercado.</p>
        <Link to="/catalogo">
          <Button variant="primary">Voltar para a Vitrine</Button>
        </Link>
      </div>
    );
  }

  const currentVariant = selectedProduct.variants?.[selectedVariantIndex];
  const activePrice = currentVariant ? currentVariant.price : selectedProduct.price;

  const handleAddToCart = () => {
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
    dispatch(
      addToCart({
        product: selectedProduct,
        variant: currentVariant,
        quantity,
      })
    );
    navigate('/checkout');
  };

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-16">
      {/* Breadcrumb Navigation */}
      <div className="flex items-center gap-2 text-xs font-semibold text-slate-500">
        <Link to="/" className="hover:text-amber-700 transition-colors">
          Início
        </Link>
        <span>/</span>
        <Link to="/catalogo" className="hover:text-amber-700 transition-colors">
          Catálogo
        </Link>
        <span>/</span>
        <span className="text-slate-900 truncate max-w-xs">{selectedProduct.name}</span>
      </div>

      {/* Main Product Info Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-start">
        {/* Gallery */}
        <div className="lg:col-span-6 space-y-4">
          <div className="relative aspect-square w-full bg-slate-100 rounded-3xl overflow-hidden border border-slate-200/80 shadow-xs">
            <img
              src={selectedProduct.images[activeImageIndex] || selectedProduct.thumbnail}
              alt={selectedProduct.name}
              className="w-full h-full object-cover"
            />
            {selectedProduct.originalPrice && (
              <div className="absolute top-4 left-4">
                <Badge variant="rose" size="md">
                  Oferta de Mercado
                </Badge>
              </div>
            )}
          </div>

          {/* Thumbnails */}
          {selectedProduct.images.length > 1 && (
            <div className="flex gap-3">
              {selectedProduct.images.map((img, idx) => (
                <button
                  key={idx}
                  onClick={() => setActiveImageIndex(idx)}
                  className={`w-20 h-20 rounded-xl overflow-hidden border-2 transition-all cursor-pointer ${
                    activeImageIndex === idx
                      ? 'border-amber-600 scale-95 shadow-sm'
                      : 'border-slate-200 opacity-70 hover:opacity-100'
                  }`}
                >
                  <img src={img} alt="Miniatura" className="w-full h-full object-cover" />
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Product Details & Actions */}
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

            {selectedProduct.subtitle && (
              <p className="text-base text-slate-500 mt-2 font-medium">
                {selectedProduct.subtitle}
              </p>
            )}

            {/* Rating */}
            <div className="flex items-center gap-3 mt-4">
              <div className="flex items-center text-amber-500">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Star
                    key={i}
                    className={`w-4 h-4 ${
                      i < Math.floor(selectedProduct.rating)
                        ? 'fill-amber-400'
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

          {/* Pricing */}
          <div className="p-6 bg-slate-50 rounded-2xl border border-slate-200/80 flex items-baseline gap-4">
            <div className="text-3xl font-black text-slate-950">
              R$ {activePrice.toFixed(2).replace('.', ',')}
            </div>
            {selectedProduct.originalPrice && (
              <div className="text-sm line-through text-slate-400">
                R$ {selectedProduct.originalPrice.toFixed(2).replace('.', ',')}
              </div>
            )}
            <span className="text-xs font-semibold text-emerald-700 bg-emerald-100/80 px-2.5 py-1 rounded-full ml-auto">
              Em Estoque ({selectedProduct.stock} disponíveis)
            </span>
          </div>

          {/* Description */}
          <p className="text-sm sm:text-base text-slate-600 leading-relaxed">
            {selectedProduct.description}
          </p>

          {/* Variants Selector */}
          {selectedProduct.variants && selectedProduct.variants.length > 0 && (
            <div className="space-y-2">
              <label className="text-xs uppercase font-bold text-slate-500">
                Formato / Opção:
              </label>
              <div className="flex flex-wrap gap-2">
                {selectedProduct.variants.map((variant, idx) => (
                  <button
                    key={variant.id}
                    onClick={() => setSelectedVariantIndex(idx)}
                    className={`px-4 py-2.5 rounded-xl text-xs font-bold border transition-all cursor-pointer ${
                      selectedVariantIndex === idx
                        ? 'border-amber-600 bg-amber-50/80 text-amber-900 shadow-xs'
                        : 'border-slate-200 text-slate-700 hover:border-slate-300'
                    }`}
                  >
                    {variant.name} — R$ {variant.price.toFixed(2).replace('.', ',')}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Quantity and Actions */}
          <div className="space-y-3 pt-4 border-t border-slate-200">
            <div className="flex items-center gap-4">
              <div className="flex items-center border border-slate-300 rounded-xl overflow-hidden bg-white">
                <button
                  onClick={() => setQuantity(Math.max(1, quantity - 1))}
                  className="px-4 py-3 text-slate-600 hover:bg-slate-100 font-bold transition-colors cursor-pointer"
                >
                  -
                </button>
                <span className="px-4 text-sm font-bold text-slate-900">
                  {quantity}
                </span>
                <button
                  onClick={() => setQuantity(quantity + 1)}
                  className="px-4 py-3 text-slate-600 hover:bg-slate-100 font-bold transition-colors cursor-pointer"
                >
                  +
                </button>
              </div>

              <Button
                onClick={handleAddToCart}
                variant="primary"
                size="lg"
                className="flex-1 gap-2 text-base font-bold shadow-md"
              >
                <ShoppingBag className="w-5 h-5" />
                <span>Adicionar ao Carrinho</span>
              </Button>
            </div>

            <Button
              onClick={handleBuyNow}
              variant="secondary"
              size="lg"
              className="w-full text-base font-bold"
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

      {/* Specifications Table */}
      {selectedProduct.attributes && (
        <div className="bg-white rounded-3xl border border-slate-200/80 p-8 shadow-xs">
          <h3 className="text-xl font-bold text-slate-900 mb-6">
            Ficha Técnica & Especificações
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            {Object.entries(selectedProduct.attributes).map(([key, val]) => (
              <div key={key} className="flex justify-between py-2 border-b border-slate-100">
                <span className="font-semibold text-slate-500 capitalize">{key}:</span>
                <span className="font-bold text-slate-800">{String(val)}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Related Products */}
      <div className="space-y-6">
        <h3 className="text-2xl font-black text-slate-900">
          Você também pode gostar
        </h3>
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
