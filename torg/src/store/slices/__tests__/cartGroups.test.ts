import { describe, expect, it } from 'vitest';
import { selectCartGroups, selectCartSummary, type RootState } from '../../index';
import cartReducer, { addToCart, enrichCartSeller, removeFromCart, updateQuantity } from '../cartSlice';
import type { Product } from '../../../types';

const product = (id: string, sellerId?: number, name = 'Shared name'): Product => ({
  id, slug: id, name: `Product ${id}`, description: '', price: 100, currency: 'BRL',
  category: '', categorySlug: '', images: [], thumbnail: '', rating: 0, reviewCount: 0, stock: 1,
  seller: sellerId ? { id: sellerId, businessName: name } : undefined,
});
const state = (items: ReturnType<typeof cartReducer>['items']) => ({ cart: { items, appliedCoupon: null, shippingAmount: 0 } } as RootState);

describe('cart tenant grouping', () => {
  it('separates tenants by ID, keeps first-added order, and updates subtotals', () => {
    let cart = cartReducer(undefined, { type: 'init' });
    cart = cartReducer(cart, addToCart({ product: product('one', 1), quantity: 2 }));
    cart = cartReducer(cart, addToCart({ product: product('two', 2), quantity: 1 }));
    cart = cartReducer(cart, addToCart({ product: product('three', 1), quantity: 1 }));
    let groups = selectCartGroups(state(cart.items));
    expect(groups.map(g => g.key)).toEqual(['seller:1', 'seller:2']);
    expect(groups.map(g => g.subtotal)).toEqual([300, 100]);
    expect(selectCartSummary(state(cart.items)).total).toBe(400);
    cart = cartReducer(cart, updateQuantity({ id: 'one', quantity: 3 }));
    expect(selectCartGroups(state(cart.items))[0].subtotal).toBe(400);
    cart = cartReducer(cart, removeFromCart('two'));
    expect(selectCartGroups(state(cart.items))).toHaveLength(1);
  });
  it('keeps unknown products separate until seller enrichment', () => {
    let cart = cartReducer(undefined, { type: 'init' });
    cart = cartReducer(cart, addToCart({ product: product('old-one') }));
    cart = cartReducer(cart, addToCart({ product: product('old-two') }));
    expect(selectCartGroups(state(cart.items)).map(g => g.key)).toEqual(['unknown:old-one', 'unknown:old-two']);
    cart = cartReducer(cart, updateQuantity({ id: 'old-one', quantity: 3 }));
    cart = cartReducer(cart, enrichCartSeller.fulfilled(
      { productId: 'old-one', seller: { id: 1, businessName: 'Seller 1' } },
      'lookup', { productId: 'old-one', slug: 'old-one' },
    ));
    expect(selectCartGroups(state(cart.items)).map(g => g.key)).toEqual(['seller:1', 'unknown:old-two']);
    expect(cart.items[0].quantity).toBe(3);
    expect(selectCartSummary(state(cart.items)).total).toBe(400);
  });
});
