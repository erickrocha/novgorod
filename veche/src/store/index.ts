import { configureStore } from "@reduxjs/toolkit";

import authReducer from "./authSlice";
import tenantReducer from "./tenantSlice";
import userReducer from "./userSlice";
import catalogReducer from "./catalogSlice";
import locationReducer from "./locationSlice";

export const store = configureStore({
  reducer: {
    tenant: tenantReducer,
    auth: authReducer,
    user: userReducer,
    catalog: catalogReducer,
    location: locationReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
