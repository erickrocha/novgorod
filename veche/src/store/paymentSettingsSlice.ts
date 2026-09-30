import { createAsyncThunk, createSlice, type PayloadAction } from "@reduxjs/toolkit";
import { paymentSettingsService } from "@/services/paymentSettingsService";
import { getApiErrorMessage } from "@/services/api";
import type { PaymentSettingsResponse, PaymentSettingsUpdate } from "@/services/types";

export interface PaymentSettingsState {
  settings: PaymentSettingsResponse | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  saveSuccess: boolean;
}

const initialState: PaymentSettingsState = {
  settings: null,
  loading: false,
  saving: false,
  error: null,
  saveSuccess: false,
};

export const fetchPaymentSettings = createAsyncThunk<
  PaymentSettingsResponse,
  number,
  { rejectValue: string }
>(
  "paymentSettings/fetchPaymentSettings",
  async (tenantId, { rejectWithValue }) => {
    try {
      return await paymentSettingsService.getSettings(tenantId);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load payment settings")
      );
    }
  }
);

export const updatePaymentSettings = createAsyncThunk<
  void,
  { tenantId: number; data: PaymentSettingsUpdate },
  { rejectValue: string }
>(
  "paymentSettings/updatePaymentSettings",
  async ({ tenantId, data }, { rejectWithValue, dispatch }) => {
    try {
      await paymentSettingsService.updateSettings(tenantId, data);
      await dispatch(fetchPaymentSettings(tenantId));
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to update payment settings")
      );
    }
  }
);

export const paymentSettingsSlice = createSlice({
  name: "paymentSettings",
  initialState,
  reducers: {
    clearPaymentSettingsStatus: (state) => {
      state.error = null;
      state.saveSuccess = false;
    },
    resetPaymentSettings: () => initialState,
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchPaymentSettings.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(
        fetchPaymentSettings.fulfilled,
        (state, action: PayloadAction<PaymentSettingsResponse>) => {
          state.loading = false;
          state.settings = action.payload;
        }
      )
      .addCase(fetchPaymentSettings.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ?? "Failed to load payment settings";
      })
      .addCase(updatePaymentSettings.pending, (state) => {
        state.saving = true;
        state.error = null;
        state.saveSuccess = false;
      })
      .addCase(updatePaymentSettings.fulfilled, (state) => {
        state.saving = false;
        state.saveSuccess = true;
      })
      .addCase(updatePaymentSettings.rejected, (state, action) => {
        state.saving = false;
        state.error = action.payload ?? "Failed to update payment settings";
      });
  },
});

export const { clearPaymentSettingsStatus, resetPaymentSettings } =
  paymentSettingsSlice.actions;

export default paymentSettingsSlice.reducer;
