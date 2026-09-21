import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { closeQuickView } from '../../store/slices/uiSlice';
import { addToCart } from '../../store/slices/cartSlice';
import { X, Star, ShoppingBag, ShieldCheck, Truck } from 'lucide-react';
import Button from '../common/Button';
import Badge from '../common/Badge';

export const QuickViewModal: React.FC = () => {
  const dispatch = useAppDispatch();
  const product = useAppSelector((state) => state.ui.quickViewProduct);
  const [selectedVariantIndex, setSelectedVariantIndex] = useState(0);
  const [quantity, setQuantity] = useState(1);

  if (!product) return null;

  const currentVariant = product.variants?.[selectedVariantIndex];
  const activePrice = currentVariant ? currentVariant.price : product.price;

  const handleAddToCart = () => {
    dispatch(
      addToCart({
        product,
        variant: currentVariant,
        quantity,
      })
    );
    dispatch(closeQuickView());
  };

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto flex items-center justify-center p-4">
      <div
        className="fixed inset-0 bg-slate-900/60 backdrop-blur-sm transition-opacity"
        onClick={() => dispatch(closeQuickView())}
      />

      <div className="relative bg-white rounded-3xl max-w-3xl w-full shadow-2xl overflow-hidden border border-slate-100 z-10">
        <button
          onClick={() => dispatch(closeQuickView())}
          className="absolute top-4 right-4 z-20 p-2 text-slate-400 hover:text-slate-600 bg-white/80 backdrop-blur rounded-full hover:bg-white shadow-sm transition-colors cursor-pointer"
          aria-label="Fechar"
        >
          <X className="w-5 h-5" />
        </button>

        <div className="grid grid-cols-1 md:grid-cols-2">
          {/* Image */}
          <div className="relative h-72 md:h-full bg-slate-100">
            <img
              src={product.images[0] || product.thumbnail}
              alt={product.name}
              className="w-full h-full object-cover"
            />
            {product.originalPrice && (
              <div className="absolute top-4 left-4">
                <Badge variant="rose" size="md">
                  Oferta Especial
                </Badge>
              </div>
            )}
          </div>

          {/* Details */}
          <div className="p-6 md:p-8 flex flex-col justify-between">
            <div>
              <div className="flex items-center gap-2 text-xs text-amber-700 font-semibold mb-1">
                <span>{product.category}</span>
                {product.attributes?.origin && (
                  <>
                    <span>•</span>
                    <span className="text-slate-500 font-normal">{product.attributes.origin}</span>
                  </>
                )}
              </div>

              <h2 className="text-xl font-bold text-slate-900 leading-snug">
                {product.name}
              </h2>

              {/* Rating */}
              <div className="flex items-center gap-2 mt-2">
                <div className="flex items-center text-amber-500">
                  <Star className="w-4 h-4 fill-amber-400" />
                  <span className="ml-1 text-sm font-bold text-slate-800">
                    {product.rating}
                  </span>
                </div>
                <span className="text-xs text-slate-400">
                  ({product.reviewCount} avaliações)
                </span>
              </div>

              {/* Price */}
              <div className="mt-4 flex items-baseline gap-3">
                <span className="text-2xl font-black text-slate-900">
                  R$ {activePrice.toFixed(2).replace('.', ',')}
                </span>
                {product.originalPrice && (
                  <span className="text-sm line-through text-slate-400">
                    R$ {product.originalPrice.toFixed(2).replace('.', ',')}
                  </span>
                )}
              </div>

              <p className="text-sm text-slate-600 mt-3 line-clamp-3 leading-relaxed">
                {product.description}
              </p>

              {/* Variants */}
              {product.variants && product.variants.length > 0 && (
                <div className="mt-5">
                  <label className="text-xs font-semibold text-slate-700 block mb-2">
                    Opções disponíveis:
                  </label>
                  <div className="flex flex-wrap gap-2">
                    {product.variants.map((v, idx) => (
                      <button
                        key={v.id}
                        onClick={() => setSelectedVariantIndex(idx)}
                        className={`text-xs px-3 py-1.5 rounded-lg border transition-all cursor-pointer ${
                          selectedVariantIndex === idx
                            ? 'border-amber-600 bg-amber-50 text-amber-900 font-semibold shadow-xs'
                            : 'border-slate-200 text-slate-700 hover:border-slate-300'
                        }`}
                      >
                        {v.name}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="mt-6 pt-4 border-t border-slate-100 space-y-4">
              <div className="flex items-center gap-3">
                <div className="flex items-center border border-slate-200 rounded-xl overflow-hidden">
                  <button
                    onClick={() => setQuantity(Math.max(1, quantity - 1))}
                    className="px-3 py-2 text-slate-600 hover:bg-slate-100 cursor-pointer"
                  >
                    -
                  </button>
                  <span className="px-3 text-sm font-semibold">{quantity}</span>
                  <button
                    onClick={() => setQuantity(quantity + 1)}
                    className="px-3 py-2 text-slate-600 hover:bg-slate-100 cursor-pointer"
                  >
                    +
                  </button>
                </div>

                <Button
                  onClick={handleAddToCart}
                  variant="primary"
                  className="flex-1 gap-2"
                >
                  <ShoppingBag className="w-4 h-4" />
                  <span>Adicionar ao Carrinho</span>
                </Button>
              </div>

              <div className="flex items-center justify-between text-[11px] text-slate-500 pt-1">
                <span className="flex items-center gap-1">
                  <Truck className="w-3.5 h-3.5 text-amber-600" /> Envio rápido
                </span>
                <span className="flex items-center gap-1">
                  <ShieldCheck className="w-3.5 h-3.5 text-emerald-600" /> Garantia Novgorod
                </span>
                <Link
                  to={`/produto/${product.slug}`}
                  onClick={() => dispatch(closeQuickView())}
                  className="text-amber-700 hover:underline font-semibold"
                >
                  Ver Detalhes Completos →
                </Link>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default QuickViewModal;
