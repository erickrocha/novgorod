import React from 'react';
import { Link } from 'react-router-dom';
import Button from '../components/common/Button';
import { Breadcrumb } from '../components/common/Breadcrumb';

export const NotFoundPage: React.FC = () => {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
      <Breadcrumb items={[{ label: 'Não encontrado' }]} />

      <div className="max-w-xl mx-auto py-16 text-center space-y-4">
        <div className="text-6xl font-black text-amber-600">404</div>
        <h1 className="text-2xl font-bold text-slate-900">Página Não Encontrada</h1>
        <p className="text-sm text-slate-500 max-w-sm mx-auto">
          A página ou produto que você procurou não existe ou mudou de endereço no mercado Torg.
        </p>
        <div className="pt-4">
          <Link to="/">
            <Button variant="primary">Voltar para o Início</Button>
          </Link>
        </div>
      </div>
    </div>
  );
};


export default NotFoundPage;
