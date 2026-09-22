import { configureStore } from "@reduxjs/toolkit";

import authReducer from "./authSlice";
import tenantReducer from "./tenantSlice";
import userReducer from "./userSlice";
import catalogReducer from "./catalogSlice";
import locationReducer from "./locationSlice";
import uiReducer from "./uiSlice";
import customerReducer from "./customerSlice";
import orderReducer from "./orderSlice";
import cartReducer from "./cartSlice";
import operationReducer from "./operationSlice";
import marketingReducer from "./marketingSlice";

export const store = configureStore({
  reducer: {
    ui: uiReducer,
    tenant: tenantReducer,
    auth: authReducer,
    user: userReducer,
    catalog: catalogReducer,
    location: locationReducer,
    customer: customerReducer,
    order: orderReducer,
    cart: cartReducer,
    operation: operationReducer,
    marketing: marketingReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

