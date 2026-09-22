import React, { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchProducts,
  fetchCategories,
  setCategory,
  setSearchQuery,
  setSortBy,
} from '../store/slices/catalogSlice';
import ProductGrid from '../components/product/ProductGrid';
import ProductFilters from '../components/product/ProductFilters';
import { SlidersHorizontal, X } from 'lucide-react';
import type { ProductFilterState } from '../types';
import { Breadcrumb } from '../components/common/Breadcrumb';

export const Catalog: React.FC = () => {
  const dispatch = useAppDispatch();
  const [searchParams, setSearchParams] = useSearchParams();
  const [mobileFiltersOpen, setMobileFiltersOpen] = useState(false);

  const { products, totalProducts, isLoading, filters } = useAppSelector(
    (state) => state.catalog
  );

  // Sync from URL query parameters on mount or change
  useEffect(() => {
    const urlCategory = searchParams.get('cat');
    const urlQuery = searchParams.get('q');

    if (urlCategory) {
      dispatch(setCategory(urlCategory));
    }
    if (urlQuery !== null) {
      dispatch(setSearchQuery(urlQuery));
    }

    dispatch(fetchCategories());
  }, [searchParams, dispatch]);

  // Fetch products whenever filters in Redux change
  useEffect(() => {
    dispatch(fetchProducts());
  }, [dispatch, filters.category, filters.searchQuery, filters.minPrice, filters.maxPrice, filters.sortBy]);

  const handleClearSearch = () => {
    dispatch(setSearchQuery(''));
    searchParams.delete('q');
    setSearchParams(searchParams);
  };

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
      <Breadcrumb items={[{ label: 'Catálogo' }]} />

      {/* Page Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-200 pb-6">

        <div>
          <h1 className="text-3xl font-extrabold text-slate-900 tracking-tight">
            Catálogo & Vitrine
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            Explorando {totalProducts} {totalProducts === 1 ? 'produto selecionado' : 'produtos selecionados'} no Grande Mercado
          </p>
        </div>

        {/* Mobile Filter Toggle & Quick Sort */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => setMobileFiltersOpen(!mobileFiltersOpen)}
            className="lg:hidden flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-xl text-sm font-semibold text-slate-700 shadow-xs cursor-pointer"
          >
            <SlidersHorizontal className="w-4 h-4 text-amber-600" />
            <span>Filtros</span>
          </button>

          <div className="flex items-center gap-2 text-xs font-semibold text-slate-500">
            <span className="hidden sm:inline">Ordenar:</span>
            <select
              value={filters.sortBy}
              onChange={(e) =>
                dispatch(setSortBy(e.target.value as ProductFilterState['sortBy']))
              }
              className="bg-white border border-slate-200 rounded-xl px-3 py-2 text-xs font-semibold text-slate-800 focus:outline-none focus:border-amber-500 cursor-pointer"
            >
              <option value="featured">Destaques</option>
              <option value="price-asc">Menor Preço</option>
              <option value="price-desc">Maior Preço</option>
              <option value="rating">Melhor Avaliação</option>
              <option value="newest">Novidades</option>
            </select>
          </div>
        </div>
      </div>

      {/* Active Search / Category Chips */}
      {(filters.searchQuery || filters.category !== 'all') && (
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs text-slate-400 font-medium">Filtros ativos:</span>
          {filters.searchQuery && (
            <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-amber-50 border border-amber-200 text-amber-900 rounded-full text-xs font-semibold">
              <span>Busca: "{filters.searchQuery}"</span>
              <button
                onClick={handleClearSearch}
                className="hover:text-amber-700 cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </span>
          )}
          {filters.category !== 'all' && (
            <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 border border-slate-200 text-slate-800 rounded-full text-xs font-semibold">
              <span>Categoria: {filters.category}</span>
              <button
                onClick={() => dispatch(setCategory('all'))}
                className="hover:text-slate-900 cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </span>
          )}
        </div>
      )}

      {/* Main Layout: Sidebar Filters + Products Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8 items-start">
        {/* Desktop Sidebar */}
        <aside className="hidden lg:block lg:col-span-1 sticky top-28">
          <ProductFilters />
        </aside>

        {/* Mobile Filter Modal / Drawer */}
        {mobileFiltersOpen && (
          <div className="fixed inset-0 z-50 lg:hidden flex">
            <div
              className="fixed inset-0 bg-slate-900/60 backdrop-blur-xs"
              onClick={() => setMobileFiltersOpen(false)}
            />
            <div className="relative ml-auto w-full max-w-xs bg-white h-full p-6 overflow-y-auto shadow-2xl space-y-4">
              <div className="flex items-center justify-between border-b pb-4">
                <span className="font-bold text-slate-900">Filtros</span>
                <button
                  onClick={() => setMobileFiltersOpen(false)}
                  className="p-1 text-slate-400 hover:text-slate-600 cursor-pointer"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
              <ProductFilters />
            </div>
          </div>
        )}

        {/* Products Grid */}
        <div className="lg:col-span-3">
          <ProductGrid products={products} isLoading={isLoading} />
        </div>
      </div>
    </div>
  );
};

export default Catalog;
