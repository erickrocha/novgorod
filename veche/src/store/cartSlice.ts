import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { cartService } from "@/services/cartService";
import { getApiErrorMessage } from "@/services/api";
import type {
  Cart,
  CartItem,
  PagedResult,
  PageQueryParams,
} from "@/services/types";

export interface CartState {
  cartsList: Cart[];
  paged: PagedResult<Cart> | null;
  selectedCart: Cart | null;
  cartItems: CartItem[];
  loading: boolean;
  error: string | null;
}

const initialState: CartState = {
  cartsList: [],
  paged: null,
  selectedCart: null,
  cartItems: [],
  loading: false,
  error: null,
};

export const fetchCartsPaged = createAsyncThunk(
  "cart/fetchCartsPaged",
  async (
    params: (PageQueryParams & { customerId?: number; status?: string }) | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await cartService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load carts"));
    }
  },
);

export const fetchCartItems = createAsyncThunk(
  "cart/fetchCartItems",
  async (cart: Cart, { rejectWithValue }) => {
    try {
      const itemsRes = await cartService.itemsPaged({ cartId: cart.id, pageSize: 50 });
      return {
        cart,
        items: itemsRes.items,
      };
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load cart items"));
    }
  },
);

export const cartSlice = createSlice({
  name: "cart",
  initialState,
  reducers: {
    clearSelectedCart(state) {
      state.selectedCart = null;
      state.cartItems = [];
    },
    clearCartError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // fetchCartsPaged
      .addCase(fetchCartsPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCartsPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.paged = action.payload;
        state.cartsList = action.payload.items;
      })
      .addCase(fetchCartsPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // fetchCartItems
      .addCase(fetchCartItems.fulfilled, (state, action) => {
        state.selectedCart = action.payload.cart;
        state.cartItems = action.payload.items;
      });
  },
});

export const { clearSelectedCart, clearCartError } = cartSlice.actions;
export default cartSlice.reducer;
