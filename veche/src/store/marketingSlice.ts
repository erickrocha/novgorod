import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { marketingService } from "@/services/marketingService";
import { getApiErrorMessage } from "@/services/api";
import type {
  Campaign,
  CampaignInput,
  CampaignTarget,
  CampaignTargetInput,
  Coupon,
  CouponInput,
  CouponRedemption,
  PagedResult,
  PageQueryParams,
} from "@/services/types";

export interface MarketingState {
  campaignsPaged: PagedResult<Campaign> | null;
  targetsPaged: PagedResult<CampaignTarget> | null;
  couponsPaged: PagedResult<Coupon> | null;
  redemptionsPaged: PagedResult<CouponRedemption> | null;
  loading: boolean;
  error: string | null;
}

const initialState: MarketingState = {
  campaignsPaged: null,
  targetsPaged: null,
  couponsPaged: null,
  redemptionsPaged: null,
  loading: false,
  error: null,
};

export const fetchCampaignsPaged = createAsyncThunk(
  "marketing/fetchCampaignsPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await marketingService.campaignsPaged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load campaigns"));
    }
  },
);

export const saveCampaign = createAsyncThunk(
  "marketing/saveCampaign",
  async (
    { id, data }: { id?: number; data: CampaignInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await marketingService.updateCampaign(id, data);
      }
      return await marketingService.createCampaign(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save campaign"));
    }
  },
);

export const fetchTargetsPaged = createAsyncThunk(
  "marketing/fetchTargetsPaged",
  async (
    params: (PageQueryParams & { campaignId?: number }) | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await marketingService.targetsPaged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load campaign targets"));
    }
  },
);

export const createTarget = createAsyncThunk(
  "marketing/createTarget",
  async (data: CampaignTargetInput, { rejectWithValue }) => {
    try {
      return await marketingService.createTarget(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to create target"));
    }
  },
);

export const fetchCouponsPaged = createAsyncThunk(
  "marketing/fetchCouponsPaged",
  async (
    params: (PageQueryParams & { code?: string }) | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await marketingService.couponsPaged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load coupons"));
    }
  },
);

export const saveCoupon = createAsyncThunk(
  "marketing/saveCoupon",
  async (
    { id, data }: { id?: number; data: CouponInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await marketingService.updateCoupon(id, data);
      }
      return await marketingService.createCoupon(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save coupon"));
    }
  },
);

export const fetchRedemptionsPaged = createAsyncThunk(
  "marketing/fetchRedemptionsPaged",
  async (
    params:
      | (PageQueryParams & { couponId?: number; customerId?: number })
      | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await marketingService.redemptionsPaged(params);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load coupon redemptions"),
      );
    }
  },
);

export const marketingSlice = createSlice({
  name: "marketing",
  initialState,
  reducers: {
    clearMarketingError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // Campaigns
      .addCase(fetchCampaignsPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCampaignsPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.campaignsPaged = action.payload;
      })
      .addCase(fetchCampaignsPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Targets
      .addCase(fetchTargetsPaged.fulfilled, (state, action) => {
        state.targetsPaged = action.payload;
      })
      // Coupons
      .addCase(fetchCouponsPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCouponsPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.couponsPaged = action.payload;
      })
      .addCase(fetchCouponsPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Redemptions
      .addCase(fetchRedemptionsPaged.fulfilled, (state, action) => {
        state.redemptionsPaged = action.payload;
      });
  },
});

export const { clearMarketingError } = marketingSlice.actions;
export default marketingSlice.reducer;
