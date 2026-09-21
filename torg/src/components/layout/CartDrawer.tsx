import React from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import {
  closeCart,
  updateQuantity,
  removeFromCart,
} from '../../store/slices/cartSlice';
import { selectCartIsOpen, selectCartSummary, selectCartItems } from '../../store';
import { X, ShoppingBag, Plus, Minus, Trash2, ArrowRight } from 'lucide-react';
import Button from '../common/Button';

export const CartDrawer: React.FC = () => {
  const dispatch = useAppDispatch();
  const isOpen = useAppSelector(selectCartIsOpen);
  const items = useAppSelector(selectCartItems);
  const summary = useAppSelector(selectCartSummary);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 overflow-hidden">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-slate-900/60 backdrop-blur-sm transition-opacity"
        onClick={() => dispatch(closeCart())}
      />

      <div className="fixed inset-y-0 right-0 max-w-full flex pl-10">
        <div className="w-screen max-w-md bg-white shadow-2xl flex flex-col">
          {/* Header */}
          <div className="p-6 border-b border-slate-100 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ShoppingBag className="w-5 h-5 text-amber-600" />
              <h2 className="text-lg font-bold text-slate-900">
                Seu Carrinho ({summary.totalItems})
              </h2>
            </div>
            <button
              onClick={() => dispatch(closeCart())}
              className="p-2 text-slate-400 hover:text-slate-600 rounded-lg hover:bg-slate-100 transition-colors cursor-pointer"
              aria-label="Fechar carrinho"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Cart Items List */}
          <div className="flex-1 overflow-y-auto p-6 divide-y divide-slate-100">
            {items.length === 0 ? (
              <div className="h-full flex flex-col items-center justify-center text-center p-6 text-slate-500">
                <div className="w-16 h-16 bg-amber-50 rounded-full flex items-center justify-center mb-4 text-amber-600">
                  <ShoppingBag className="w-8 h-8" />
                </div>
                <h3 className="text-base font-semibold text-slate-800 mb-1">
                  Seu carrinho está vazio
                </h3>
                <p className="text-sm text-slate-500 mb-6">
                  Explore os produtos nobres e artesanais do mercado Torg.
                </p>
                <Button
                  onClick={() => dispatch(closeCart())}
                  variant="primary"
                  size="md"
                >
                  Continuar Comprando
                </Button>
              </div>
            ) : (
              items.map((item) => (
                <div key={item.id} className="py-4 first:pt-0 last:pb-0 flex gap-4">
                  <img
                    src={item.product.thumbnail}
                    alt={item.product.name}
                    className="w-20 h-20 object-cover rounded-xl border border-slate-100 shrink-0"
                  />
                  <div className="flex-1 min-w-0 flex flex-col justify-between">
                    <div>
                      <div className="flex justify-between items-start">
                        <Link
                          to={`/produto/${item.product.slug}`}
                          onClick={() => dispatch(closeCart())}
                          className="text-sm font-semibold text-slate-900 hover:text-amber-600 truncate transition-colors"
                        >
                          {item.product.name}
                        </Link>
                        <button
                          onClick={() => dispatch(removeFromCart(item.id))}
                          className="text-slate-400 hover:text-rose-500 p-1 transition-colors cursor-pointer"
                          title="Remover item"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                      {item.variant && (
                        <p className="text-xs text-slate-500">{item.variant.name}</p>
                      )}
                      <p className="text-sm font-bold text-amber-700 mt-1">
                        R$ {item.unitPrice.toFixed(2).replace('.', ',')}
                      </p>
                    </div>

                    {/* Quantity Selector */}
                    <div className="flex items-center gap-2 mt-2">
                      <div className="flex items-center border border-slate-200 rounded-lg overflow-hidden">
                        <button
                          onClick={() =>
                            dispatch(
                              updateQuantity({ id: item.id, quantity: item.quantity - 1 })
                            )
                          }
                          className="p-1 hover:bg-slate-100 text-slate-600 cursor-pointer transition-colors"
                        >
                          <Minus className="w-3.5 h-3.5" />
                        </button>
                        <span className="px-3 text-xs font-semibold text-slate-800">
                          {item.quantity}
                        </span>
                        <button
                          onClick={() =>
                            dispatch(
                              updateQuantity({ id: item.id, quantity: item.quantity + 1 })
                            )
                          }
                          className="p-1 hover:bg-slate-100 text-slate-600 cursor-pointer transition-colors"
                        >
                          <Plus className="w-3.5 h-3.5" />
                        </button>
                      </div>
                      <span className="text-xs text-slate-400 ml-auto font-medium">
                        Total: R$ {item.totalPrice.toFixed(2).replace('.', ',')}
                      </span>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>

          {/* Footer & Checkout Summary */}
          {items.length > 0 && (
            <div className="p-6 border-t border-slate-100 bg-slate-50/70 space-y-4">
              <div className="space-y-2 text-sm">
                <div className="flex justify-between text-slate-600">
                  <span>Subtotal</span>
                  <span className="font-semibold text-slate-900">
                    R$ {summary.subtotal.toFixed(2).replace('.', ',')}
                  </span>
                </div>
                {summary.discountAmount > 0 && (
                  <div className="flex justify-between text-emerald-600 font-medium">
                    <span>Desconto ({summary.coupon?.code})</span>
                    <span>- R$ {summary.discountAmount.toFixed(2).replace('.', ',')}</span>
                  </div>
                )}
                <div className="border-t border-slate-200 pt-2 flex justify-between text-base font-bold text-slate-900">
                  <span>Total Previsto</span>
                  <span className="text-amber-700">
                    R$ {summary.total.toFixed(2).replace('.', ',')}
                  </span>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3 pt-2">
                <Link
                  to="/carrinho"
                  onClick={() => dispatch(closeCart())}
                  className="w-full inline-flex items-center justify-center px-4 py-2.5 text-sm font-medium rounded-xl border border-slate-300 text-slate-700 bg-white hover:bg-slate-50 transition-colors"
                >
                  Ver Carrinho
                </Link>
                <Link
                  to="/checkout"
                  onClick={() => dispatch(closeCart())}
                  className="w-full inline-flex items-center justify-center gap-1.5 px-4 py-2.5 text-sm font-medium rounded-xl bg-amber-600 text-white hover:bg-amber-700 transition-colors shadow-sm"
                >
                  <span>Finalizar</span>
                  <ArrowRight className="w-4 h-4" />
                </Link>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default CartDrawer;
