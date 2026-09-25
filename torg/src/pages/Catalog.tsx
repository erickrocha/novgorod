import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchProducts, fetchCategories, setFilters } from '../store/slices/catalogSlice';
import ProductGrid from '../components/product/ProductGrid';
import ProductFilters from '../components/product/ProductFilters';
import { SlidersHorizontal, X } from 'lucide-react';
import type { ProductFilterState } from '../types';

const readFilters = (params: URLSearchParams): ProductFilterState => {
  const price = (key: string) => {
    const raw = params.get(key);
    if (raw === null || raw.trim() === '') return undefined;
    const parsed = Number(raw);
    return Number.isFinite(parsed) && parsed >= 0 ? parsed : undefined;
  };
  const sort = params.get('sort');
  return {
    searchQuery: params.get('q')?.trim() ?? '',
    category: params.get('cat') ?? 'all',
    minPrice: price('min'), maxPrice: price('max'),
    sortBy: sort === 'price-asc' || sort === 'price-desc' ? sort : 'newest',
  };
};

export const Catalog: React.FC = () => {
  const dispatch = useAppDispatch();
  const [searchParams, setSearchParams] = useSearchParams();
  const [mobileFiltersOpen, setMobileFiltersOpen] = useState(false);
  const { products, totalProducts, isLoading, filters, nextCursor, error, categories } = useAppSelector(state => state.catalog);
  const urlKey = searchParams.toString();
  const urlFilters = useMemo(() => readFilters(searchParams), [urlKey]);
  useEffect(() => { dispatch(fetchCategories()); }, [dispatch]);
  useEffect(() => {
    dispatch(setFilters(urlFilters));
    dispatch(fetchProducts());
  }, [dispatch, urlFilters]);
  const updateParams = (change: Record<string, string | undefined>) => {
    const next = new URLSearchParams(searchParams);
    for (const [key, value] of Object.entries(change)) {
      if (value === undefined || value === '') next.delete(key);
      else next.set(key, value);
    }
    setSearchParams(next);
  };
  const apply = (values: Pick<ProductFilterState, 'category' | 'minPrice' | 'maxPrice'>) => {
    updateParams({ cat: values.category === 'all' ? undefined : values.category,
      min: values.minPrice?.toString(), max: values.maxPrice?.toString() });
    setMobileFiltersOpen(false);
  };
  const clear = () => { setSearchParams(new URLSearchParams()); setMobileFiltersOpen(false); };
  const categoryName = categories.find(c => c.slug === filters.category)?.name ?? filters.category;
  return <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-200 pb-6">
      <div><h1 className="text-3xl font-extrabold text-slate-900 tracking-tight">Catálogo & Vitrine</h1>
        <p className="text-sm text-slate-500 mt-1">{totalProducts} {totalProducts === 1 ? 'produto' : 'produtos'} encontrados</p></div>
      <div className="flex items-center gap-3">
        <button onClick={() => setMobileFiltersOpen(true)} className="lg:hidden flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-xl text-sm font-semibold cursor-pointer"><SlidersHorizontal className="w-4 h-4" />Filtros</button>
        <label className="text-xs font-semibold text-slate-500">Ordenar:
          <select value={filters.sortBy} onChange={e => updateParams({sort: e.target.value === 'newest' ? undefined : e.target.value})} className="ml-2 bg-white border border-slate-200 rounded-xl px-3 py-2 text-xs font-semibold text-slate-800">
            <option value="newest">Novidades</option><option value="price-asc">Menor Preço</option><option value="price-desc">Maior Preço</option>
          </select>
        </label>
      </div>
    </div>
    {(filters.searchQuery || filters.category !== 'all' || filters.minPrice !== undefined || filters.maxPrice !== undefined) &&
      <div className="flex flex-wrap gap-2 text-xs items-center"><span className="text-slate-500">Filtros ativos:</span>
        {filters.searchQuery && <button onClick={() => updateParams({q: undefined})} className="px-3 py-1 bg-amber-50 rounded-full">Busca: {filters.searchQuery} ×</button>}
        {filters.category !== 'all' && <button onClick={() => updateParams({cat: undefined})} className="px-3 py-1 bg-amber-50 rounded-full">Categoria: {categoryName} ×</button>}
        {(filters.minPrice !== undefined || filters.maxPrice !== undefined) && <button onClick={() => updateParams({min: undefined, max: undefined})} className="px-3 py-1 bg-amber-50 rounded-full">Preço: {filters.minPrice ?? 0} – {filters.maxPrice ?? 'sem limite'} ×</button>}
      </div>}
    <div className="grid grid-cols-1 lg:grid-cols-4 gap-8 items-start">
      <aside className="hidden lg:block lg:col-span-1 sticky top-28"><ProductFilters filters={filters} onApply={apply} onClear={clear} /></aside>
      {mobileFiltersOpen && <div className="fixed inset-0 z-50 lg:hidden flex"><div className="fixed inset-0 bg-slate-900/60" onClick={() => setMobileFiltersOpen(false)} />
        <div className="relative ml-auto w-full max-w-xs bg-white h-full p-6 overflow-y-auto shadow-2xl space-y-4"><div className="flex justify-between"><strong>Filtros</strong><button onClick={() => setMobileFiltersOpen(false)} aria-label="Fechar filtros"><X /></button></div><ProductFilters filters={filters} onApply={apply} onClear={clear} /></div></div>}
      <div className="lg:col-span-3">
        {error && <div role="alert" className="mb-4 p-4 bg-rose-50 text-rose-700 rounded-xl">Falha ao carregar catálogo. <button onClick={() => dispatch(fetchProducts())} className="underline">Tentar novamente</button></div>}
        {!error && <ProductGrid products={products} isLoading={isLoading && !products.length} />}
        {nextCursor !== null && <div className="mt-10 flex justify-center"><button onClick={() => dispatch(fetchProducts({loadMore: true}))} disabled={isLoading} className="px-6 py-3 bg-amber-600 text-white font-semibold rounded-xl disabled:opacity-70">{isLoading ? 'Carregando...' : 'Carregar Mais Produtos'}</button></div>}
      </div>
    </div>
  </div>;
};
export default Catalog;
