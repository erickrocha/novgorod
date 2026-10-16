import { describe, it, expect, beforeEach, vi } from 'vitest';
import cartReducer, {
  addToCart,
  removeFromCart,
  updateQuantity,
  clearCart,
  setShipping,
  estimateShipping,
  type CartState,
} from '../cartSlice';
import type { Product, CartItem } from '../../../types';
import cartService from '../../../services/cartService';

vi.mock('../../../services/catalogService');
vi.mock('../../../services/cartService');

const mockProduct: Product = {
  id: 'prod-1',
  name: 'Vinho Tinto Reserva',
  slug: 'vinho-tinto-reserva',
  price: 150.0,
  currency: 'BRL',
  category: 'Vinhos',
  categorySlug: 'vinhos',
  stock: 10,
  isNew: true,
  thumbnail: 'https://example.com/thumb.jpg',
  images: ['https://example.com/thumb.jpg'],
  rating: 4.8,
  reviewCount: 12,
  description: 'Excelente vinho tinto',
};


describe('cartSlice', () => {
  let initialState: CartState;

  beforeEach(() => {
    localStorage.clear();
    initialState = {
      items: [],
      isOpen: false,
      shippingAmount: 0,
      shippingCep: '',
      shippingLoading: false,
      shippingError: null,
      shippingDetails: null,
      sellerLookupPendingIds: [],
      sellerLookupFailedIds: [],
    };
  });

  it('should handle addToCart for a new product', () => {
    const nextState = cartReducer(initialState, addToCart({ product: mockProduct, quantity: 2 }));

    expect(nextState.items).toHaveLength(1);
    expect(nextState.items[0].product.id).toBe('prod-1');
    expect(nextState.items[0].quantity).toBe(2);
    expect(nextState.items[0].totalPrice).toBe(300.0);
    expect(nextState.isOpen).toBe(true);
  });

  it('should increment quantity when adding an existing product', () => {
    const stateWithItem: CartState = {
      ...initialState,
      items: [
        {
          id: 'prod-1',
          product: mockProduct,
          quantity: 1,
          unitPrice: 150.0,
          totalPrice: 150.0,
        },
      ],
    };

    const nextState = cartReducer(stateWithItem, addToCart({ product: mockProduct, quantity: 2 }));
    expect(nextState.items).toHaveLength(1);
    expect(nextState.items[0].quantity).toBe(3);
    expect(nextState.items[0].totalPrice).toBe(450.0);
  });

  it('should handle removeFromCart', () => {
    const stateWithItem: CartState = {
      ...initialState,
      items: [
        {
          id: 'prod-1',
          product: mockProduct,
          quantity: 1,
          unitPrice: 150.0,
          totalPrice: 150.0,
        },
      ],
    };

    const nextState = cartReducer(stateWithItem, removeFromCart('prod-1'));
    expect(nextState.items).toHaveLength(0);
  });

  it('should handle updateQuantity to a positive number', () => {
    const stateWithItem: CartState = {
      ...initialState,
      items: [
        {
          id: 'prod-1',
          product: mockProduct,
          quantity: 1,
          unitPrice: 150.0,
          totalPrice: 150.0,
        },
      ],
    };

    const nextState = cartReducer(stateWithItem, updateQuantity({ id: 'prod-1', quantity: 4 }));
    expect(nextState.items[0].quantity).toBe(4);
    expect(nextState.items[0].totalPrice).toBe(600.0);
  });

  it('should remove item if updateQuantity is set to 0', () => {
    const stateWithItem: CartState = {
      ...initialState,
      items: [
        {
          id: 'prod-1',
          product: mockProduct,
          quantity: 2,
          unitPrice: 150.0,
          totalPrice: 300.0,
        },
      ],
    };

    const nextState = cartReducer(stateWithItem, updateQuantity({ id: 'prod-1', quantity: 0 }));
    expect(nextState.items).toHaveLength(0);
  });

  it('should handle clearCart', () => {
    const stateWithData: CartState = {
      ...initialState,
      items: [
        {
          id: 'prod-1',
          product: mockProduct,
          quantity: 2,
          unitPrice: 150.0,
          totalPrice: 300.0,
        },
      ],
      shippingAmount: 25.0,
      shippingCep: '01310-100',
    };

    const nextState = cartReducer(stateWithData, clearCart());
    expect(nextState.items).toHaveLength(0);
    expect(nextState.shippingAmount).toBe(0);
    expect(nextState.shippingCep).toBe('');
  });

  it('should handle setShipping', () => {
    const nextState = cartReducer(
      initialState,
      setShipping({
        amount: 22.5,
        cep: '01310-100',
        details: { cost: 22.5, deliveryDays: 3, service: 'Padrão' },
      })
    );
    expect(nextState.shippingAmount).toBe(22.5);
    expect(nextState.shippingCep).toBe('01310-100');
    expect(nextState.shippingDetails?.service).toBe('Padrão');
  });

  it('should handle estimateShipping fulfilled', () => {
    const action = {
      type: estimateShipping.fulfilled.type,
      payload: {
        quote: { cost: 35.0, deliveryDays: 2, service: 'Express' },
        cep: '01310-100',
      },
    };
    const nextState = cartReducer(initialState, action);
    expect(nextState.shippingLoading).toBe(false);
    expect(nextState.shippingAmount).toBe(35.0);
    expect(nextState.shippingCep).toBe('01310-100');
    expect(nextState.shippingDetails?.service).toBe('Express');
  });

  it('preserves shipping estimates while recalculating and clears them after failure', () => {
    const staleState: CartState = {
      ...initialState,
      shippingAmount: 35,
      shippingCep: '01310-100',
      shippingDetails: { cost: 35, deliveryDays: 2, service: 'Express' },
    };
    const pendingState = cartReducer(staleState, { type: estimateShipping.pending.type });

    expect(pendingState.shippingAmount).toBe(35);
    expect(pendingState.shippingCep).toBe('01310-100');
    expect(pendingState.shippingDetails?.service).toBe('Express');

    const rejectedState = cartReducer(staleState, {
      type: estimateShipping.rejected.type,
      payload: 'O vendedor não entrega neste CEP',
    });
    expect(rejectedState.shippingAmount).toBe(0);
    expect(rejectedState.shippingCep).toBe('');
    expect(rejectedState.shippingDetails).toBeNull();
    expect(rejectedState.shippingError).toBe('O vendedor não entrega neste CEP');
  });

  it('rejects estimateShipping with the backend message', async () => {
    vi.mocked(cartService.estimateShipping).mockRejectedValueOnce({ message: 'O vendedor 2 não entrega no RJ' });
    const item = { id: 'i', product: mockProduct, variant: { id: '7', name: 'v', sku: 's', price: 1, stock: 1 }, quantity: 1, unitPrice: 1, totalPrice: 1 } as CartItem;
    const result = await estimateShipping({ cep: '20040-020', items: [item] })(vi.fn(), () => ({}), undefined);
    expect(result.type).toBe(estimateShipping.rejected.type);
    expect(result.payload).toBe('O vendedor 2 não entrega no RJ');
    expect(cartService.estimateShipping).toHaveBeenCalledWith('20040-020', [{ skuId: 7, quantity: 1 }]);
  });
});
