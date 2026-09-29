import { createAsyncThunk, createSlice, type PayloadAction } from "@reduxjs/toolkit";
import { shippingSettingsService } from "@/services/shippingSettingsService";
import { getApiErrorMessage } from "@/services/api";
import type { ShippingSettingsResponse, ShippingSettingsUpdate } from "@/services/types";

export interface ShippingSettingsState {
  settings: ShippingSettingsResponse | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  saveSuccess: boolean;
}

const initialState: ShippingSettingsState = {
  settings: null,
  loading: false,
  saving: false,
  error: null,
  saveSuccess: false,
};

export const fetchShippingSettings = createAsyncThunk<
  ShippingSettingsResponse,
  number,
  { rejectValue: string }
>(
  "shippingSettings/fetchShippingSettings",
  async (tenantId, { rejectWithValue }) => {
    try {
      return await shippingSettingsService.getSettings(tenantId);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load shipping settings")
      );
    }
  }
);

export const updateShippingSettings = createAsyncThunk<
  void,
  { tenantId: number; data: ShippingSettingsUpdate },
  { rejectValue: string }
>(
  "shippingSettings/updateShippingSettings",
  async ({ tenantId, data }, { rejectWithValue, dispatch }) => {
    try {
      await shippingSettingsService.updateSettings(tenantId, data);
      await dispatch(fetchShippingSettings(tenantId));
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to update shipping settings")
      );
    }
  }
);

export const shippingSettingsSlice = createSlice({
  name: "shippingSettings",
  initialState,
  reducers: {
    clearShippingSettingsStatus: (state) => {
      state.error = null;
      state.saveSuccess = false;
    },
    resetShippingSettings: () => initialState,
  },
  extraReducers: (builder) => {
    builder
      // Fetch
      .addCase(fetchShippingSettings.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(
        fetchShippingSettings.fulfilled,
        (state, action: PayloadAction<ShippingSettingsResponse>) => {
          state.loading = false;
          state.settings = action.payload;
        }
      )
      .addCase(fetchShippingSettings.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ?? "Failed to load shipping settings";
      })
      // Update
      .addCase(updateShippingSettings.pending, (state) => {
        state.saving = true;
        state.error = null;
        state.saveSuccess = false;
      })
      .addCase(updateShippingSettings.fulfilled, (state) => {
        state.saving = false;
        state.saveSuccess = true;
      })
      .addCase(updateShippingSettings.rejected, (state, action) => {
        state.saving = false;
        state.error = action.payload ?? "Failed to update shipping settings";
      });
  },
});

export const { clearShippingSettingsStatus, resetShippingSettings } =
  shippingSettingsSlice.actions;

export default shippingSettingsSlice.reducer;
