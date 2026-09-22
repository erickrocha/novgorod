import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useForm, Controller } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { selectCartItems, selectCartSummary, selectShippingCep } from '../store';
import { clearCart } from '../store/slices/cartSlice';
import { addToast } from '../store/slices/uiSlice';
import Button from '../components/common/Button';
import { Breadcrumb } from '../components/common/Breadcrumb';
import {
  ShieldCheck,
  CreditCard,
  QrCode,
  FileText,
  CheckCircle,
  Truck,
  ArrowLeft,
} from 'lucide-react';

interface CheckoutFormData {
  name: string;
  email: string;
  phone: string;
  postalCode: string;
  street: string;
  number: string;
  neighborhood: string;
  city: string;
  state: string;
  paymentMethod: 'pix' | 'credit' | 'boleto';
}

const checkoutSchema = yup.object({
  name: yup.string().trim().min(3, 'Nome deve ter no mínimo 3 caracteres').required('Nome é obrigatório'),
  email: yup.string().trim().email('E-mail inválido').required('E-mail é obrigatório'),
  phone: yup.string().trim().min(8, 'Telefone inválido').required('Telefone é obrigatório'),
  postalCode: yup
    .string()
    .trim()
    .required('CEP é obrigatório')
    .matches(/^\d{5}-?\d{3}$/, 'CEP deve ter 8 dígitos'),
  street: yup.string().trim().required('Rua/Avenida é obrigatória'),
  number: yup.string().trim().required('Número é obrigatório'),
  neighborhood: yup.string().trim().required('Bairro é obrigatório'),
  city: yup.string().trim().required('Cidade é obrigatória'),
  state: yup.string().trim().required('Estado é obrigatório'),
  paymentMethod: yup
    .string()
    .oneOf(['pix', 'credit', 'boleto'] as const)
    .required('Selecione um método de pagamento'),
});

export const CheckoutPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const items = useAppSelector(selectCartItems);
  const summary = useAppSelector(selectCartSummary);
  const savedCep = useAppSelector(selectShippingCep);

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [orderCompleted, setOrderCompleted] = useState(false);
  const [orderNumber, setOrderNumber] = useState('');
  const [completedPaymentMethod, setCompletedPaymentMethod] = useState<'pix' | 'credit' | 'boleto'>('pix');

  const {
    register,
    handleSubmit,
    control,
    formState: { errors },
  } = useForm<CheckoutFormData>({
    resolver: yupResolver(checkoutSchema),
    defaultValues: {
      name: '',
      email: '',
      phone: '',
      postalCode: savedCep || '01310-100',
      street: '',
      number: '',
      neighborhood: '',
      city: 'São Paulo',
      state: 'SP',
      paymentMethod: 'pix',
    },
  });


  const onSubmit = (data: CheckoutFormData) => {
    setIsSubmitting(true);

    setTimeout(() => {
      setIsSubmitting(false);
      const generatedOrder = `TRG-${Math.floor(100000 + Math.random() * 900000)}`;
      setOrderNumber(generatedOrder);
      setCompletedPaymentMethod(data.paymentMethod);
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
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <Breadcrumb
          items={[
            { label: 'Carrinho', href: '/carrinho' },
            { label: 'Checkout', href: '/checkout' },
            { label: 'Confirmação' },
          ]}
          className="mb-6"
        />

        <div className="max-w-2xl mx-auto py-12 text-center space-y-6">
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
              <span className="font-bold text-slate-900 uppercase">{completedPaymentMethod}</span>
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
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <Breadcrumb items={[{ label: 'Carrinho', href: '/carrinho' }, { label: 'Checkout' }]} className="mb-6" />
        <div className="max-w-4xl mx-auto py-16 text-center">
          <h2 className="text-2xl font-bold text-slate-900">Seu carrinho está vazio</h2>
          <p className="text-slate-500 mt-2 mb-6">Adicione produtos antes de ir para o checkout.</p>
          <Link to="/catalogo">
            <Button variant="primary">Ver Catálogo</Button>
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
      <Breadcrumb
        items={[
          { label: 'Carrinho', href: '/carrinho' },
          { label: 'Checkout' },
        ]}
      />

      <div className="border-b border-slate-200 pb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold text-slate-950 tracking-tight">
            Finalização de Compra (Checkout)
          </h1>
          <p className="text-sm text-slate-500 mt-0.5">
            Preencha seus dados de entrega e selecione a forma de pagamento seguro.
          </p>
        </div>
        <Link
          to="/carrinho"
          className="inline-flex items-center gap-1.5 text-xs font-bold text-slate-600 hover:text-amber-700 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          <span>Voltar ao Carrinho</span>
        </Link>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* Forms column */}
        <div className="lg:col-span-8 space-y-6">
          {/* Customer info */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <span>1. Informações de Contato</span>
            </h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
              <div className="sm:col-span-2">
                <label htmlFor="customer-name" className="text-xs font-semibold text-slate-700 block mb-1">
                  Nome Completo
                </label>
                <input
                  id="customer-name"
                  type="text"
                  placeholder="Ex: João da Silva"
                  {...register('name')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.name ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.name && (
                  <p className="text-xs text-rose-600 mt-1">{errors.name.message}</p>
                )}
              </div>
              <div>
                <label htmlFor="customer-email" className="text-xs font-semibold text-slate-700 block mb-1">
                  E-mail
                </label>
                <input
                  id="customer-email"
                  type="email"
                  placeholder="joao@exemplo.com"
                  {...register('email')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.email ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.email && (
                  <p className="text-xs text-rose-600 mt-1">{errors.email.message}</p>
                )}
              </div>
              <div>
                <label htmlFor="customer-phone" className="text-xs font-semibold text-slate-700 block mb-1">
                  Telefone / WhatsApp
                </label>
                <input
                  id="customer-phone"
                  type="tel"
                  placeholder="(11) 99999-9999"
                  {...register('phone')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.phone ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.phone && (
                  <p className="text-xs text-rose-600 mt-1">{errors.phone.message}</p>
                )}
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
                <label htmlFor="shipping-postal-code" className="text-xs font-semibold text-slate-700 block mb-1">
                  CEP
                </label>
                <input
                  id="shipping-postal-code"
                  type="text"
                  placeholder="00000-000"
                  {...register('postalCode')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.postalCode ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.postalCode && (
                  <p className="text-xs text-rose-600 mt-1">{errors.postalCode.message}</p>
                )}
              </div>
              <div className="sm:col-span-2">
                <label htmlFor="shipping-street" className="text-xs font-semibold text-slate-700 block mb-1">
                  Rua / Avenida
                </label>
                <input
                  id="shipping-street"
                  type="text"
                  placeholder="Av. Paulista"
                  {...register('street')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.street ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.street && (
                  <p className="text-xs text-rose-600 mt-1">{errors.street.message}</p>
                )}
              </div>
              <div>
                <label htmlFor="shipping-number" className="text-xs font-semibold text-slate-700 block mb-1">
                  Número
                </label>
                <input
                  id="shipping-number"
                  type="text"
                  placeholder="1000"
                  {...register('number')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.number ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.number && (
                  <p className="text-xs text-rose-600 mt-1">{errors.number.message}</p>
                )}
              </div>
              <div>
                <label htmlFor="shipping-neighborhood" className="text-xs font-semibold text-slate-700 block mb-1">
                  Bairro
                </label>
                <input
                  id="shipping-neighborhood"
                  type="text"
                  placeholder="Bela Vista"
                  {...register('neighborhood')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.neighborhood ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.neighborhood && (
                  <p className="text-xs text-rose-600 mt-1">{errors.neighborhood.message}</p>
                )}
              </div>
              <div>
                <label htmlFor="shipping-city" className="text-xs font-semibold text-slate-700 block mb-1">
                  Cidade
                </label>
                <input
                  id="shipping-city"
                  type="text"
                  placeholder="São Paulo"
                  {...register('city')}
                  className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
                    errors.city ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
                  }`}
                />
                {errors.city && (
                  <p className="text-xs text-rose-600 mt-1">{errors.city.message}</p>
                )}
              </div>
            </div>
          </div>

          {/* Payment Method */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-amber-600" />
              <span>3. Método de Pagamento Seguro</span>
            </h2>

            <Controller
              name="paymentMethod"
              control={control}
              render={({ field }) => (
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                  <button
                    type="button"
                    onClick={() => field.onChange('pix')}
                    className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                      field.value === 'pix'
                        ? 'border-amber-600 bg-amber-50/50 text-slate-950 ring-2 ring-amber-600/20'
                        : 'border-slate-200 hover:border-slate-300 text-slate-600'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <QrCode className="w-6 h-6 text-amber-600" />
                      <span className="text-[10px] font-bold uppercase bg-emerald-100 text-emerald-800 px-2 py-0.5 rounded-full">
                        Instantâneo
                      </span>
                    </div>
                    <div>
                      <div className="font-bold text-sm text-slate-900">PIX</div>
                      <p className="text-xs text-slate-500">Chave QR Code instantânea</p>
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => field.onChange('credit')}
                    className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                      field.value === 'credit'
                        ? 'border-amber-600 bg-amber-50/50 text-slate-950 ring-2 ring-amber-600/20'
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
                    onClick={() => field.onChange('boleto')}
                    className={`p-4 rounded-2xl border-2 text-left flex flex-col justify-between transition-all cursor-pointer ${
                      field.value === 'boleto'
                        ? 'border-amber-600 bg-amber-50/50 text-slate-950 ring-2 ring-amber-600/20'
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
              )}
            />
            {errors.paymentMethod && (
              <p className="text-xs text-rose-600 mt-1">{errors.paymentMethod.message}</p>
            )}
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

            {/* Actions: Cancel / Back on left, Submit on right */}
            <div className="pt-4 flex flex-col gap-2">
              <Button
                type="submit"
                variant="primary"
                size="lg"
                isLoading={isSubmitting}
                className="w-full font-bold shadow-md"
              >
                Confirmar e Pagar
              </Button>
              <Link to="/carrinho" className="w-full">
                <Button
                  type="button"
                  variant="ghost"
                  size="md"
                  className="w-full text-slate-500 hover:text-slate-700 text-xs"
                >
                  Voltar e revisar carrinho
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </form>
    </div>
  );
};

export default CheckoutPage;
