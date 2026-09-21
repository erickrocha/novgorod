import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { selectCartItems, selectCartSummary } from '../store';
import { clearCart } from '../store/slices/cartSlice';
import { addToast } from '../store/slices/uiSlice';
import Button from '../components/common/Button';
import {
  ShieldCheck,
  CreditCard,
  QrCode,
  FileText,
  CheckCircle,
  Truck,
  ArrowLeft,
} from 'lucide-react';

export const CheckoutPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const items = useAppSelector(selectCartItems);
  const summary = useAppSelector(selectCartSummary);

  const [paymentMethod, setPaymentMethod] = useState<'pix' | 'credit' | 'boleto'>('pix');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [orderCompleted, setOrderCompleted] = useState(false);
  const [orderNumber, setOrderNumber] = useState('');

  // Form states
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    phone: '',
    street: '',
    number: '',
    neighborhood: '',
    city: 'São Paulo',
    state: 'SP',
    postalCode: '01310-100',
  });

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handlePlaceOrder = (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    setTimeout(() => {
      setIsSubmitting(false);
      const generatedOrder = `TRG-${Math.floor(100000 + Math.random() * 900000)}`;
      setOrderNumber(generatedOrder);
      setOrderCompleted(true);
      dispatch(clearCart());
      dispatch(
        addToast({
          type: 'success',
          message: `Pedido ${generatedOrder} confirmado com sucesso!`,
        })
      );
    }, 1200);
  };

  if (orderCompleted) {
    return (
      <div className="max-w-2xl mx-auto px-4 py-20 text-center space-y-6">
        <div className="w-20 h-20 bg-emerald-100 text-emerald-600 rounded-full flex items-center justify-center mx-auto shadow-inner">
          <CheckCircle className="w-10 h-10" />
        </div>
        <h1 className="text-3xl font-extrabold text-slate-950">
          Pedido Realizado com Sucesso!
        </h1>
        <p className="text-slate-600 max-w-md mx-auto">
          Obrigado por comprar no Mercado Torg. Seu pedido <strong className="text-slate-900">#{orderNumber}</strong> foi registrado e enviado para o núcleo de processamento Novgorod.
        </p>

        <div className="p-6 bg-white rounded-3xl border border-slate-200/80 max-w-md mx-auto text-left space-y-3 text-sm">
          <div className="flex justify-between border-b pb-2">
            <span className="text-slate-500">Número do Pedido:</span>
            <span className="font-bold text-slate-900">{orderNumber}</span>
          </div>
          <div className="flex justify-between border-b pb-2">
            <span className="text-slate-500">Método de Pagamento:</span>
            <span className="font-bold text-slate-900 uppercase">{paymentMethod}</span>
          </div>
          <div className="flex justify-between border-b pb-2">
            <span className="text-slate-500">Valor Total:</span>
            <span className="font-bold text-amber-700">R$ {summary.total.toFixed(2).replace('.', ',')}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-500">Status:</span>
            <span className="font-bold text-emerald-600">Aguardando Confirmação</span>
          </div>
        </div>

        <div className="pt-4">
          <Link to="/">
            <Button variant="primary" size="lg" className="font-bold">
              Retornar à Página Inicial
            </Button>
          </Link>
        </div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-20 text-center">
        <h2 className="text-2xl font-bold text-slate-900">Seu carrinho está vazio</h2>
        <p className="text-slate-500 mt-2 mb-6">Adicione produtos antes de ir para o checkout.</p>
        <Link to="/catalogo">
          <Button variant="primary">Ver Catálogo</Button>
        </Link>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-8">
      <div className="border-b border-slate-200 pb-6">
        <Link
          to="/carrinho"
          className="inline-flex items-center gap-1.5 text-xs font-bold text-slate-500 hover:text-amber-700 mb-2 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          <span>Voltar ao Carrinho</span>
        </Link>
        <h1 className="text-3xl font-extrabold text-slate-950 tracking-tight">
          Finalização de Compra (Checkout)
        </h1>
        <p className="text-sm text-slate-500 mt-0.5">
          Preencha seus dados de entrega e selecione a forma de pagamento.
        </p>
      </div>

      <form onSubmit={handlePlaceOrder} className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* Forms column */}
        <div className="lg:col-span-8 space-y-6">
          {/* Customer info */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <span>1. Informações de Contato</span>
            </h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
              <div className="sm:col-span-2">
                <label className="text-xs font-semibold text-slate-700 block mb-1">Nome Completo</label>
                <input
                  type="text"
                  name="name"
                  required
                  placeholder="Ex: João da Silva"
                  value={formData.name}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">E-mail</label>
                <input
                  type="email"
                  name="email"
                  required
                  placeholder="joao@exemplo.com"
                  value={formData.email}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">Telefone / WhatsApp</label>
                <input
                  type="tel"
                  name="phone"
                  required
                  placeholder="(11) 99999-9999"
                  value={formData.phone}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
            </div>
          </div>

          {/* Shipping Address */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <Truck className="w-5 h-5 text-amber-600" />
              <span>2. Endereço de Entrega Climatizada</span>
            </h2>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 text-sm">
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">CEP</label>
                <input
                  type="text"
                  name="postalCode"
                  required
                  value={formData.postalCode}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div className="sm:col-span-2">
                <label className="text-xs font-semibold text-slate-700 block mb-1">Rua / Avenida</label>
                <input
                  type="text"
                  name="street"
                  required
                  placeholder="Av. Paulista"
                  value={formData.street}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">Número</label>
                <input
                  type="text"
                  name="number"
                  required
                  placeholder="1000"
                  value={formData.number}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">Bairro</label>
                <input
                  type="text"
                  name="neighborhood"
                  required
                  placeholder="Bela Vista"
                  value={formData.neighborhood}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">Cidade / UF</label>
                <input
                  type="text"
                  name="city"
                  required
                  value={`${formData.city} - ${formData.state}`}
                  onChange={handleInputChange}
                  className="w-full p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500 text-sm"
                />
              </div>
            </div>
          </div>

          {/* Payment Method */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-amber-600" />
              <span>3. Método de Pagamento Seguro</span>
            </h2>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <button
                type="button"
                onClick={() => setPaymentMethod('pix')}
                className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                  paymentMethod === 'pix'
                    ? 'border-amber-600 bg-amber-50/50 text-slate-950'
                    : 'border-slate-200 hover:border-slate-300 text-slate-600'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <QrCode className="w-6 h-6 text-amber-600" />
                  <span className="text-[10px] font-bold uppercase bg-emerald-100 text-emerald-800 px-2 py-0.5 rounded-full">
                    Aprovação Instantânea
                  </span>
                </div>
                <div>
                  <div className="font-bold text-sm text-slate-900">PIX</div>
                  <p className="text-xs text-slate-500">Chave QR Code instantânea</p>
                </div>
              </button>

              <button
                type="button"
                onClick={() => setPaymentMethod('credit')}
                className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                  paymentMethod === 'credit'
                    ? 'border-amber-600 bg-amber-50/50 text-slate-950'
                    : 'border-slate-200 hover:border-slate-300 text-slate-600'
                }`}
              >
                <CreditCard className="w-6 h-6 text-amber-600 mb-2" />
                <div>
                  <div className="font-bold text-sm text-slate-900">Cartão de Crédito</div>
                  <p className="text-xs text-slate-500">Até 6x sem juros</p>
                </div>
              </button>

              <button
                type="button"
                onClick={() => setPaymentMethod('boleto')}
                className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                  paymentMethod === 'boleto'
                    ? 'border-amber-600 bg-amber-50/50 text-slate-950'
                    : 'border-slate-200 hover:border-slate-300 text-slate-600'
                }`}
              >
                <FileText className="w-6 h-6 text-amber-600 mb-2" />
                <div>
                  <div className="font-bold text-sm text-slate-900">Boleto Bancário</div>
                  <p className="text-xs text-slate-500">Compensação em 1 dia útil</p>
                </div>
              </button>
            </div>
          </div>
        </div>

        {/* Order Review sidebar */}
        <div className="lg:col-span-4 space-y-6">
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 shadow-xs space-y-4">
            <h3 className="font-bold text-slate-900 text-base border-b pb-3">
              Itens do Pedido ({summary.totalItems})
            </h3>

            <div className="max-h-60 overflow-y-auto divide-y divide-slate-100 pr-1">
              {items.map((item) => (
                <div key={item.id} className="py-2.5 flex items-center gap-3 text-xs">
                  <img
                    src={item.product.thumbnail}
                    alt={item.product.name}
                    className="w-10 h-10 object-cover rounded-lg shrink-0 border"
                  />
                  <div className="flex-1 min-w-0">
                    <span className="font-bold text-slate-800 truncate block">
                      {item.product.name}
                    </span>
                    <span className="text-slate-400">Qtd: {item.quantity}</span>
                  </div>
                  <span className="font-bold text-slate-900">
                    R$ {item.totalPrice.toFixed(2).replace('.', ',')}
                  </span>
                </div>
              ))}
            </div>

            {/* Calculations */}
            <div className="border-t border-slate-100 pt-3 space-y-2 text-xs">
              <div className="flex justify-between text-slate-600">
                <span>Subtotal</span>
                <span>R$ {summary.subtotal.toFixed(2).replace('.', ',')}</span>
              </div>
              {summary.discountAmount > 0 && (
                <div className="flex justify-between text-emerald-600 font-bold">
                  <span>Desconto</span>
                  <span>- R$ {summary.discountAmount.toFixed(2).replace('.', ',')}</span>
                </div>
              )}
              <div className="flex justify-between text-slate-600">
                <span>Frete Climatizado</span>
                <span>{summary.shippingAmount > 0 ? `R$ ${summary.shippingAmount.toFixed(2).replace('.', ',')}` : 'Grátis'}</span>
              </div>
              <div className="border-t pt-2 flex justify-between text-base font-extrabold text-slate-950">
                <span>Total a Pagar</span>
                <span className="text-amber-700">R$ {summary.total.toFixed(2).replace('.', ',')}</span>
              </div>
            </div>

            {/* Submit */}
            <Button
              type="submit"
              variant="primary"
              size="lg"
              isLoading={isSubmitting}
              className="w-full font-bold shadow-md mt-4"
            >
              Confirmar e Pagar
            </Button>
          </div>
        </div>
      </form>
    </div>
  );
};

export default CheckoutPage;
