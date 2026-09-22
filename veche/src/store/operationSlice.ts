import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { shippingRateService } from "@/services/shippingRateService";
import { taxRuleService } from "@/services/taxRuleService";
import { skuStockService } from "@/services/skuStockService";
import { skuAttributeService } from "@/services/skuAttributeService";
import { productCategoryService } from "@/services/productCategoryService";
import { getApiErrorMessage } from "@/services/api";
import type {
  ShippingRate,
  ShippingRateInput,
  TaxRule,
  TaxRuleInput,
  SkuStock,
  SkuStockInput,
  SkuAttributeValue,
  SkuAttributeValueInput,
  ProductCategory,
  ProductCategoryInput,
  PagedResult,
  PageQueryParams,
} from "@/services/types";

export interface OperationState {
  shippingRatesPaged: PagedResult<ShippingRate> | null;
  taxRulesPaged: PagedResult<TaxRule> | null;
  skuStocksPaged: PagedResult<SkuStock> | null;
  skuAttributesPaged: PagedResult<SkuAttributeValue> | null;
  productCategoriesPaged: PagedResult<ProductCategory> | null;
  loading: boolean;
  error: string | null;
}

const initialState: OperationState = {
  shippingRatesPaged: null,
  taxRulesPaged: null,
  skuStocksPaged: null,
  skuAttributesPaged: null,
  productCategoriesPaged: null,
  loading: false,
  error: null,
};

// Shipping Rates
export const fetchShippingRatesPaged = createAsyncThunk(
  "operation/fetchShippingRatesPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await shippingRateService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load shipping rates"));
    }
  },
);

export const saveShippingRate = createAsyncThunk(
  "operation/saveShippingRate",
  async (
    { id, data }: { id?: number; data: ShippingRateInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await shippingRateService.update(id, data);
      }
      return await shippingRateService.create(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save shipping rate"));
    }
  },
);

// Tax Rules
export const fetchTaxRulesPaged = createAsyncThunk(
  "operation/fetchTaxRulesPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await taxRuleService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load tax rules"));
    }
  },
);

export const saveTaxRule = createAsyncThunk(
  "operation/saveTaxRule",
  async (
    { id, data }: { id?: number; data: TaxRuleInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await taxRuleService.update(id, data);
      }
      return await taxRuleService.create(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save tax rule"));
    }
  },
);

// Sku Stocks (Inventory)
export const fetchSkuStocksPaged = createAsyncThunk(
  "operation/fetchSkuStocksPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await skuStockService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load inventory"));
    }
  },
);

export const saveSkuStock = createAsyncThunk(
  "operation/saveSkuStock",
  async (
    { id, data }: { id?: number; data: SkuStockInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await skuStockService.update(id, data);
      }
      return await skuStockService.create(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save stock"));
    }
  },
);

export const deleteSkuStock = createAsyncThunk(
  "operation/deleteSkuStock",
  async (id: number, { rejectWithValue }) => {
    try {
      await skuStockService.delete(id);
      return id;
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to delete stock"));
    }
  },
);

// Sku Attributes
export const fetchSkuAttributesPaged = createAsyncThunk(
  "operation/fetchSkuAttributesPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await skuAttributeService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load attributes"));
    }
  },
);

export const saveSkuAttribute = createAsyncThunk(
  "operation/saveSkuAttribute",
  async (
    { id, data }: { id?: number; data: SkuAttributeValueInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await skuAttributeService.update(id, data);
      }
      return await skuAttributeService.create(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save attribute"));
    }
  },
);

export const deleteSkuAttribute = createAsyncThunk(
  "operation/deleteSkuAttribute",
  async (id: number, { rejectWithValue }) => {
    try {
      await skuAttributeService.delete(id);
      return id;
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to delete attribute"));
    }
  },
);

// Product Categories
export const fetchProductCategoriesPaged = createAsyncThunk(
  "operation/fetchProductCategoriesPaged",
  async (params: PageQueryParams | undefined, { rejectWithValue }) => {
    try {
      return await productCategoryService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load product categories"));
    }
  },
);

export const saveProductCategory = createAsyncThunk(
  "operation/saveProductCategory",
  async (
    { id, data }: { id?: number; data: ProductCategoryInput },
    { rejectWithValue },
  ) => {
    try {
      if (id) {
        return await productCategoryService.update(id, data);
      }
      return await productCategoryService.create(data);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to save product category"));
    }
  },
);

export const deleteProductCategory = createAsyncThunk(
  "operation/deleteProductCategory",
  async (id: number, { rejectWithValue }) => {
    try {
      await productCategoryService.delete(id);
      return id;
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to delete product category"));
    }
  },
);

export const operationSlice = createSlice({
  name: "operation",
  initialState,
  reducers: {
    clearOperationError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // Shipping Rates
      .addCase(fetchShippingRatesPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchShippingRatesPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.shippingRatesPaged = action.payload;
      })
      .addCase(fetchShippingRatesPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Tax Rules
      .addCase(fetchTaxRulesPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchTaxRulesPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.taxRulesPaged = action.payload;
      })
      .addCase(fetchTaxRulesPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Sku Stocks
      .addCase(fetchSkuStocksPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchSkuStocksPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.skuStocksPaged = action.payload;
      })
      .addCase(fetchSkuStocksPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Sku Attributes
      .addCase(fetchSkuAttributesPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchSkuAttributesPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.skuAttributesPaged = action.payload;
      })
      .addCase(fetchSkuAttributesPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // Product Categories
      .addCase(fetchProductCategoriesPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchProductCategoriesPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.productCategoriesPaged = action.payload;
      })
      .addCase(fetchProductCategoriesPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      });
  },
});

export const { clearOperationError } = operationSlice.actions;
export default operationSlice.reducer;
