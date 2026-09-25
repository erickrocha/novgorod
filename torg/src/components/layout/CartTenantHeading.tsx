import React from 'react';
import type { CartItem } from '../../types';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { enrichCartSeller } from '../../store/slices/cartSlice';

interface Props { name: string; subtotal: number; unresolved: boolean; items: CartItem[] }
export const CartTenantHeading: React.FC<Props> = ({ name, subtotal, unresolved, items }) => {
  const dispatch = useAppDispatch();
  const pending = useAppSelector(state => state.cart.sellerLookupPendingIds);
  const first = items[0];
  return <div className="py-3 border-b border-slate-200 flex items-start justify-between gap-2">
    <div><h3 className="font-bold text-slate-900">{name}</h3>
      {unresolved && first && <button disabled={pending.includes(first.product.id)}
        onClick={() => dispatch(enrichCartSeller({productId: first.product.id, slug: first.product.slug}))}
        className="text-xs text-amber-700 underline disabled:opacity-50 cursor-pointer">{pending.includes(first.product.id) ? 'Buscando vendedor...' : 'Tentar identificar vendedor'}</button>}
    </div>
    <span className="text-sm font-semibold whitespace-nowrap">Subtotal: R$ {subtotal.toFixed(2).replace('.', ',')}</span>
  </div>;
};

export default CartTenantHeading;
