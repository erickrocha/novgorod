import React from 'react';
import { Link } from 'react-router-dom';
import { ShieldCheck, Truck, RotateCcw, Award, Mail, Phone, MapPin } from 'lucide-react';

export const Footer: React.FC = () => {
  return (
    <footer className="bg-slate-950 text-slate-300 border-t border-slate-900 pt-16 pb-12">
      {/* Value Proposition Banners */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mb-16">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 py-8 px-6 bg-slate-900/60 rounded-3xl border border-slate-800/80">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-2xl bg-amber-500/10 flex items-center justify-center text-amber-400 shrink-0">
              <Truck className="w-6 h-6" />
            </div>
            <div>
              <h4 className="text-sm font-bold text-white">Logística Climatizada</h4>
              <p className="text-xs text-slate-400 mt-0.5">Vinhos e queijos protegidos em temperatura ideal.</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-2xl bg-emerald-500/10 flex items-center justify-center text-emerald-400 shrink-0">
              <Award className="w-6 h-6" />
            </div>
            <div>
              <h4 className="text-sm font-bold text-white">Autenticidade Garantida</h4>
              <p className="text-xs text-slate-400 mt-0.5">Rótulos e itens com selo de origem verificada.</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-2xl bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <div>
              <h4 className="text-sm font-bold text-white">Pagamento Seguro</h4>
              <p className="text-xs text-slate-400 mt-0.5">PIX, cartões de crédito e faturamento direto.</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-2xl bg-rose-500/10 flex items-center justify-center text-rose-400 shrink-0">
              <RotateCcw className="w-6 h-6" />
            </div>
            <div>
              <h4 className="text-sm font-bold text-white">Satisfação Assegurada</h4>
              <p className="text-xs text-slate-400 mt-0.5">Atendimento do sommelier e troca descomplicada.</p>
            </div>
          </div>
        </div>
      </div>

      {/* Main Footer Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-10 pb-12 border-b border-slate-800">
          {/* Brand info */}
          <div className="lg:col-span-2 space-y-4">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-amber-500 flex items-center justify-center">
                <span className="text-slate-950 font-black text-lg">T</span>
              </div>
              <span className="text-xl font-bold text-white tracking-tight">TORG</span>
            </div>
            <p className="text-sm text-slate-400 max-w-sm leading-relaxed">
              O Grande Mercado (*Torgovaya Storona*) do ecossistema Novgorod. Curadoria rigorosa de vinhos finos, azeites de terroir e iguarias artesanais para clientes exigentes.
            </p>
            <div className="space-y-2 text-xs text-slate-400">
              <div className="flex items-center gap-2">
                <MapPin className="w-4 h-4 text-amber-500" />
                <span>Novgorod E-Commerce Hub • Vale dos Vinhedos & São Paulo</span>
              </div>
              <div className="flex items-center gap-2">
                <Mail className="w-4 h-4 text-amber-500" />
                <span>contato@torg-novgorod.com</span>
              </div>
              <div className="flex items-center gap-2">
                <Phone className="w-4 h-4 text-amber-500" />
                <span>+55 (11) 3450-8800</span>
              </div>
            </div>
          </div>

          {/* Quick links */}
          <div>
            <h5 className="text-xs uppercase tracking-wider text-slate-200 font-bold mb-4">Departamentos</h5>
            <ul className="space-y-2.5 text-sm text-slate-400">
              <li><Link to="/?cat=vinhos-tintos" className="hover:text-amber-400 transition-colors">Vinhos Tintos</Link></li>
              <li><Link to="/?cat=vinhos-brancos" className="hover:text-amber-400 transition-colors">Vinhos Brancos</Link></li>
              <li><Link to="/?cat=azeites-especiarias" className="hover:text-amber-400 transition-colors">Azeites & Temperos</Link></li>
              <li><Link to="/?cat=queijos-charcutaria" className="hover:text-amber-400 transition-colors">Queijos & Charcutaria</Link></li>
              <li><Link to="/?cat=acessorios" className="hover:text-amber-400 transition-colors">Acessórios de Sommelier</Link></li>
            </ul>
          </div>

          {/* Customer service */}
          <div>
            <h5 className="text-xs uppercase tracking-wider text-slate-200 font-bold mb-4">Experiência & Ajuda</h5>
            <ul className="space-y-2.5 text-sm text-slate-400">
              <li><Link to="/carrinho" className="hover:text-amber-400 transition-colors">Meu Carrinho</Link></li>
              <li><a href="#frete" className="hover:text-amber-400 transition-colors">Políticas de Envio Climatizado</a></li>
              <li><a href="#trocas" className="hover:text-amber-400 transition-colors">Garantia & Devoluções</a></li>
              <li><a href="#veche-admin" className="hover:text-amber-400 transition-colors">Painel Veché (Admin)</a></li>
              <li><a href="#faq" className="hover:text-amber-400 transition-colors">Perguntas Frequentes</a></li>
            </ul>
          </div>

          {/* Newsletter */}
          <div>
            <h5 className="text-xs uppercase tracking-wider text-slate-200 font-bold mb-4">Clube Novgorod</h5>
            <p className="text-xs text-slate-400 mb-3 leading-relaxed">
              Receba prévias de safras exclusivas e convites para degustações privadas.
            </p>
            <div className="space-y-2">
              <input
                type="email"
                placeholder="Seu melhor e-mail"
                className="w-full bg-slate-900 border border-slate-800 text-white placeholder:text-slate-500 text-xs px-3.5 py-2.5 rounded-xl focus:outline-none focus:border-amber-500"
              />
              <button
                type="button"
                className="w-full bg-amber-600 hover:bg-amber-700 text-white text-xs font-semibold py-2.5 rounded-xl transition-colors cursor-pointer"
              >
                Cadastrar-se
              </button>
            </div>
          </div>
        </div>

        {/* Bottom Credits */}
        <div className="pt-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-slate-500">
          <p>© {new Date().getFullYear()} Torg — Módulo Storefront do Projeto Novgorod. Todos os direitos reservados.</p>
          <div className="flex items-center gap-6">
            <span>Kremlin Core API v1.0</span>
            <span>•</span>
            <span>Veché Admin Sync</span>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
