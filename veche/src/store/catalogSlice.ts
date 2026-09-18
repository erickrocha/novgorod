import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import {
  catalogService,
  type Category,
  type CategoryInput,
  type Product,
  type ProductInput,
  type Sku,
  type SkuInput,
} from "@/services/catalogService";
import type { PagedResult, PageQueryParams } from "@/services/types";
import { getApiErrorMessage } from "@/services/api";

interface CatalogState {
  categories: Category[];
  categoriesTotal: number;
  categoriesPage: number;
  products: Product[];
  productsTotal: number;
  productsPage: number;
  skus: Sku[];
  skusTotal: number;
  skusPage: number;
  pageSize: number;
  loading: boolean;
  error: string | null;
}

const initialState: CatalogState = {
  categories: [],
  categoriesTotal: 0,
  categoriesPage: 1,
  products: [],
  productsTotal: 0,
  productsPage: 1,
  skus: [],
  skusTotal: 0,
  skusPage: 1,
  pageSize: 10,
  loading: false,
  error: null,
};

const fail = (e: unknown) => getApiErrorMessage(e, "Catalog request failed");

export const fetchCategories = createAsyncThunk<
  PagedResult<Category>,
  PageQueryParams | void,
  { rejectValue: string }
>("catalog/fetchCategories", async (params, { rejectWithValue }) => {
  try {
    return await catalogService.categoriesPaged(params || { page: 1, pageSize: 10 });
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

export const fetchProducts = createAsyncThunk<
  PagedResult<Product>,
  PageQueryParams | void,
  { rejectValue: string }
>("catalog/fetchProducts", async (params, { rejectWithValue }) => {
  try {
    return await catalogService.productsPaged(params || { page: 1, pageSize: 10 });
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

export const fetchSkus = createAsyncThunk<
  PagedResult<Sku>,
  PageQueryParams | void,
  { rejectValue: string }
>("catalog/fetchSkus", async (params, { rejectWithValue }) => {
  try {
    return await catalogService.skusPaged(params || { page: 1, pageSize: 10 });
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

export const saveCategory = createAsyncThunk<
  Category,
  { id?: number; data: CategoryInput },
  { rejectValue: string }
>("catalog/saveCategory", async ({ id, data }, { rejectWithValue, dispatch }) => {
  try {
    const result = id
      ? await catalogService.updateCategory(id, data)
      : await catalogService.createCategory(data);
    dispatch(fetchCategories());
    return result;
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

export const saveProduct = createAsyncThunk<
  Product,
  { id?: number; data: ProductInput },
  { rejectValue: string }
>("catalog/saveProduct", async ({ id, data }, { rejectWithValue, dispatch }) => {
  try {
    const result = id
      ? await catalogService.updateProduct(id, data)
      : await catalogService.createProduct(data);
    dispatch(fetchProducts());
    return result;
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

export const saveSku = createAsyncThunk<
  Sku,
  { id?: number; data: SkuInput },
  { rejectValue: string }
>("catalog/saveSku", async ({ id, data }, { rejectWithValue, dispatch }) => {
  try {
    const result = id
      ? await catalogService.updateSku(id, data)
      : await catalogService.createSku(data);
    dispatch(fetchSkus());
    return result;
  } catch (e) {
    return rejectWithValue(fail(e));
  }
});

const catalogSlice = createSlice({
  name: "catalog",
  initialState,
  reducers: {
    clearCatalogError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    for (const thunk of [fetchCategories, fetchProducts, fetchSkus]) {
      builder
        .addCase(thunk.pending, (s) => {
          s.loading = true;
          s.error = null;
        })
        .addCase(thunk.rejected, (s, a) => {
          s.loading = false;
          s.error = a.payload || "Catalog request failed";
        });
    }
    builder
      .addCase(fetchCategories.fulfilled, (s, a) => {
        s.loading = false;
        s.categories = a.payload.items;
        s.categoriesTotal = a.payload.total;
        s.categoriesPage = a.payload.page;
        s.pageSize = a.payload.pageSize;
      })
      .addCase(fetchProducts.fulfilled, (s, a) => {
        s.loading = false;
        s.products = a.payload.items;
        s.productsTotal = a.payload.total;
        s.productsPage = a.payload.page;
        s.pageSize = a.payload.pageSize;
      })
      .addCase(fetchSkus.fulfilled, (s, a) => {
        s.loading = false;
        s.skus = a.payload.items;
        s.skusTotal = a.payload.total;
        s.skusPage = a.payload.page;
        s.pageSize = a.payload.pageSize;
      })
      .addCase(saveCategory.fulfilled, (s) => {
        s.loading = false;
      })
      .addCase(saveProduct.fulfilled, (s) => {
        s.loading = false;
      })
      .addCase(saveSku.fulfilled, (s) => {
        s.loading = false;
      })
      .addCase(saveCategory.rejected, (s, a) => {
        s.loading = false;
        s.error = a.payload || "Unable to save category";
      })
      .addCase(saveProduct.rejected, (s, a) => {
        s.loading = false;
        s.error = a.payload || "Unable to save product";
      })
      .addCase(saveSku.rejected, (s, a) => {
        s.loading = false;
        s.error = a.payload || "Unable to save SKU";
      });
  },
});

export const { clearCatalogError } = catalogSlice.actions;
export default catalogSlice.reducer;
