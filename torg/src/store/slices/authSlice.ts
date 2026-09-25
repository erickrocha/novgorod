import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

export interface CustomerProfile {
  id: string;
  name: string;
  cpf?: string | null;
  email: string;
  phone?: string;
  addresses?: Array<{
    id: string;
    label?: string;
    recipient?: string;
    addressLine1?: string;
    addressLine2?: string;
    locality?: string;
    administrativeArea?: string;
    postalCode?: string;
    countryCode?: string;
    isDefault?: boolean;
  }>;
}

interface AuthState {
  customer: CustomerProfile | null;
  token: string | null;
  isAuthenticated: boolean;
}

const initialToken = localStorage.getItem('torg_customer_token');
const savedCustomer = localStorage.getItem('torg_customer_profile');

const initialState: AuthState = {
  customer: savedCustomer ? JSON.parse(savedCustomer) : null,
  token: initialToken,
  isAuthenticated: Boolean(initialToken),
};

export const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    setCustomerSession: (
      state,
      action: PayloadAction<{ customer: CustomerProfile; token: string }>
    ) => {
      state.customer = action.payload.customer;
      state.token = action.payload.token;
      state.isAuthenticated = true;

      localStorage.setItem('torg_customer_token', action.payload.token);
      localStorage.setItem('torg_customer_profile', JSON.stringify(action.payload.customer));
    },
    logout: (state) => {
      state.customer = null;
      state.token = null;
      state.isAuthenticated = false;

      localStorage.removeItem('torg_customer_token');
      localStorage.removeItem('torg_customer_profile');
    },
  },
});

export const { setCustomerSession, logout } = authSlice.actions;
export default authSlice.reducer;
