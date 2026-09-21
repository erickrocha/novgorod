import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { Product } from '../../types';

interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'info';
  message: string;
}

interface UiState {
  mobileMenuOpen: boolean;
  searchOpen: boolean;
  quickViewProduct: Product | null;
  toasts: ToastMessage[];
}

const initialState: UiState = {
  mobileMenuOpen: false,
  searchOpen: false,
  quickViewProduct: null,
  toasts: [],
};

export const uiSlice = createSlice({
  name: 'ui',
  initialState,
  reducers: {
    toggleMobileMenu: (state) => {
      state.mobileMenuOpen = !state.mobileMenuOpen;
    },
    setMobileMenuOpen: (state, action: PayloadAction<boolean>) => {
      state.mobileMenuOpen = action.payload;
    },
    toggleSearch: (state) => {
      state.searchOpen = !state.searchOpen;
    },
    setSearchOpen: (state, action: PayloadAction<boolean>) => {
      state.searchOpen = action.payload;
    },
    openQuickView: (state, action: PayloadAction<Product>) => {
      state.quickViewProduct = action.payload;
    },
    closeQuickView: (state) => {
      state.quickViewProduct = null;
    },
    addToast: (state, action: PayloadAction<Omit<ToastMessage, 'id'>>) => {
      const id = `${Date.now()}-${Math.random()}`;
      state.toasts.push({ ...action.payload, id });
    },
    removeToast: (state, action: PayloadAction<string>) => {
      state.toasts = state.toasts.filter(t => t.id !== action.payload);
    },
  },
});

export const {
  toggleMobileMenu,
  setMobileMenuOpen,
  toggleSearch,
  setSearchOpen,
  openQuickView,
  closeQuickView,
  addToast,
  removeToast,
} = uiSlice.actions;

export default uiSlice.reducer;
