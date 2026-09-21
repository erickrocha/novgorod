import { configureStore } from '@reduxjs/toolkit';
import cartReducer from './slices/cartSlice';
import catalogReducer from './slices/catalogSlice';
import authReducer from './slices/authSlice';
import uiReducer from './slices/uiSlice';

export const store = configureStore({
  reducer: {
    cart: cartReducer,
    catalog: catalogReducer,
    auth: authReducer,
    ui: uiReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// Helper selectors
export const selectCartItems = (state: RootState) => state.cart.items;
export const selectCartIsOpen = (state: RootState) => state.cart.isOpen;
export const selectAppliedCoupon = (state: RootState) => state.cart.appliedCoupon;
export const selectShippingAmount = (state: RootState) => state.cart.shippingAmount;

export const selectCartSummary = (state: RootState) => {
  const items = state.cart.items;
  const subtotal = items.reduce((sum, item) => sum + item.totalPrice, 0);
  const totalItems = items.reduce((sum, item) => sum + item.quantity, 0);

  let discountAmount = 0;
  const coupon = state.cart.appliedCoupon;
  if (coupon) {
    if (coupon.discountType === 'percentage') {
      discountAmount = (subtotal * coupon.discountValue) / 100;
    } else {
      discountAmount = Math.min(coupon.discountValue, subtotal);
    }
  }

  const shippingAmount = state.cart.shippingAmount;
  const total = Math.max(0, subtotal - discountAmount + shippingAmount);

  return {
    subtotal,
    discountAmount,
    shippingAmount,
    total,
    totalItems,
    coupon,
  };
};
