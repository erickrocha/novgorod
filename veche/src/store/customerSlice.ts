import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { customerService } from "@/services/customerService";
import { getApiErrorMessage } from "@/services/api";
import type {
  Customer,
  CustomerAddress,
  CustomerAddressInput,
  CustomerInput,
  PagedResult,
  PageQueryParams,
} from "@/services/types";

export interface CustomerState {
  customersList: Customer[];
  paged: PagedResult<Customer> | null;
  currentCustomer: Customer | null;
  addresses: CustomerAddress[];
  loading: boolean;
  error: string | null;
}

const initialState: CustomerState = {
  customersList: [],
  paged: null,
  currentCustomer: null,
  addresses: [],
  loading: false,
  error: null,
};

export const fetchCustomers = createAsyncThunk(
  "customer/fetchCustomers",
  async (_, { rejectWithValue }) => {
    try {
      return await customerService.list();
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load customers"),
      );
    }
  },
);

export const fetchCustomersPaged = createAsyncThunk(
  "customer/fetchCustomersPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await customerService.paged(params);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load customers page"),
      );
    }
  },
);

export const fetchCustomerById = createAsyncThunk(
  "customer/fetchCustomerById",
  async (id: number, { rejectWithValue }) => {
    try {
      return await customerService.getById(id);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load customer details"),
      );
    }
  },
);

export const createCustomer = createAsyncThunk(
  "customer/createCustomer",
  async (data: CustomerInput, { rejectWithValue }) => {
    try {
      return await customerService.create(data);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to create customer"),
      );
    }
  },
);

export const updateCustomer = createAsyncThunk(
  "customer/updateCustomer",
  async ({ id, data }: { id: number; data: CustomerInput }, { rejectWithValue }) => {
    try {
      return await customerService.update(id, data);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to update customer"),
      );
    }
  },
);

export const fetchCustomerAddresses = createAsyncThunk(
  "customer/fetchCustomerAddresses",
  async (
    params: (PageQueryParams & { customerId?: number }) | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await customerService.addressesPaged(params);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to load customer addresses"),
      );
    }
  },
);

export const createCustomerAddress = createAsyncThunk(
  "customer/createCustomerAddress",
  async (data: CustomerAddressInput, { rejectWithValue }) => {
    try {
      return await customerService.createAddress(data);
    } catch (error) {
      return rejectWithValue(
        getApiErrorMessage(error, "Failed to create customer address"),
      );
    }
  },
);

export const customerSlice = createSlice({
  name: "customer",
  initialState,
  reducers: {
    clearCustomerError(state) {
      state.error = null;
    },
    setCurrentCustomer(state, action) {
      state.currentCustomer = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      // fetchCustomers
      .addCase(fetchCustomers.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCustomers.fulfilled, (state, action) => {
        state.loading = false;
        state.customersList = action.payload;
      })
      .addCase(fetchCustomers.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // fetchCustomersPaged
      .addCase(fetchCustomersPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCustomersPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.paged = action.payload;
        state.customersList = action.payload.items;
      })
      .addCase(fetchCustomersPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // fetchCustomerById
      .addCase(fetchCustomerById.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCustomerById.fulfilled, (state, action) => {
        state.loading = false;
        state.currentCustomer = action.payload;
      })
      .addCase(fetchCustomerById.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // fetchCustomerAddresses
      .addCase(fetchCustomerAddresses.fulfilled, (state, action) => {
        state.addresses = action.payload.items;
      });
  },
});

export const { clearCustomerError, setCurrentCustomer } = customerSlice.actions;
export default customerSlice.reducer;
