import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { getApiErrorMessage } from "@/services/api";
import { locationService } from "@/services/locationService";
import type { City, ImportReport, PagedResult, PageQueryParams, Province } from "@/services/types";

interface LocationState {
  provinces: Province[];
  provincesTotal: number;
  provincesPage: number;
  provincesPageSize: number;
  cities: City[];
  citiesTotal: number;
  citiesPage: number;
  citiesPageSize: number;
  loading: boolean;
  error: string | null;
  report: ImportReport | null;
}

const initialState: LocationState = {
  provinces: [],
  provincesTotal: 0,
  provincesPage: 1,
  provincesPageSize: 10,
  cities: [],
  citiesTotal: 0,
  citiesPage: 1,
  citiesPageSize: 10,
  loading: false,
  error: null,
  report: null,
};

const fail = (e: unknown) => getApiErrorMessage(e, "Location request failed");

export const fetchProvinces = createAsyncThunk<PagedResult<Province>, PageQueryParams | void, { rejectValue: string }>(
  "location/fetchProvinces",
  async (params, { rejectWithValue }) => {
    try {
      return await locationService.provincesPaged(params || { page: 1, pageSize: 10 });
    } catch (e) {
      return rejectWithValue(fail(e));
    }
  },
);

export const fetchCities = createAsyncThunk<PagedResult<City>, PageQueryParams | void, { rejectValue: string }>(
  "location/fetchCities",
  async (params, { rejectWithValue }) => {
    try {
      return await locationService.citiesPaged(params || { page: 1, pageSize: 10 });
    } catch (e) {
      return rejectWithValue(fail(e));
    }
  },
);

export const saveProvince = createAsyncThunk<Province, Province, { rejectValue: string }>("location/saveProvince", async (data, { rejectWithValue, dispatch }) => { try { const r = data.id ? await locationService.updateProvince(data.id, data) : await locationService.createProvince(data); dispatch(fetchProvinces()); return r; } catch (e) { return rejectWithValue(fail(e)); } });
export const saveCity = createAsyncThunk<City, City, { rejectValue: string }>("location/saveCity", async (data, { rejectWithValue, dispatch }) => { try { const r = data.id ? await locationService.updateCity(data.id, data) : await locationService.createCity(data); dispatch(fetchCities()); return r; } catch (e) { return rejectWithValue(fail(e)); } });
export const importProvinces = createAsyncThunk<ImportReport, File, { rejectValue: string }>("location/importProvinces", async (file, { rejectWithValue }) => { try { return await locationService.importProvinces(file); } catch (e) { return rejectWithValue(fail(e)); } });
export const importCities = createAsyncThunk<ImportReport, File, { rejectValue: string }>("location/importCities", async (file, { rejectWithValue }) => { try { return await locationService.importCities(file); } catch (e) { return rejectWithValue(fail(e)); } });

const locationSlice = createSlice({
  name: "location",
  initialState,
  reducers: {
    clearLocationError: (s) => {
      s.error = null;
      s.report = null;
    },
  },
  extraReducers: (b) => {
    for (const t of [fetchProvinces, fetchCities, saveProvince, saveCity, importProvinces, importCities]) {
      b.addCase(t.pending, (s) => {
        s.loading = true;
        s.error = null;
      }).addCase(t.rejected, (s, a) => {
        s.loading = false;
        s.error = a.payload || "Location request failed";
      });
    }
    b.addCase(fetchProvinces.fulfilled, (s, a) => {
      s.loading = false;
      s.provinces = a.payload.items;
      s.provincesTotal = a.payload.total;
      s.provincesPage = a.payload.page;
      s.provincesPageSize = a.payload.pageSize;
    })
    .addCase(fetchCities.fulfilled, (s, a) => {
      s.loading = false;
      s.cities = a.payload.items;
      s.citiesTotal = a.payload.total;
      s.citiesPage = a.payload.page;
      s.citiesPageSize = a.payload.pageSize;
    })
    .addCase(saveProvince.fulfilled, (s) => {
      s.loading = false;
    })
    .addCase(saveCity.fulfilled, (s) => {
      s.loading = false;
    })
    .addCase(importProvinces.fulfilled, (s, a) => {
      s.loading = false;
      s.report = a.payload;
    })
    .addCase(importCities.fulfilled, (s, a) => {
      s.loading = false;
      s.report = a.payload;
    });
  },
});

export const { clearLocationError } = locationSlice.actions;
export default locationSlice.reducer;
