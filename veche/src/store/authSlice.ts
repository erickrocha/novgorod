import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";

import { authService } from "@/services/authService";
import { resourceService } from "@/services/resourceService";
import { getApiErrorMessage } from "@/services/api";
import type {
  AuthResponse,
  AuthSession,
  Person,
  PersonAddress,
  PersonAddressInput,
  PersonInput,
  ResourceProfile,
} from "@/services/types";
import { ROLES } from "@/utils/enums";
import { isTokenExpired } from "@/utils/jwt";
import { fetchTenantById } from "./tenantSlice";

interface LoginCredentials {
  email: string;
  password: string;
}

interface AuthState {
  token: string | null;
  refreshToken: string | null;
  user: AuthResponse | null;
  person: Person | null;
  addresses: PersonAddress[];
  isAuthenticated: boolean;
  isInitializing: boolean;
  isSysAdmin: boolean;
  loading: boolean;
  error: string | null;
}

interface AuthThunkConfig {
  rejectValue: string;
}

const clearStoredAuth = () => {
  localStorage.removeItem("token");
  localStorage.removeItem("refreshToken");
  localStorage.removeItem("user");
  localStorage.removeItem("person");
};

const readStoredUser = (): AuthResponse | null => {
  const storedUser = localStorage.getItem("user");
  return storedUser ? (JSON.parse(storedUser) as AuthResponse) : null;
};

const readStoredPerson = (): Person | null => {
  const stored = localStorage.getItem("person");
  return stored ? (JSON.parse(stored) as Person) : null;
};

const getAccessToken = (data: AuthResponse): string =>
  data.accessToken || data.access_token || "";
const getRefreshToken = (data: AuthResponse): string | null =>
  data.refreshToken || data.refresh_token || null;
const getTenantId = (data: AuthResponse): number | null | undefined =>
  data.tenantId ?? data.tenant_id;

export const fetchProfile = createAsyncThunk<
  ResourceProfile,
  void,
  AuthThunkConfig
>("auth/fetchProfile", async (_, { rejectWithValue }) => {
  try {
    const data = await resourceService.getProfile();
    if (data.person) {
      localStorage.setItem("person", JSON.stringify(data.person));
    }
    return data;
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao carregar perfil do usuário"),
    );
  }
});

export const updateUserProfile = createAsyncThunk<
  Person,
  PersonInput,
  AuthThunkConfig
>("auth/updateUserProfile", async (personData, { rejectWithValue }) => {
  try {
    const data = await resourceService.updateProfile(personData);
    localStorage.setItem("person", JSON.stringify(data));
    return data;
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao atualizar perfil"),
    );
  }
});

export const loginUser = createAsyncThunk<
  AuthResponse,
  LoginCredentials,
  AuthThunkConfig
>(
  "auth/loginUser",
  async ({ email, password }, { rejectWithValue, dispatch }) => {
    try {
      const data = await authService.login(email, password);
      const token = getAccessToken(data);
      const refreshToken = getRefreshToken(data);

      if (token) localStorage.setItem("token", token);
      if (refreshToken) localStorage.setItem("refreshToken", refreshToken);
      localStorage.setItem("user", JSON.stringify(data));

      const tenantId = getTenantId(data);
      if (tenantId) dispatch(fetchTenantById(tenantId));
      dispatch(fetchProfile());
      return data;
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(
          error,
          "Email ou senha inválidos. Por favor, tente novamente.",
        ),
      );
    }
  },
);

export const validateOrRefreshToken = createAsyncThunk<
  AuthSession,
  void,
  AuthThunkConfig
>("auth/validateOrRefreshToken", async (_, { rejectWithValue, dispatch }) => {
  const token = localStorage.getItem("token");
  const refreshToken = localStorage.getItem("refreshToken");
  const storedUser = readStoredUser();

    if (token && !isTokenExpired(token)) {
      if (storedUser) {
        const tenantId = getTenantId(storedUser);
        if (tenantId) dispatch(fetchTenantById(tenantId));
      }
      dispatch(fetchProfile());
      return { token, refreshToken, user: storedUser };
  }

  if (refreshToken && !isTokenExpired(refreshToken)) {
    try {
      const data = await authService.refreshToken(refreshToken);
      const newToken = getAccessToken(data);
      const newRefreshToken = getRefreshToken(data) || refreshToken;

      if (!newToken) {
        clearStoredAuth();
        return rejectWithValue("Refresh token expired or invalid");
      }

      localStorage.setItem("token", newToken);
      localStorage.setItem("refreshToken", newRefreshToken);
      localStorage.setItem("user", JSON.stringify(data));

      const tenantId = getTenantId(data);
      if (tenantId) dispatch(fetchTenantById(tenantId));
      dispatch(fetchProfile());

      return { token: newToken, refreshToken: newRefreshToken, user: data };
    } catch (error: unknown) {
      console.error("Refresh token validation failed:", error);
      clearStoredAuth();
      return rejectWithValue("Refresh token expired or invalid");
    }
  }

  clearStoredAuth();
  return rejectWithValue("No valid session or token expired");
});

export const fetchPersonAddresses = createAsyncThunk<
  PersonAddress[],
  number,
  AuthThunkConfig
>("auth/fetchPersonAddresses", async (personId, { rejectWithValue }) => {
  try {
    return await resourceService.getAddressesByPerson(personId);
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao carregar endereços"),
    );
  }
});

export const addPersonAddress = createAsyncThunk<
  PersonAddress,
  PersonAddressInput,
  AuthThunkConfig
>("auth/addPersonAddress", async (input, { rejectWithValue }) => {
  try {
    return await resourceService.addAddress(input);
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao adicionar endereço"),
    );
  }
});

export const updatePersonAddress = createAsyncThunk<
  PersonAddress,
  { id: number; input: PersonAddressInput },
  AuthThunkConfig
>("auth/updatePersonAddress", async ({ id, input }, { rejectWithValue }) => {
  try {
    return await resourceService.updateAddress(id, input);
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao atualizar endereço"),
    );
  }
});

export const deletePersonAddress = createAsyncThunk<
  number,
  number,
  AuthThunkConfig
>("auth/deletePersonAddress", async (id, { rejectWithValue }) => {
  try {
    await resourceService.deleteAddress(id);
    return id;
  } catch (error: unknown) {
    return rejectWithValue(
      getApiErrorMessage(error, "Falha ao remover endereço"),
    );
  }
});

const initialUser = readStoredUser();

const initialState: AuthState = {
  token: localStorage.getItem("token"),
  refreshToken: localStorage.getItem("refreshToken"),
  user: initialUser,
  person: readStoredPerson(),
  addresses: [],
  isAuthenticated: false,
  isInitializing: true,
  isSysAdmin: initialUser?.role === ROLES.SYS_ADMIN,
  loading: false,
  error: null,
};

const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    logout: (state) => {
      state.token = null;
      state.refreshToken = null;
      state.user = null;
      state.person = null;
      state.addresses = [];
      state.isAuthenticated = false;
      state.isSysAdmin = false;
      state.isInitializing = false;
      clearStoredAuth();
    },
    clearError: (state) => {
      state.error = null;
    },
    setAddresses: (state, action: PayloadAction<PersonAddress[]>) => {
      state.addresses = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(loginUser.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(loginUser.fulfilled, (state, action) => {
        state.loading = false;
        state.token = getAccessToken(action.payload);
        state.refreshToken = getRefreshToken(action.payload);
        state.user = action.payload;
        state.isAuthenticated = true;
        state.isSysAdmin = action.payload.role === ROLES.SYS_ADMIN;
      })
      .addCase(loginUser.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to sign in";
      })
      .addCase(validateOrRefreshToken.pending, (state) => {
        state.isInitializing = true;
      })
      .addCase(validateOrRefreshToken.fulfilled, (state, action) => {
        state.isInitializing = false;
        state.token = action.payload.token;
        state.refreshToken = action.payload.refreshToken;
        state.user = action.payload.user;
        state.isAuthenticated = true;
        state.isSysAdmin = action.payload.user?.role === ROLES.SYS_ADMIN;
      })
      .addCase(validateOrRefreshToken.rejected, (state) => {
        state.isInitializing = false;
        state.token = null;
        state.refreshToken = null;
        state.user = null;
        state.person = null;
        state.addresses = [];
        state.isAuthenticated = false;
        state.isSysAdmin = false;
      })
      .addCase(fetchProfile.fulfilled, (state, action) => {
        state.person = action.payload.person;
        state.addresses = action.payload.addresses || [];
      })
      .addCase(updateUserProfile.fulfilled, (state, action) => {
        state.person = action.payload;
      })
      .addCase(fetchPersonAddresses.fulfilled, (state, action) => {
        state.addresses = action.payload;
      })
      .addCase(addPersonAddress.fulfilled, (state, action) => {
        state.addresses.push(action.payload);
      })
      .addCase(updatePersonAddress.fulfilled, (state, action) => {
        const index = state.addresses.findIndex((a) => a.id === action.payload.id);
        if (index !== -1) {
          state.addresses[index] = action.payload;
        }
      })
      .addCase(deletePersonAddress.fulfilled, (state, action) => {
        state.addresses = state.addresses.filter((a) => a.id !== action.payload);
      });
  },
});

export const { logout, clearError, setAddresses } = authSlice.actions;
export default authSlice.reducer;
