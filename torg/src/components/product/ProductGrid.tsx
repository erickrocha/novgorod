import React from 'react';
import type { Product } from '../../types';
import ProductCard from './ProductCard';
import { PackageOpen } from 'lucide-react';

interface ProductGridProps {
  products: Product[];
  isLoading?: boolean;
}

export const ProductGrid: React.FC<ProductGridProps> = ({
  products,
  isLoading = false,
}) => {
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {Array.from({ length: 8 }).map((_, i) => (
          <div
            key={i}
            className="bg-white rounded-2xl border border-slate-200/80 p-4 animate-pulse space-y-4"
          >
            <div className="aspect-4/3 bg-slate-200 rounded-xl w-full" />
            <div className="h-4 bg-slate-200 rounded-md w-3/4" />
            <div className="h-3 bg-slate-200 rounded-md w-1/2" />
            <div className="pt-2 flex justify-between items-center">
              <div className="h-5 bg-slate-200 rounded-md w-1/3" />
              <div className="h-8 bg-slate-200 rounded-xl w-20" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (products.length === 0) {
    return (
      <div className="text-center py-16 bg-white rounded-3xl border border-dashed border-slate-300 p-8">
        <div className="w-16 h-16 bg-amber-50 rounded-2xl flex items-center justify-center mx-auto mb-4 text-amber-600">
          <PackageOpen className="w-8 h-8" />
        </div>
        <h3 className="text-lg font-bold text-slate-800">Nenhum produto encontrado</h3>
        <p className="text-sm text-slate-500 mt-1 max-w-sm mx-auto">
          Tente ajustar os filtros, a busca ou selecione outra categoria do mercado.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {products.map((product) => (
        <ProductCard key={product.id} product={product} />
      ))}
    </div>
  );
};

export default ProductGrid;
