import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { CartItem, Product, ProductVariant, ShippingQuote } from '../../types';
import catalogService from '../../services/catalogService';
import cartService from '../../services/cartService';

export interface CartState {
  items: CartItem[];
  isOpen: boolean;
  shippingAmount: number;
  shippingCep: string;
  shippingLoading: boolean;
  shippingError: string | null;
  shippingDetails: ShippingQuote | null;
  sellerLookupPendingIds: string[];
  sellerLookupFailedIds: string[];
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
  shippingAmount: 0,
  shippingCep: '',
  shippingLoading: false,
  shippingError: null,
  shippingDetails: null,
  sellerLookupPendingIds: [],
  sellerLookupFailedIds: [],
};

export const estimateShipping = createAsyncThunk(
  'cart/estimateShipping',
  async ({ cep, items }: { cep: string; items: CartItem[] }, { rejectWithValue }) => {
    try {
      // The variant id is the SKU id; a product without a variant has a single SKU.
      const lines = await Promise.all(items.map(async item => {
        if (item.variant) return { skuId: Number(item.variant.id), quantity: item.quantity };
        const skus = (await catalogService.getProductBySlug(item.product.slug))?.skus ?? [];
        if (skus.length !== 1) throw new Error('Escolha a variação do produto para calcular o frete');
        return { skuId: skus[0].id, quantity: item.quantity };
      }));
      const quote = await cartService.estimateShipping(cep.trim(), lines);
      return { quote, cep: cep.trim() };
    } catch (err: unknown) {
      // apiClient rejects with an ApiError whose message is the backend's (already translated)
      const message = (err as { message?: string } | null)?.message || 'Falha ao calcular frete para este CEP';
      return rejectWithValue(message);
    }
  }
);

export const enrichCartSeller = createAsyncThunk(
  'cart/enrichSeller',
  async ({ productId, slug }: { productId: string; slug: string }) => {
    const product = await catalogService.getProductBySlug(slug);
    if (!product) throw new Error('Product details unavailable');
    return { productId, seller: product.seller ?? null };
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
        if (product.seller !== undefined) state.items[existingIndex].product.seller = product.seller;
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
      state.shippingAmount = 0;
      state.shippingCep = '';
      state.shippingDetails = null;
      state.sellerLookupPendingIds = [];
      state.sellerLookupFailedIds = [];
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
    builder.addCase(enrichCartSeller.pending, (state, action) => {
      const id = action.meta.arg.productId;
      if (!state.sellerLookupPendingIds.includes(id)) state.sellerLookupPendingIds.push(id);
      state.sellerLookupFailedIds = state.sellerLookupFailedIds.filter(value => value !== id);
    });
    builder.addCase(enrichCartSeller.fulfilled, (state, action) => {
      const { productId, seller } = action.payload;
      state.sellerLookupPendingIds = state.sellerLookupPendingIds.filter(id => id !== productId);
      for (const item of state.items) if (item.product.id === productId) item.product.seller = seller;
      saveCart(state.items);
    });
    builder.addCase(enrichCartSeller.rejected, (state, action) => {
      const id = action.meta.arg.productId;
      state.sellerLookupPendingIds = state.sellerLookupPendingIds.filter(value => value !== id);
      if (!state.sellerLookupFailedIds.includes(id)) state.sellerLookupFailedIds.push(id);
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
      state.shippingAmount = 0;
      state.shippingCep = '';
      state.shippingDetails = null;
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
  setShipping,
} = cartSlice.actions;

export default cartSlice.reducer;
