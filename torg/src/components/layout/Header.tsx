import React, { useEffect, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { toggleCart } from '../../store/slices/cartSlice';
import { selectCartSummary } from '../../store';
import { ShoppingBag, Search, User, Heart, Globe } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export const Header: React.FC = () => {
  const { t, i18n } = useTranslation();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const cartSummary = useAppSelector(selectCartSummary);
  const [searchInput, setSearchInput] = useState(searchParams.get('q') ?? '');
  useEffect(() => setSearchInput(searchParams.get('q') ?? ''), [searchParams]);

  const handleSearch = (event: React.FormEvent) => {
    event.preventDefault();
    const params = new URLSearchParams(searchParams);
    const query = searchInput.trim();
    if (query) params.set('q', query);
    else params.delete('q');
    navigate(`/${params.size ? `?${params}` : ''}`);
  };

  return (
    <header className="sticky top-0 z-40 bg-white/95 backdrop-blur-md border-b border-slate-200/80">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center gap-3 md:gap-6 min-h-20">
          <Link to="/" className="flex items-center gap-3 shrink-0 group" aria-label="Torg: catálogo">
            <div className="w-10 h-10 rounded-xl bg-slate-950 flex items-center justify-center shadow-md">
              <span className="text-amber-400 font-black text-xl">T</span>
            </div>
            <div className="flex flex-col">
              <span className="text-xl font-extrabold text-slate-950 tracking-tight leading-none">TORG</span>
              <span className="text-[10px] tracking-widest uppercase font-semibold text-amber-700">Novgorod Vitrine</span>
            </div>
          </Link>
          <form onSubmit={handleSearch} role="search" className="hidden md:flex flex-1 max-w-xl mx-auto relative">
            <label className="sr-only" htmlFor="catalog-search">Buscar produtos</label>
            <input id="catalog-search" value={searchInput} onChange={e => setSearchInput(e.target.value)}
              placeholder="Buscar produtos..." className="w-full bg-slate-100 text-sm pl-11 pr-24 py-2.5 rounded-full border border-slate-200 focus:outline-none focus:border-amber-600" />
            <Search className="w-4 h-4 text-slate-400 absolute left-4 top-3.5" />
            <button type="submit" className="absolute right-1.5 top-1.5 bottom-1.5 px-4 bg-amber-600 text-white text-xs font-semibold rounded-full cursor-pointer">Buscar</button>
          </form>
          <div className="flex items-center gap-1 sm:gap-2 ml-auto md:ml-0">
            <Link to="/" title="Favoritos" aria-label="Favoritos" className="p-2 text-slate-600 hover:text-slate-900 rounded-full hover:bg-slate-100"><Heart className="w-5 h-5" /></Link>
            <button onClick={() => i18n.changeLanguage(i18n.language === 'en' ? 'pt' : 'en')}
              className="flex items-center gap-1 p-2 text-slate-600 rounded-full hover:bg-slate-100 uppercase font-bold text-xs cursor-pointer" title="Change Language" aria-label="Selecionar idioma">
              <Globe className="w-5 h-5" /><span>{i18n.language === 'en' ? 'EN' : 'PT'}</span>
            </button>
            <Link to="/carrinho" title="Minha Conta" aria-label="Minha Conta" className="p-2 text-slate-600 hover:text-slate-900 rounded-full hover:bg-slate-100"><User className="w-5 h-5" /></Link>
            <button onClick={() => dispatch(toggleCart())} aria-label="Abrir carrinho"
              className="relative flex items-center gap-2 bg-slate-950 text-white px-3 sm:px-4 py-2.5 rounded-full font-semibold text-sm cursor-pointer">
              <ShoppingBag className="w-4 h-4 text-amber-400" /><span className="hidden sm:inline">{t('nav.cart')}</span>
              <span className="bg-amber-500 text-slate-950 font-black text-xs px-2 py-0.5 rounded-full">{cartSummary.totalItems}</span>
            </button>
          </div>
        </div>
        <form onSubmit={handleSearch} role="search" className="md:hidden relative pb-3">
          <label className="sr-only" htmlFor="mobile-catalog-search">Buscar produtos</label>
          <input id="mobile-catalog-search" value={searchInput} onChange={e => setSearchInput(e.target.value)}
            placeholder="Buscar produtos..." className="w-full bg-slate-100 text-sm pl-10 pr-20 py-2.5 rounded-xl border border-slate-200" />
          <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-3.5" />
          <button type="submit" className="absolute right-2 top-1 px-3 py-1.5 text-xs font-semibold text-amber-700">Buscar</button>
        </form>
      </div>
    </header>
  );
};
export default Header;
