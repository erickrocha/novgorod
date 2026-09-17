import {
  createAsyncThunk,
  createSlice,
  type PayloadAction,
} from "@reduxjs/toolkit";

import { getApiErrorMessage } from "@/services/api";
import { tenantService } from "@/services/tenantService";
import type {
  Tenant,
  TenantInput,
  TenantPlan,
  TenantPlanInput,
} from "@/services/types";

interface TenantState {
  activeTenant: Tenant | null;
  tenantsList: Tenant[];
  loading: boolean;
  error: string | null;
}

interface TenantThunkConfig {
  rejectValue: string;
}

interface UpdateTenantArgs {
  id: number;
  tenantData: TenantInput;
}

interface AddTenantPlanArgs {
  id: number;
  planData: TenantPlanInput;
}

export const fetchTenants = createAsyncThunk<Tenant[], void, TenantThunkConfig>(
  "tenant/fetchTenants",
  async (_, { rejectWithValue }) => {
    try {
      return await tenantService.getTenants();
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao buscar tenants"),
      );
    }
  },
);

export const fetchTenantById = createAsyncThunk<
  Tenant,
  number,
  TenantThunkConfig
>("tenant/fetchTenantById", async (id, { rejectWithValue, dispatch }) => {
  try {
    const tenant = await tenantService.getTenantById(id);
    dispatch(setActiveTenant(tenant));
    return tenant;
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao buscar detalhes do tenant"),
    );
  }
});

export const createTenant = createAsyncThunk<
  Tenant,
  TenantInput | string,
  TenantThunkConfig
>("tenant/createTenant", async (tenantData, { rejectWithValue, dispatch }) => {
  try {
    const payload: TenantInput =
      typeof tenantData === "string" ? { name: tenantData } : tenantData;
    const newTenant = await tenantService.createTenant(payload);
    dispatch(fetchTenants());
    return newTenant;
  } catch (error: unknown) {
    return rejectWithValue(getApiErrorMessage(error, "Falha ao criar tenant"));
  }
});

export const updateTenant = createAsyncThunk<
  Tenant,
  UpdateTenantArgs,
  TenantThunkConfig
>(
  "tenant/updateTenant",
  async ({ id, tenantData }, { rejectWithValue, dispatch }) => {
    try {
      const updatedTenant = await tenantService.updateTenant(id, tenantData);
      dispatch(fetchTenants());
      return updatedTenant;
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao atualizar tenant"),
      );
    }
  },
);

export const addTenantPlan = createAsyncThunk<
  TenantPlan,
  AddTenantPlanArgs,
  TenantThunkConfig
>(
  "tenant/addTenantPlan",
  async ({ id, planData }, { rejectWithValue, dispatch }) => {
    try {
      const response = await tenantService.addTenantPlan(id, planData);
      dispatch(fetchTenants());
      return response;
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao atualizar plano do tenant"),
      );
    }
  },
);

const initialState: TenantState = {
  activeTenant: null,
  tenantsList: [],
  loading: false,
  error: null,
};

const tenantSlice = createSlice({
  name: "tenant",
  initialState,
  reducers: {
    setActiveTenant: (state, action: PayloadAction<Tenant>) => {
      state.activeTenant = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchTenants.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchTenants.fulfilled, (state, action) => {
        state.loading = false;
        state.tenantsList = action.payload;
        if (
          state.activeTenant &&
          !state.tenantsList.some(
            (tenant) => tenant.id === state.activeTenant?.id,
          )
        ) {
          state.activeTenant = null;
        }
      })
      .addCase(fetchTenants.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to fetch tenants";
      })
      .addCase(fetchTenantById.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchTenantById.fulfilled, (state) => {
        state.loading = false;
      })
      .addCase(fetchTenantById.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to fetch tenant";
      })
      .addCase(createTenant.pending, (state) => {
        state.loading = true;
      })
      .addCase(createTenant.fulfilled, (state) => {
        state.loading = false;
      })
      .addCase(createTenant.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to create tenant";
      })
      .addCase(updateTenant.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(updateTenant.fulfilled, (state, action) => {
        state.loading = false;
        if (state.activeTenant?.id === action.payload.id) {
          state.activeTenant = { ...state.activeTenant, ...action.payload };
        }
      })
      .addCase(updateTenant.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to update tenant";
      });
  },
});

export const { setActiveTenant } = tenantSlice.actions;
export default tenantSlice.reducer;
