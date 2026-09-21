import React from 'react';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import {
  setCategory,
  setPriceRange,
  setSortBy,
  resetFilters,
} from '../../store/slices/catalogSlice';
import { RotateCcw } from 'lucide-react';
import type { ProductFilterState } from '../../types';

export const ProductFilters: React.FC = () => {
  const dispatch = useAppDispatch();
  const { categories, filters } = useAppSelector((state) => state.catalog);

  return (
    <div className="bg-white rounded-2xl border border-slate-200/80 p-6 space-y-6 shadow-xs">
      <div className="flex items-center justify-between border-b border-slate-100 pb-4">
        <h3 className="font-bold text-slate-900 text-base">Filtros do Mercado</h3>
        <button
          onClick={() => dispatch(resetFilters())}
          className="flex items-center gap-1 text-xs font-semibold text-amber-700 hover:text-amber-800 transition-colors cursor-pointer"
        >
          <RotateCcw className="w-3.5 h-3.5" />
          <span>Limpar</span>
        </button>
      </div>

      {/* Categories */}
      <div>
        <h4 className="text-xs uppercase tracking-wider font-bold text-slate-500 mb-3">
          Categorias
        </h4>
        <div className="space-y-1.5">
          <button
            onClick={() => dispatch(setCategory('all'))}
            className={`w-full text-left px-3 py-2 rounded-xl text-xs font-semibold transition-colors cursor-pointer ${
              filters.category === 'all'
                ? 'bg-amber-50 text-amber-800 font-bold'
                : 'text-slate-600 hover:bg-slate-50'
            }`}
          >
            Todas as Categorias
          </button>
          {categories.map((cat) => (
            <button
              key={cat.id}
              onClick={() => dispatch(setCategory(cat.slug))}
              className={`w-full text-left px-3 py-2 rounded-xl text-xs font-semibold transition-colors cursor-pointer flex items-center justify-between ${
                filters.category === cat.slug
                  ? 'bg-amber-50 text-amber-800 font-bold'
                  : 'text-slate-600 hover:bg-slate-50'
              }`}
            >
              <span className="truncate">{cat.name}</span>
              {cat.itemCount && (
                <span className="text-[10px] bg-slate-100 text-slate-500 px-1.5 py-0.5 rounded-full">
                  {cat.itemCount}
                </span>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Price Range */}
      <div className="border-t border-slate-100 pt-5">
        <h4 className="text-xs uppercase tracking-wider font-bold text-slate-500 mb-3">
          Faixa de Preço (R$)
        </h4>
        <div className="flex items-center gap-2">
          <input
            type="number"
            min={0}
            max={filters.maxPrice}
            value={filters.minPrice}
            onChange={(e) =>
              dispatch(
                setPriceRange({ min: Number(e.target.value), max: filters.maxPrice })
              )
            }
            className="w-full text-xs p-2 rounded-lg border border-slate-200 focus:outline-none focus:border-amber-500"
            placeholder="Min"
          />
          <span className="text-slate-400 text-xs">até</span>
          <input
            type="number"
            min={filters.minPrice}
            max={5000}
            value={filters.maxPrice}
            onChange={(e) =>
              dispatch(
                setPriceRange({ min: filters.minPrice, max: Number(e.target.value) })
              )
            }
            className="w-full text-xs p-2 rounded-lg border border-slate-200 focus:outline-none focus:border-amber-500"
            placeholder="Max"
          />
        </div>
        <div className="mt-3">
          <input
            type="range"
            min={0}
            max={2000}
            step={25}
            value={filters.maxPrice}
            onChange={(e) =>
              dispatch(
                setPriceRange({ min: filters.minPrice, max: Number(e.target.value) })
              )
            }
            className="w-full accent-amber-600 cursor-pointer"
          />
          <div className="flex justify-between text-[11px] text-slate-400 mt-1">
            <span>R$ 0</span>
            <span>R$ {filters.maxPrice}</span>
          </div>
        </div>
      </div>

      {/* Order / Sorting */}
      <div className="border-t border-slate-100 pt-5">
        <h4 className="text-xs uppercase tracking-wider font-bold text-slate-500 mb-2">
          Ordenar por
        </h4>
        <select
          value={filters.sortBy}
          onChange={(e) =>
            dispatch(setSortBy(e.target.value as ProductFilterState['sortBy']))
          }
          className="w-full text-xs p-2.5 rounded-xl border border-slate-200 bg-white focus:outline-none focus:border-amber-500 font-medium text-slate-700 cursor-pointer"
        >
          <option value="featured">Destaques da Vitrine</option>
          <option value="price-asc">Menor Preço</option>
          <option value="price-desc">Maior Preço</option>
          <option value="rating">Melhor Avaliação</option>
          <option value="newest">Novidades Recentes</option>
        </select>
      </div>
    </div>
  );
};

export default ProductFilters;
