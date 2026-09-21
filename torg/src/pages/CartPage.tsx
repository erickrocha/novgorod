import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  updateQuantity,
  removeFromCart,
  applyCoupon,
  removeCoupon,
  setShipping,
  clearCart,
} from '../store/slices/cartSlice';
import { addToast } from '../store/slices/uiSlice';
import { selectCartItems, selectCartSummary, selectAppliedCoupon } from '../store';
import cartService from '../services/cartService';
import catalogService from '../services/catalogService';
import {
  ShoppingBag,
  Trash2,
  Plus,
  Minus,
  ArrowRight,
  ArrowLeft,
  Tag,
  Truck,
} from 'lucide-react';
import Button from '../components/common/Button';

export const CartPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const items = useAppSelector(selectCartItems);
  const summary = useAppSelector(selectCartSummary);
  const appliedCoupon = useAppSelector(selectAppliedCoupon);

  const [couponCode, setCouponCode] = useState('');
  const [couponLoading, setCouponLoading] = useState(false);
  const [cep, setCep] = useState('');
  const [shippingLoading, setShippingLoading] = useState(false);
  const [shippingDetails, setShippingDetails] = useState<{
    cost: number;
    deliveryDays: number;
    service: string;
  } | null>(null);

  const handleApplyCoupon = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!couponCode.trim()) return;

    setCouponLoading(true);
    try {
      const coupon = await catalogService.validateCoupon(couponCode.trim());
      dispatch(applyCoupon(coupon));
      dispatch(
        addToast({
          type: 'success',
          message: `Cupom ${coupon.code} aplicado com sucesso!`,
        })
      );
      setCouponCode('');
    } catch {
      dispatch(
        addToast({
          type: 'error',
          message: 'Cupom inválido ou expirado. Tente TORG10 ou NOVGOROD50',
        })
      );
    } finally {
      setCouponLoading(false);
    }
  };

  const handleCalculateShipping = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!cep.trim()) return;

    setShippingLoading(true);
    try {
      const quote = await cartService.estimateShipping(cep);
      setShippingDetails(quote);
      dispatch(setShipping({ amount: quote.cost, cep }));
      dispatch(
        addToast({
          type: 'info',
          message: `Frete calculado: ${quote.service} - R$ ${quote.cost.toFixed(2)}`,
        })
      );
    } catch {
      dispatch(
        addToast({
          type: 'error',
          message: 'Falha ao calcular frete para este CEP.',
        })
      );
    } finally {
      setShippingLoading(false);
    }
  };

  if (items.length === 0) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-24 text-center">
        <div className="w-20 h-20 bg-amber-50 rounded-full flex items-center justify-center mx-auto mb-6 text-amber-600">
          <ShoppingBag className="w-10 h-10" />
        </div>
        <h1 className="text-3xl font-extrabold text-slate-900 mb-2">
          Seu Carrinho está Vazio
        </h1>
        <p className="text-slate-500 mb-8 max-w-md mx-auto">
          Parece que você ainda não selecionou nenhum item. Conheça nossa seleção de vinhos, azeites e queijos artesanais.
        </p>
        <Link to="/catalogo">
          <Button variant="primary" size="lg" className="gap-2 font-bold">
            <ArrowLeft className="w-4 h-4" />
            <span>Explorar Vitrine</span>
          </Button>
        </Link>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-8">
      <div className="flex items-center justify-between border-b border-slate-200 pb-6">
        <div>
          <h1 className="text-3xl font-extrabold text-slate-900 tracking-tight">
            Carrinho de Compras
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            Revise seus produtos antes de seguir para o pagamento seguro.
          </p>
        </div>
        <button
          onClick={() => dispatch(clearCart())}
          className="text-xs font-semibold text-rose-600 hover:text-rose-700 cursor-pointer"
        >
          Esvaziar Carrinho
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        {/* Items Table / List */}
        <div className="lg:col-span-8 bg-white rounded-3xl border border-slate-200/80 shadow-xs overflow-hidden">
          <div className="p-6 divide-y divide-slate-100">
            {items.map((item) => (
              <div key={item.id} className="py-5 first:pt-0 last:pb-0 flex flex-col sm:flex-row gap-5 items-center">
                <img
                  src={item.product.thumbnail}
                  alt={item.product.name}
                  className="w-24 h-24 object-cover rounded-2xl border border-slate-100 shrink-0"
                />

                <div className="flex-1 min-w-0 text-center sm:text-left">
                  <Link
                    to={`/produto/${item.product.slug}`}
                    className="text-base font-bold text-slate-900 hover:text-amber-700 transition-colors"
                  >
                    {item.product.name}
                  </Link>
                  {item.variant && (
                    <p className="text-xs text-slate-500 mt-0.5">{item.variant.name}</p>
                  )}
                  <p className="text-sm font-semibold text-slate-600 mt-1">
                    Preço unitário: R$ {item.unitPrice.toFixed(2).replace('.', ',')}
                  </p>
                </div>

                {/* Stepper */}
                <div className="flex items-center border border-slate-300 rounded-xl overflow-hidden bg-slate-50">
                  <button
                    onClick={() =>
                      dispatch(
                        updateQuantity({ id: item.id, quantity: item.quantity - 1 })
                      )
                    }
                    className="px-3 py-2 text-slate-600 hover:bg-slate-200 cursor-pointer"
                  >
                    <Minus className="w-3.5 h-3.5" />
                  </button>
                  <span className="px-3 text-xs font-bold text-slate-800">
                    {item.quantity}
                  </span>
                  <button
                    onClick={() =>
                      dispatch(
                        updateQuantity({ id: item.id, quantity: item.quantity + 1 })
                      )
                    }
                    className="px-3 py-2 text-slate-600 hover:bg-slate-200 cursor-pointer"
                  >
                    <Plus className="w-3.5 h-3.5" />
                  </button>
                </div>

                {/* Subtotal */}
                <div className="text-right min-w-[100px]">
                  <span className="text-base font-extrabold text-slate-950 block">
                    R$ {item.totalPrice.toFixed(2).replace('.', ',')}
                  </span>
                </div>

                {/* Remove */}
                <button
                  onClick={() => dispatch(removeFromCart(item.id))}
                  className="p-2 text-slate-400 hover:text-rose-600 transition-colors cursor-pointer"
                  title="Remover produto"
                >
                  <Trash2 className="w-5 h-5" />
                </button>
              </div>
            ))}
          </div>

          <div className="p-6 bg-slate-50/70 border-t border-slate-100 flex items-center justify-between">
            <Link
              to="/catalogo"
              className="inline-flex items-center gap-2 text-sm font-bold text-amber-700 hover:text-amber-800 transition-colors"
            >
              <ArrowLeft className="w-4 h-4" />
              <span>Continuar Comprando</span>
            </Link>
          </div>
        </div>

        {/* Order Summary & Actions */}
        <div className="lg:col-span-4 space-y-6">
          {/* Summary Box */}
          <div className="bg-white rounded-3xl border border-slate-200/80 p-6 shadow-xs space-y-5">
            <h2 className="text-lg font-bold text-slate-900 border-b border-slate-100 pb-3">
              Resumo do Pedido
            </h2>

            {/* Coupon Section */}
            <form onSubmit={handleApplyCoupon} className="space-y-2">
              <label className="text-xs font-bold uppercase tracking-wider text-slate-500 block">
                Cupom de Desconto
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="Ex: TORG10"
                  value={couponCode}
                  onChange={(e) => setCouponCode(e.target.value.toUpperCase())}
                  className="flex-1 text-xs uppercase p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500"
                />
                <Button
                  type="submit"
                  variant="outline"
                  size="sm"
                  isLoading={couponLoading}
                >
                  Aplicar
                </Button>
              </div>
              {appliedCoupon && (
                <div className="flex items-center justify-between text-xs bg-emerald-50 text-emerald-800 p-2.5 rounded-xl border border-emerald-200">
                  <div className="flex items-center gap-1.5">
                    <Tag className="w-3.5 h-3.5" />
                    <span className="font-bold">{appliedCoupon.code} aplicado</span>
                  </div>
                  <button
                    type="button"
                    onClick={() => dispatch(removeCoupon())}
                    className="text-rose-600 hover:underline font-semibold cursor-pointer"
                  >
                    Remover
                  </button>
                </div>
              )}
            </form>

            {/* Shipping Estimator */}
            <form onSubmit={handleCalculateShipping} className="space-y-2 border-t border-slate-100 pt-4">
              <label className="text-xs font-bold uppercase tracking-wider text-slate-500 block">
                Calcular Frete Climatizado
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="00000-000"
                  value={cep}
                  onChange={(e) => setCep(e.target.value)}
                  className="flex-1 text-xs p-2.5 rounded-xl border border-slate-200 focus:outline-none focus:border-amber-500"
                />
                <Button
                  type="submit"
                  variant="outline"
                  size="sm"
                  isLoading={shippingLoading}
                >
                  Calcular
                </Button>
              </div>
              {shippingDetails && (
                <p className="text-xs text-slate-600 mt-1 flex items-center gap-1">
                  <Truck className="w-3.5 h-3.5 text-amber-600" />
                  <span>{shippingDetails.service} (em até {shippingDetails.deliveryDays} dias)</span>
                </p>
              )}
            </form>

            {/* Price Calculations */}
            <div className="space-y-2.5 border-t border-slate-100 pt-4 text-sm">
              <div className="flex justify-between text-slate-600">
                <span>Subtotal ({summary.totalItems} itens)</span>
                <span className="font-semibold text-slate-900">
                  R$ {summary.subtotal.toFixed(2).replace('.', ',')}
                </span>
              </div>

              {summary.discountAmount > 0 && (
                <div className="flex justify-between text-emerald-600 font-semibold">
                  <span>Desconto Aplicado</span>
                  <span>- R$ {summary.discountAmount.toFixed(2).replace('.', ',')}</span>
                </div>
              )}

              <div className="flex justify-between text-slate-600">
                <span>Frete</span>
                <span className="font-semibold text-slate-900">
                  {summary.shippingAmount > 0
                    ? `R$ ${summary.shippingAmount.toFixed(2).replace('.', ',')}`
                    : 'A calcular'}
                </span>
              </div>

              <div className="border-t border-slate-200 pt-3 flex justify-between text-lg font-black text-slate-950">
                <span>Total</span>
                <span className="text-amber-700">
                  R$ {summary.total.toFixed(2).replace('.', ',')}
                </span>
              </div>
            </div>

            {/* Checkout Button */}
            <Link to="/checkout" className="block pt-2">
              <Button variant="primary" size="lg" className="w-full gap-2 font-bold shadow-md">
                <span>Fechar Pedido</span>
                <ArrowRight className="w-4 h-4" />
              </Button>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CartPage;
