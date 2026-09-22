import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { CartItem, Product, ProductVariant, Coupon, ShippingQuote } from '../../types';
import catalogService from '../../services/catalogService';
import cartService from '../../services/cartService';

export interface CartState {
  items: CartItem[];
  isOpen: boolean;
  appliedCoupon: Coupon | null;
  couponLoading: boolean;
  couponError: string | null;
  shippingAmount: number;
  shippingCep: string;
  shippingLoading: boolean;
  shippingError: string | null;
  shippingDetails: ShippingQuote | null;
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
  couponLoading: false,
  couponError: null,
  shippingAmount: 0,
  shippingCep: '',
  shippingLoading: false,
  shippingError: null,
  shippingDetails: null,
};

export const validateAndApplyCoupon = createAsyncThunk(
  'cart/validateAndApplyCoupon',
  async (code: string, { rejectWithValue }) => {
    try {
      const coupon = await catalogService.validateCoupon(code.trim());
      return coupon;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Cupom inválido ou expirado';
      return rejectWithValue(message);
    }
  }
);

export const estimateShipping = createAsyncThunk(
  'cart/estimateShipping',
  async (cep: string, { rejectWithValue }) => {
    try {
      const quote = await cartService.estimateShipping(cep.trim());
      return { quote, cep: cep.trim() };
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Falha ao calcular frete para este CEP';
      return rejectWithValue(message);
    }
  }
);

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
      state.isOpen = true;
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
      state.shippingCep = '';
      state.shippingDetails = null;
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
      state.couponError = null;
    },

    removeCoupon: (state) => {
      state.appliedCoupon = null;
      state.couponError = null;
    },

    setShipping: (
      state,
      action: PayloadAction<{ amount: number; cep: string; details?: ShippingQuote }>
    ) => {
      state.shippingAmount = action.payload.amount;
      state.shippingCep = action.payload.cep;
      if (action.payload.details) {
        state.shippingDetails = action.payload.details;
      }
    },
  },
  extraReducers: (builder) => {
    // Coupon
    builder.addCase(validateAndApplyCoupon.pending, (state) => {
      state.couponLoading = true;
      state.couponError = null;
    });
    builder.addCase(validateAndApplyCoupon.fulfilled, (state, action) => {
      state.couponLoading = false;
      state.appliedCoupon = action.payload;
      state.couponError = null;
    });
    builder.addCase(validateAndApplyCoupon.rejected, (state, action) => {
      state.couponLoading = false;
      state.couponError = action.payload as string;
    });

    // Shipping
    builder.addCase(estimateShipping.pending, (state) => {
      state.shippingLoading = true;
      state.shippingError = null;
    });
    builder.addCase(estimateShipping.fulfilled, (state, action) => {
      state.shippingLoading = false;
      state.shippingAmount = action.payload.quote.cost;
      state.shippingCep = action.payload.cep;
      state.shippingDetails = action.payload.quote;
      state.shippingError = null;
    });
    builder.addCase(estimateShipping.rejected, (state, action) => {
      state.shippingLoading = false;
      state.shippingError = action.payload as string;
    });
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
