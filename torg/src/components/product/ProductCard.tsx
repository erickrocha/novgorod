import React from 'react';
import { Link } from 'react-router-dom';
import type { Product } from '../../types';
import { useAppDispatch } from '../../store/hooks';
import { addToCart } from '../../store/slices/cartSlice';
import { openQuickView, addToast } from '../../store/slices/uiSlice';
import { ShoppingBag, Star, Eye } from 'lucide-react';
import Badge from '../common/Badge';

interface ProductCardProps {
  product: Product;
}

export const ProductCard: React.FC<ProductCardProps> = ({ product }) => {
  const dispatch = useAppDispatch();

  const handleAddToCart = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dispatch(addToCart({ product, quantity: 1 }));
    dispatch(addToast({
      type: 'success',
      message: `${product.name} adicionado ao carrinho!`,
    }));
  };

  const handleQuickView = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dispatch(openQuickView(product));
  };

  const discountPercentage = product.originalPrice
    ? Math.round(((product.originalPrice - product.price) / product.originalPrice) * 100)
    : null;

  return (
    <div className="group relative bg-white rounded-2xl border border-slate-200/80 shadow-xs hover:shadow-xl transition-all duration-300 flex flex-col overflow-hidden">
      {/* Product Image Frame */}
      <div className="relative aspect-4/3 w-full bg-slate-100 overflow-hidden">
        <Link to={`/produto/${product.slug}`} className="block w-full h-full">
          <img
            src={product.thumbnail}
            alt={product.name}
            className="w-full h-full object-cover object-center group-hover:scale-105 transition-transform duration-500 ease-out"
            loading="lazy"
          />
        </Link>

        {/* Badges Overlay */}
        <div className="absolute top-3 left-3 flex flex-col gap-1.5 z-10 pointer-events-none">
          {discountPercentage && (
            <Badge variant="rose" size="sm">
              -{discountPercentage}%
            </Badge>
          )}
          {product.isNew && (
            <Badge variant="emerald" size="sm">
              Novidade
            </Badge>
          )}
          {product.isFeatured && !discountPercentage && (
            <Badge variant="amber" size="sm">
              Destaque
            </Badge>
          )}
        </div>

        {/* Quick View Button Hover Action */}
        <div className="absolute inset-0 bg-slate-950/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 pointer-events-none">
          <button
            onClick={handleQuickView}
            className="pointer-events-auto p-3 bg-white/90 hover:bg-white text-slate-800 rounded-full shadow-lg transform translate-y-4 group-hover:translate-y-0 transition-all duration-200 cursor-pointer"
            title="Visualização Rápida"
          >
            <Eye className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Product Info */}
      <div className="p-5 flex-1 flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium mb-1">
            <span>{product.category}</span>
            {product.attributes?.weight && (
              <span className="text-slate-500 font-medium">{product.attributes.weight}</span>
            )}
          </div>

          <Link
            to={`/produto/${product.slug}`}
            className="font-bold text-slate-900 group-hover:text-amber-700 transition-colors line-clamp-1 text-base leading-snug"
          >
            {product.name}
          </Link>

          {product.subtitle && (
            <p className="text-xs text-slate-500 line-clamp-1 mt-0.5">
              {product.subtitle}
            </p>
          )}

          {/* Rating */}
          <div className="flex items-center gap-1.5 mt-2.5">
            <div className="flex items-center text-amber-500">
              <Star className="w-3.5 h-3.5 fill-amber-400" />
            </div>
            <span className="text-xs font-bold text-slate-700">{product.rating}</span>
            <span className="text-[11px] text-slate-400">({product.reviewCount})</span>
          </div>
        </div>

        {/* Price & Add to Cart button */}
        <div className="pt-4 mt-3 border-t border-slate-100 flex items-center justify-between gap-2">
          <div>
            <div className="text-lg font-extrabold text-slate-950">
              R$ {product.price.toFixed(2).replace('.', ',')}
            </div>
            {product.originalPrice && (
              <div className="text-xs text-slate-400 line-through">
                R$ {product.originalPrice.toFixed(2).replace('.', ',')}
              </div>
            )}
          </div>

          <button
            onClick={handleAddToCart}
            className="flex items-center gap-1.5 px-3.5 py-2 bg-amber-600 hover:bg-amber-700 text-white text-xs font-bold rounded-xl shadow-xs hover:shadow active:scale-95 transition-all cursor-pointer"
            title="Adicionar ao carrinho"
          >
            <ShoppingBag className="w-3.5 h-3.5" />
            <span>Comprar</span>
          </button>
        </div>
      </div>
    </div>
  );
};

export default ProductCard;
