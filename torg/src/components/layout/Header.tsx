import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { toggleCart } from '../../store/slices/cartSlice';
import { setSearchQuery } from '../../store/slices/catalogSlice';
import { selectCartSummary } from '../../store';
import {
  ShoppingBag,
  Search,
  Menu,
  X,
  User,
  Heart,
  ChevronRight,
} from 'lucide-react';

export const Header: React.FC = () => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const cartSummary = useAppSelector(selectCartSummary);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [searchInput, setSearchInput] = useState('');

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (searchInput.trim()) {
      dispatch(setSearchQuery(searchInput.trim()));
      navigate(`/catalogo?q=${encodeURIComponent(searchInput.trim())}`);
      setMobileNavOpen(false);
    }
  };

  return (
    <header className="sticky top-0 z-40 bg-white/95 backdrop-blur-md border-b border-slate-200/80 transition-all">
      {/* Top micro bar: Novgorod announcement */}
      <div className="bg-slate-900 text-slate-300 text-xs py-1.5 px-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="inline-block w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span>
            <span className="font-medium text-white">Torg • O Grande Mercado</span>
            <span className="hidden sm:inline text-slate-400">| Vitrine exclusiva Novgorod</span>
          </div>
          <div className="flex items-center gap-4 text-slate-300">
            <span className="hidden md:inline">Frete grátis em pedidos selecionados</span>
            <Link to="/catalogo" className="text-amber-400 hover:text-amber-300 font-semibold underline underline-offset-2">
              Explorar Ofertas
            </Link>
          </div>
        </div>
      </div>

      {/* Main Header Container */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-20 gap-4">
          {/* Brand / Logo */}
          <div className="flex items-center gap-3">
            <button
              onClick={() => setMobileNavOpen(!mobileNavOpen)}
              className="lg:hidden p-2 text-slate-600 hover:text-slate-950 rounded-lg cursor-pointer"
              aria-label="Abrir menu"
            >
              {mobileNavOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </button>

            <Link to="/" className="flex items-center gap-3 group">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-slate-950 via-slate-900 to-amber-950 flex items-center justify-center shadow-md group-hover:scale-105 transition-transform">
                <span className="text-amber-400 font-black text-xl tracking-tighter">T</span>
              </div>
              <div className="flex flex-col">
                <span className="text-xl font-extrabold text-slate-950 tracking-tight leading-none group-hover:text-amber-700 transition-colors">
                  TORG
                </span>
                <span className="text-[10px] tracking-widest uppercase font-semibold text-amber-700 leading-tight">
                  Novgorod Vitrine
                </span>
              </div>
            </Link>
          </div>

          {/* Desktop Search Bar */}
          <form
            onSubmit={handleSearch}
            className="hidden md:flex flex-1 max-w-lg relative mx-4"
          >
            <input
              type="text"
              placeholder="Buscar vinhos nobres, azeites, queijos, especiarias..."
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              className="w-full bg-slate-100/90 text-slate-900 placeholder:text-slate-400 text-sm pl-11 pr-24 py-2.5 rounded-full border border-slate-200 focus:outline-none focus:border-amber-600 focus:bg-white focus:ring-2 focus:ring-amber-500/20 transition-all"
            />
            <Search className="w-4 h-4 text-slate-400 absolute left-4 top-3.5" />
            <button
              type="submit"
              className="absolute right-1.5 top-1.5 bottom-1.5 px-4 bg-amber-600 hover:bg-amber-700 text-white text-xs font-semibold rounded-full transition-colors cursor-pointer"
            >
              Buscar
            </button>
          </form>

          {/* Navigation Links */}
          <nav className="hidden lg:flex items-center gap-6 text-sm font-semibold text-slate-700">
            <Link to="/" className="hover:text-amber-600 transition-colors">
              Início
            </Link>
            <Link to="/catalogo" className="hover:text-amber-600 transition-colors">
              Catálogo & Vitrine
            </Link>
            <Link to="/catalogo?cat=vinhos-tintos" className="hover:text-amber-600 transition-colors">
              Vinhos
            </Link>
            <Link to="/catalogo?cat=azeites-especiarias" className="hover:text-amber-600 transition-colors">
              Gastronomia
            </Link>
          </nav>

          {/* User & Cart Actions */}
          <div className="flex items-center gap-2 sm:gap-3">
            <Link
              to="/catalogo"
              className="p-2.5 text-slate-600 hover:text-slate-900 rounded-full hover:bg-slate-100 transition-colors"
              title="Favoritos"
            >
              <Heart className="w-5 h-5" />
            </Link>

            <Link
              to="/carrinho"
              className="p-2.5 text-slate-600 hover:text-slate-900 rounded-full hover:bg-slate-100 transition-colors"
              title="Minha Conta"
            >
              <User className="w-5 h-5" />
            </Link>

            {/* Cart Button with Counter */}
            <button
              onClick={() => dispatch(toggleCart())}
              className="relative flex items-center gap-2.5 bg-slate-950 hover:bg-slate-900 text-white px-4 py-2.5 rounded-full font-semibold text-sm shadow-md hover:shadow-lg transition-all cursor-pointer group"
              aria-label="Abrir carrinho"
            >
              <ShoppingBag className="w-4 h-4 text-amber-400 group-hover:scale-110 transition-transform" />
              <span className="hidden sm:inline">Carrinho</span>
              <span className="bg-amber-500 text-slate-950 font-black text-xs px-2 py-0.5 rounded-full">
                {cartSummary.totalItems}
              </span>
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Nav Drawer */}
      {mobileNavOpen && (
        <div className="lg:hidden border-t border-slate-200 bg-white px-4 pt-3 pb-6 space-y-4 animate-in slide-in-from-top duration-200">
          <form onSubmit={handleSearch} className="relative">
            <input
              type="text"
              placeholder="Buscar produtos..."
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              className="w-full bg-slate-100 text-sm pl-10 pr-4 py-2.5 rounded-xl border border-slate-200"
            />
            <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-3.5" />
          </form>

          <div className="flex flex-col gap-2 font-medium text-slate-800">
            <Link
              to="/"
              onClick={() => setMobileNavOpen(false)}
              className="flex items-center justify-between py-2 px-3 hover:bg-slate-50 rounded-lg"
            >
              <span>Início</span>
              <ChevronRight className="w-4 h-4 text-slate-400" />
            </Link>
            <Link
              to="/catalogo"
              onClick={() => setMobileNavOpen(false)}
              className="flex items-center justify-between py-2 px-3 hover:bg-slate-50 rounded-lg"
            >
              <span>Catálogo Completo</span>
              <ChevronRight className="w-4 h-4 text-slate-400" />
            </Link>
            <Link
              to="/catalogo?cat=vinhos-tintos"
              onClick={() => setMobileNavOpen(false)}
              className="flex items-center justify-between py-2 px-3 hover:bg-slate-50 rounded-lg"
            >
              <span>Vinhos Nobres</span>
              <ChevronRight className="w-4 h-4 text-slate-400" />
            </Link>
            <Link
              to="/catalogo?cat=azeites-especiarias"
              onClick={() => setMobileNavOpen(false)}
              className="flex items-center justify-between py-2 px-3 hover:bg-slate-50 rounded-lg"
            >
              <span>Azeites & Especiarias</span>
              <ChevronRight className="w-4 h-4 text-slate-400" />
            </Link>
            <Link
              to="/carrinho"
              onClick={() => setMobileNavOpen(false)}
              className="flex items-center justify-between py-2 px-3 hover:bg-slate-50 rounded-lg"
            >
              <span>Meu Carrinho ({cartSummary.totalItems})</span>
              <ChevronRight className="w-4 h-4 text-slate-400" />
            </Link>
          </div>
        </div>
      )}
    </header>
  );
};

export default Header;
