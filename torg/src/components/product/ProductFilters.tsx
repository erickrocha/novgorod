import React, { useEffect, useState } from 'react';
import { useAppSelector } from '../../store/hooks';
import type { ProductFilterState } from '../../types';
import { RotateCcw } from 'lucide-react';

interface Props {
  filters: ProductFilterState;
  onApply: (values: Pick<ProductFilterState, 'category' | 'minPrice' | 'maxPrice'>) => void;
  onClear: () => void;
}
export const ProductFilters: React.FC<Props> = ({ filters, onApply, onClear }) => {
  const categories = useAppSelector(state => state.catalog.categories);
  const [category, setCategory] = useState(filters.category);
  const [minPrice, setMinPrice] = useState(filters.minPrice?.toString() ?? '');
  const [maxPrice, setMaxPrice] = useState(filters.maxPrice?.toString() ?? '');
  const [error, setError] = useState('');
  useEffect(() => {
    setCategory(filters.category);
    setMinPrice(filters.minPrice?.toString() ?? '');
    setMaxPrice(filters.maxPrice?.toString() ?? '');
    setError('');
  }, [filters.category, filters.minPrice, filters.maxPrice]);
  const apply = (event: React.FormEvent) => {
    event.preventDefault();
    const min = minPrice.trim() === '' ? undefined : Number(minPrice);
    const max = maxPrice.trim() === '' ? undefined : Number(maxPrice);
    if ([min, max].some(value => value !== undefined && (!Number.isFinite(value) || value < 0)) ||
        (min !== undefined && max !== undefined && min > max)) {
      setError('Informe valores válidos; o mínimo não pode superar o máximo.');
      return;
    }
    setError('');
    onApply({ category, minPrice: min, maxPrice: max });
  };
  return <form onSubmit={apply} className="bg-white rounded-2xl border border-slate-200/80 p-6 space-y-6 shadow-xs">
    <div className="flex items-center justify-between border-b border-slate-100 pb-4">
      <h3 className="font-bold text-slate-900 text-base">Filtros do Mercado</h3>
      <button type="button" onClick={onClear} className="flex items-center gap-1 text-xs font-semibold text-amber-700 cursor-pointer"><RotateCcw className="w-3.5 h-3.5" />Limpar</button>
    </div>
    <fieldset><legend className="text-xs uppercase tracking-wider font-bold text-slate-500 mb-3">Categorias</legend>
      <div className="space-y-1.5">
        {[{ id: 'all', slug: 'all', name: 'Todas as Categorias' }, ...categories].map(cat =>
          <label key={cat.id} className="flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-semibold cursor-pointer hover:bg-slate-50">
            <input type="radio" name="category" checked={category === cat.slug} onChange={() => setCategory(cat.slug)} />{cat.name}
          </label>
        )}
      </div>
    </fieldset>
    <div className="border-t border-slate-100 pt-5">
      <h4 className="text-xs uppercase tracking-wider font-bold text-slate-500 mb-3">Faixa de Preço (R$)</h4>
      <div className="flex items-center gap-2">
        <label className="text-xs flex-1">Mínimo<input type="number" min="0" step="0.01" value={minPrice} onChange={e => setMinPrice(e.target.value)} className="w-full mt-1 text-xs p-2 rounded-lg border border-slate-200" /></label>
        <label className="text-xs flex-1">Máximo<input type="number" min="0" step="0.01" value={maxPrice} onChange={e => setMaxPrice(e.target.value)} className="w-full mt-1 text-xs p-2 rounded-lg border border-slate-200" /></label>
      </div>
      {error && <p role="alert" className="text-xs text-rose-600 mt-2">{error}</p>}
    </div>
    <button type="submit" className="w-full px-4 py-2.5 bg-amber-600 text-white font-semibold rounded-xl cursor-pointer">Aplicar filtros</button>
  </form>;
};
export default ProductFilters;
