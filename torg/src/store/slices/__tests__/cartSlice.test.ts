import { describe, it, expect, beforeEach, vi } from 'vitest';
import cartReducer, {
  addToCart,
  removeFromCart,
  updateQuantity,
  clearCart,
  applyCoupon,
  removeCoupon,
  setShipping,
  validateAndApplyCoupon,
  estimateShipping,
  type CartState,
} from '../cartSlice';
import type { Product, Coupon } from '../../../types';

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


const mockCoupon: Coupon = {
  code: 'TORG10',
  discountType: 'percentage',
  discountValue: 10,
};

describe('cartSlice', () => {
  let initialState: CartState;

  beforeEach(() => {
    localStorage.clear();
    initialState = {
      items: [],
      isOpen: false,
      appliedCoupon: null,
      couponLoading: false,
      couponError: null,
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
      appliedCoupon: mockCoupon,
      shippingAmount: 25.0,
      shippingCep: '01310-100',
    };

    const nextState = cartReducer(stateWithData, clearCart());
    expect(nextState.items).toHaveLength(0);
    expect(nextState.appliedCoupon).toBeNull();
    expect(nextState.shippingAmount).toBe(0);
    expect(nextState.shippingCep).toBe('');
  });

  it('should handle applyCoupon and removeCoupon', () => {
    const stateWithCoupon = cartReducer(initialState, applyCoupon(mockCoupon));
    expect(stateWithCoupon.appliedCoupon).toEqual(mockCoupon);

    const clearedCouponState = cartReducer(stateWithCoupon, removeCoupon());
    expect(clearedCouponState.appliedCoupon).toBeNull();
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

  it('should handle validateAndApplyCoupon fulfilled', () => {
    const action = {
      type: validateAndApplyCoupon.fulfilled.type,
      payload: mockCoupon,
    };
    const nextState = cartReducer(initialState, action);
    expect(nextState.couponLoading).toBe(false);
    expect(nextState.appliedCoupon).toEqual(mockCoupon);
    expect(nextState.couponError).toBeNull();
  });

  it('should handle validateAndApplyCoupon rejected', () => {
    const action = {
      type: validateAndApplyCoupon.rejected.type,
      payload: 'Cupom inválido ou expirado',
    };
    const nextState = cartReducer(initialState, action);
    expect(nextState.couponLoading).toBe(false);
    expect(nextState.couponError).toBe('Cupom inválido ou expirado');
    expect(nextState.appliedCoupon).toBeNull();
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
});
