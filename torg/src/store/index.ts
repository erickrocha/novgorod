import { configureStore, createSelector } from '@reduxjs/toolkit';
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

export const selectCartSummary = createSelector(
  [
    (state: RootState) => state.cart.items,
    (state: RootState) => state.cart.appliedCoupon,
    (state: RootState) => state.cart.shippingAmount,
  ],
  (items, coupon, shippingAmount) => {
    const subtotal = items.reduce((sum, item) => sum + item.totalPrice, 0);
    const totalItems = items.reduce((sum, item) => sum + item.quantity, 0);

    let discountAmount = 0;
    if (coupon) {
      if (coupon.discountType === 'percentage') {
        discountAmount = (subtotal * coupon.discountValue) / 100;
      } else {
        discountAmount = Math.min(coupon.discountValue, subtotal);
      }
    }

    const total = Math.max(0, subtotal - discountAmount + shippingAmount);

    return {
      subtotal,
      discountAmount,
      shippingAmount,
      total,
      totalItems,
      coupon,
    };
  }
);
