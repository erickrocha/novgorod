import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { CartItem, Product, ProductVariant, Coupon } from '../../types';

interface CartState {
  items: CartItem[];
  isOpen: boolean;
  appliedCoupon: Coupon | null;
  shippingAmount: number;
  shippingCep: string;
}

const STORAGE_KEY = 'torg_cart_items';

const loadSavedCart = (): CartItem[] => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved ? JSON.parse(saved) : [];
  } catch {
    return [];
  }
};

const saveCart = (items: CartItem[]) => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
  } catch {
    // Ignore storage quota errors
  }
};

const initialState: CartState = {
  items: loadSavedCart(),
  isOpen: false,
  appliedCoupon: null,
  shippingAmount: 0,
  shippingCep: '',
};

export const cartSlice = createSlice({
  name: 'cart',
  initialState,
  reducers: {
    addToCart: (
      state,
      action: PayloadAction<{ product: Product; variant?: ProductVariant; quantity?: number }>
    ) => {
      const { product, variant, quantity = 1 } = action.payload;
      const unitPrice = variant ? variant.price : product.price;
      const itemId = variant ? `${product.id}-${variant.id}` : product.id;

      const existingIndex = state.items.findIndex(item => item.id === itemId);

      if (existingIndex > -1) {
        state.items[existingIndex].quantity += quantity;
        state.items[existingIndex].totalPrice = state.items[existingIndex].quantity * state.items[existingIndex].unitPrice;
      } else {
        state.items.push({
          id: itemId,
          product,
          variant,
          quantity,
          unitPrice,
          totalPrice: unitPrice * quantity,
        });
      }

      saveCart(state.items);
      state.isOpen = true; // Automatically open cart drawer for immediate user feedback
    },

    removeFromCart: (state, action: PayloadAction<string>) => {
      state.items = state.items.filter(item => item.id !== action.payload);
      saveCart(state.items);
    },

    updateQuantity: (
      state,
      action: PayloadAction<{ id: string; quantity: number }>
    ) => {
      const { id, quantity } = action.payload;
      if (quantity <= 0) {
        state.items = state.items.filter(item => item.id !== id);
      } else {
        const item = state.items.find(item => item.id === id);
        if (item) {
          item.quantity = quantity;
          item.totalPrice = item.quantity * item.unitPrice;
        }
      }
      saveCart(state.items);
    },

    clearCart: (state) => {
      state.items = [];
      state.appliedCoupon = null;
      state.shippingAmount = 0;
      saveCart([]);
    },

    openCart: (state) => {
      state.isOpen = true;
    },

    closeCart: (state) => {
      state.isOpen = false;
    },

    toggleCart: (state) => {
      state.isOpen = !state.isOpen;
    },

    applyCoupon: (state, action: PayloadAction<Coupon>) => {
      state.appliedCoupon = action.payload;
    },

    removeCoupon: (state) => {
      state.appliedCoupon = null;
    },

    setShipping: (
      state,
      action: PayloadAction<{ amount: number; cep: string }>
    ) => {
      state.shippingAmount = action.payload.amount;
      state.shippingCep = action.payload.cep;
    },
  },
});

export const {
  addToCart,
  removeFromCart,
  updateQuantity,
  clearCart,
  openCart,
  closeCart,
  toggleCart,
  applyCoupon,
  removeCoupon,
  setShipping,
} = cartSlice.actions;

export default cartSlice.reducer;
