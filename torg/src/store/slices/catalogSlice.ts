import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { Product, ProductCategory, ProductFilterState } from '../../types';
import catalogService, { type GetProductsParams } from '../../services/catalogService';

interface CatalogState {
  products: Product[];
  featuredProducts: Product[];
  categories: ProductCategory[];
  selectedProduct: Product | null;
  totalProducts: number;
  nextCursor: number | null;
  isLoading: boolean;
  isLoadingProduct: boolean;
  error: string | null;
  filters: ProductFilterState;
  activeRequestId: string | null;
}

const initialFilters: ProductFilterState = {
  searchQuery: '',
  category: 'all',
  minPrice: undefined,
  maxPrice: undefined,
  sortBy: 'newest',
};

const initialState: CatalogState = {
  products: [],
  featuredProducts: [],
  categories: [],
  selectedProduct: null,
  totalProducts: 0,
  nextCursor: null,
  isLoading: false,
  isLoadingProduct: false,
  error: null,
  filters: initialFilters,
  activeRequestId: null,
};

export const fetchProducts = createAsyncThunk(
  'catalog/fetchProducts',
  async (customParams: Partial<GetProductsParams> & { loadMore?: boolean } | undefined, { getState, rejectWithValue }) => {
    try {
      const state = getState() as { catalog: CatalogState };
      const currentFilters = state.catalog.filters;
      
      const cursor = customParams?.loadMore ? state.catalog.nextCursor : undefined;

      const params: GetProductsParams = {
        category: customParams?.category !== undefined ? customParams.category : currentFilters.category,
        query: customParams?.query !== undefined ? customParams.query : currentFilters.searchQuery,
        minPrice: customParams?.minPrice !== undefined ? customParams.minPrice : currentFilters.minPrice,
        maxPrice: customParams?.maxPrice !== undefined ? customParams.maxPrice : currentFilters.maxPrice,
        sortBy: customParams?.sortBy !== undefined ? customParams.sortBy : currentFilters.sortBy,
        cursor,
        limit: customParams?.limit || 12,
      };

      const result = await catalogService.getProducts(params);
      return result;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Falha ao buscar catálogo';
      return rejectWithValue(message);
    }
  },
  {
    condition: (params: Partial<GetProductsParams> & { loadMore?: boolean } | undefined, { getState }) => {
      if (!params?.loadMore) return true;
      const { isLoading, nextCursor } = (getState() as { catalog: CatalogState }).catalog;
      return !isLoading && nextCursor !== null;
    },
  }
);

export const fetchCategories = createAsyncThunk(
  'catalog/fetchCategories',
  async (_, { rejectWithValue }) => {
    try {
      return await catalogService.getCategories();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Falha ao carregar categorias';
      return rejectWithValue(message);
    }
  }
);

export const fetchFeaturedProducts = createAsyncThunk(
  'catalog/fetchFeaturedProducts',
  async (_, { rejectWithValue }) => {
    try {
      return await catalogService.getFeaturedProducts();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Falha ao buscar destaques';
      return rejectWithValue(message);
    }
  }
);

export const fetchProductBySlug = createAsyncThunk(
  'catalog/fetchProductBySlug',
  async (slug: string, { rejectWithValue }) => {
    try {
      const product = await catalogService.getProductBySlug(slug);
      if (!product) throw new Error('Produto não encontrado');
      return product;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Falha ao carregar produto';
      return rejectWithValue(message);
    }
  }
);

export const catalogSlice = createSlice({
  name: 'catalog',
  initialState,
  reducers: {
    setFilters: (state, action: PayloadAction<ProductFilterState>) => {
      state.filters = action.payload;
    },
    setSearchQuery: (state, action: PayloadAction<string>) => {
      state.filters.searchQuery = action.payload;
    },
    setCategory: (state, action: PayloadAction<string>) => {
      state.filters.category = action.payload;
    },
    setSortBy: (state, action: PayloadAction<ProductFilterState['sortBy']>) => {
      state.filters.sortBy = action.payload;
    },
    setPriceRange: (state, action: PayloadAction<{ min: number; max: number }>) => {
      state.filters.minPrice = action.payload.min;
      state.filters.maxPrice = action.payload.max;
    },
    resetFilters: (state) => {
      state.filters = { ...initialFilters };
    },
    clearSelectedProduct: (state) => {
      state.selectedProduct = null;
    },
  },
  extraReducers: (builder) => {
    // Products
    builder.addCase(fetchProducts.pending, (state, action) => {
      if (action.meta.arg?.loadMore && (state.isLoading || state.nextCursor === null)) return;
      state.activeRequestId = action.meta.requestId;
      state.isLoading = true;
      state.error = null;
      if (!action.meta.arg?.loadMore) {
        state.products = [];
        state.totalProducts = 0;
        state.nextCursor = null;
      }
    });
    builder.addCase(fetchProducts.fulfilled, (state, action) => {
      if (state.activeRequestId !== action.meta.requestId) return;
      state.activeRequestId = null;
      state.isLoading = false;
      if (action.meta.arg?.loadMore) {
        state.products = [...state.products, ...action.payload.items];
      } else {
        state.products = action.payload.items;
      }
      if (action.payload.total !== undefined) {
        state.totalProducts = action.payload.total;
      }
      state.nextCursor = action.payload.nextCursor ?? null;
    });
    builder.addCase(fetchProducts.rejected, (state, action) => {
      if (state.activeRequestId !== action.meta.requestId) return;
      state.activeRequestId = null;
      state.isLoading = false;
      state.error = action.payload as string;
    });

    // Categories
    builder.addCase(fetchCategories.fulfilled, (state, action) => {
      state.categories = action.payload;
    });

    // Featured
    builder.addCase(fetchFeaturedProducts.fulfilled, (state, action) => {
      state.featuredProducts = action.payload;
    });

    // Product Detail
    builder.addCase(fetchProductBySlug.pending, (state) => {
      state.isLoadingProduct = true;
      state.error = null;
    });
    builder.addCase(fetchProductBySlug.fulfilled, (state, action) => {
      state.isLoadingProduct = false;
      state.selectedProduct = action.payload;
    });
    builder.addCase(fetchProductBySlug.rejected, (state, action) => {
      state.isLoadingProduct = false;
      state.error = action.payload as string;
    });
  },
});

export const {
  setFilters,
  setSearchQuery,
  setCategory,
  setSortBy,
  setPriceRange,
  resetFilters,
  clearSelectedProduct,
} = catalogSlice.actions;

export default catalogSlice.reducer;
