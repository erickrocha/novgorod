import React, { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchFeaturedProducts,
  fetchCategories,
  setCategory,
} from '../store/slices/catalogSlice';
import ProductCard from '../components/product/ProductCard';
import {
  Sparkles,
  ArrowRight,
  Wine,
  Flame,
  ChevronRight,
  Gift,
} from 'lucide-react';
import Button from '../components/common/Button';

export const Home: React.FC = () => {
  const dispatch = useAppDispatch();
  const { featuredProducts, categories } = useAppSelector(
    (state) => state.catalog
  );

  useEffect(() => {
    dispatch(fetchCategories());
    dispatch(fetchFeaturedProducts());
  }, [dispatch]);

  return (
    <div className="space-y-16 pb-16">
      {/* Hero Section */}
      <section className="relative overflow-hidden bg-slate-950 text-white py-20 lg:py-28">
        <div className="absolute inset-0 opacity-20 pointer-events-none bg-[radial-gradient(#f59e0b_1px,transparent_1px)] [background-size:20px_20px]" />
        
        {/* Ambient Glows */}
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-amber-600/20 rounded-full blur-3xl pointer-events-none" />

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
            <div className="lg:col-span-7 space-y-6 text-center lg:text-left">
              <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-400 text-xs font-bold uppercase tracking-wider">
                <Sparkles className="w-3.5 h-3.5" />
                <span>Vitrine & Mercado Oficial • Novgorod</span>
              </div>

              <h1 className="text-4xl sm:text-5xl lg:text-6xl font-black tracking-tight leading-[1.1]">
                O Grande Mercado de <span className="text-amber-400">Vinhos Nobres</span> e Sabores Raros.
              </h1>

              <p className="text-slate-300 text-base sm:text-lg max-w-2xl leading-relaxed">
                Inspirado na lendária *Torgovaya Storona*, reunimos uma curadoria exclusiva de rótulos de guarda, azeites extravirgens premiados e iguarias artesanais com logística climatizada.
              </p>

              <div className="flex flex-wrap items-center justify-center lg:justify-start gap-4 pt-2">
                <Link to="/catalogo">
                  <Button variant="primary" size="lg" className="gap-2 shadow-amber-900/30">
                    <span>Explorar Vitrine</span>
                    <ArrowRight className="w-4 h-4" />
                  </Button>
                </Link>
                <Link to="/catalogo?cat=vinhos-tintos">
                  <Button variant="outline" size="lg" className="border-slate-700 bg-slate-900/80 text-white hover:bg-slate-800">
                    <span>Vinhos Tintos</span>
                  </Button>
                </Link>
              </div>

              {/* Metrics */}
              <div className="grid grid-cols-3 gap-6 pt-6 border-t border-slate-800/80 max-w-lg mx-auto lg:mx-0">
                <div>
                  <div className="text-2xl font-black text-amber-400">100%</div>
                  <div className="text-xs text-slate-400 mt-0.5">Origem Verificada</div>
                </div>
                <div>
                  <div className="text-2xl font-black text-amber-400">14°C - 16°C</div>
                  <div className="text-xs text-slate-400 mt-0.5">Transporte Climatizado</div>
                </div>
                <div>
                  <div className="text-2xl font-black text-amber-400">4.9 ★</div>
                  <div className="text-xs text-slate-400 mt-0.5">Avaliação Média</div>
                </div>
              </div>
            </div>

            {/* Hero Visual Card */}
            <div className="lg:col-span-5">
              <div className="relative mx-auto max-w-md bg-gradient-to-b from-slate-900 to-slate-950 p-3 rounded-3xl border border-slate-800 shadow-2xl">
                <div className="relative rounded-2xl overflow-hidden aspect-4/5">
                  <img
                    src="https://images.unsplash.com/photo-1510812431401-41d2bd2722f3?q=80&w=1000&auto=format&fit=crop"
                    alt="Coleção Especial Torg"
                    className="w-full h-full object-cover"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-slate-950 via-transparent to-transparent opacity-90" />
                  <div className="absolute bottom-6 left-6 right-6 text-white space-y-2">
                    <span className="px-2.5 py-1 bg-amber-500 text-slate-950 text-xs font-black rounded-lg uppercase">
                      Safra Histórica
                    </span>
                    <h3 className="text-xl font-bold">Château Detinets Gran Reserva</h3>
                    <p className="text-xs text-slate-300">Envelhecido 24 meses em carvalho francês com notas de cassis e tabaco.</p>
                    <div className="flex items-center justify-between pt-2">
                      <span className="text-lg font-black text-amber-400">R$ 349,90</span>
                      <Link
                        to="/produto/chateau-detinets-reserva-2018"
                        className="text-xs font-bold text-white bg-slate-800/90 px-3 py-1.5 rounded-lg hover:bg-amber-600 transition-colors"
                      >
                        Ver Detalhes
                      </Link>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Category Explorer */}
      <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row md:items-end justify-between mb-8 gap-4">
          <div>
            <div className="flex items-center gap-2 text-amber-700 text-xs font-bold uppercase tracking-wider mb-1">
              <Wine className="w-4 h-4" />
              <span>Departamentos do Mercado</span>
            </div>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-slate-900 tracking-tight">
              Navegue por Categoria
            </h2>
          </div>
          <Link
            to="/catalogo"
            className="inline-flex items-center gap-1.5 text-sm font-bold text-amber-700 hover:text-amber-800 transition-colors"
          >
            <span>Ver todo o catálogo</span>
            <ChevronRight className="w-4 h-4" />
          </Link>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 sm:gap-6">
          {categories.map((cat) => (
            <Link
              key={cat.id}
              to={`/catalogo?cat=${cat.slug}`}
              onClick={() => dispatch(setCategory(cat.slug))}
              className="group relative rounded-2xl overflow-hidden border border-slate-200/80 bg-white p-4 shadow-xs hover:shadow-lg transition-all text-center flex flex-col items-center"
            >
              <div className="w-20 h-20 sm:w-24 sm:h-24 rounded-full overflow-hidden mb-3 border-2 border-slate-100 group-hover:border-amber-500 transition-colors">
                <img
                  src={cat.imageUrl}
                  alt={cat.name}
                  className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-300"
                />
              </div>
              <h3 className="font-bold text-slate-900 text-sm group-hover:text-amber-700 transition-colors line-clamp-1">
                {cat.name}
              </h3>
              {cat.itemCount && (
                <span className="text-xs text-slate-400 mt-1 font-medium">
                  {cat.itemCount} produtos
                </span>
              )}
            </Link>
          ))}
        </div>
      </section>

      {/* Featured Products Showcase */}
      <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row md:items-end justify-between mb-8 gap-4">
          <div>
            <div className="flex items-center gap-2 text-amber-700 text-xs font-bold uppercase tracking-wider mb-1">
              <Flame className="w-4 h-4" />
              <span>Seleção Especial</span>
            </div>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-slate-900 tracking-tight">
              Destaques da Vitrine
            </h2>
          </div>
          <Link
            to="/catalogo"
            className="inline-flex items-center gap-1.5 text-sm font-bold text-amber-700 hover:text-amber-800 transition-colors"
          >
            <span>Ver todos os produtos</span>
            <ChevronRight className="w-4 h-4" />
          </Link>
        </div>

        {/* Product Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {featuredProducts.slice(0, 4).map((product) => (
            <ProductCard key={product.id} product={product} />
          ))}
        </div>
      </section>

      {/* Promo Callout Banner */}
      <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="relative rounded-3xl overflow-hidden bg-gradient-to-r from-amber-600 via-amber-700 to-slate-900 text-white p-8 sm:p-12 shadow-xl">
          <div className="relative z-10 max-w-xl space-y-4">
            <div className="inline-flex items-center gap-2 bg-white/20 backdrop-blur-sm px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wide">
              <Gift className="w-3.5 h-3.5" />
              <span>Cupom de Boas-Vindas</span>
            </div>
            <h3 className="text-3xl sm:text-4xl font-black leading-tight">
              Ganhe 10% OFF na sua primeira compra no Mercado Torg
            </h3>
            <p className="text-white/80 text-sm leading-relaxed">
              Utilize o código promocional <strong className="text-white underline font-bold">TORG10</strong> na finalização do seu pedido e experimente os melhores rótulos e azeites artesanais.
            </p>
            <div className="pt-2">
              <Link to="/catalogo">
                <Button variant="secondary" size="md" className="bg-slate-950 hover:bg-slate-900 text-white font-bold">
                  Aproveitar Oferta Agora
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default Home;
