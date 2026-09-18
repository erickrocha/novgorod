import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";

import { getApiErrorMessage } from "@/services/api";
import type { PagedResult, PageQueryParams, User, UserInput } from "@/services/types";
import { userService } from "@/services/userService";

interface UserState {
  usersList: User[];
  total: number;
  page: number;
  pageSize: number;
  loading: boolean;
  error: string | null;
}

interface UserThunkConfig {
  rejectValue: string;
}

interface UpdateUserArgs {
  id: number;
  userData: UserInput;
}

export const fetchUsers = createAsyncThunk<
  PagedResult<User>,
  PageQueryParams | void,
  UserThunkConfig
>(
  "user/fetchUsers",
  async (params, { rejectWithValue }) => {
    try {
      return await userService.getUsersPaged(params || { page: 1, pageSize: 10 });
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao buscar usuários"),
      );
    }
  },
);

export const createUser = createAsyncThunk<User, UserInput, UserThunkConfig>(
  "user/createUser",
  async (userData, { rejectWithValue, dispatch }) => {
    try {
      const newUser = await userService.createUser(userData);
      dispatch(fetchUsers());
      return newUser;
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao criar usuário"),
      );
    }
  },
);

export const updateUser = createAsyncThunk<
  User,
  UpdateUserArgs,
  UserThunkConfig
>(
  "user/updateUser",
  async ({ id, userData }, { rejectWithValue, dispatch }) => {
    try {
      const updatedUser = await userService.updateUser(id, userData);
      dispatch(fetchUsers());
      return updatedUser;
    } catch (error: unknown) {
      return rejectWithValue(
        getApiErrorMessage(error, "Falha ao atualizar usuário"),
      );
    }
  },
);

const initialState: UserState = {
  usersList: [],
  total: 0,
  page: 1,
  pageSize: 10,
  loading: false,
  error: null,
};

const userSlice = createSlice({
  name: "user",
  initialState,
  reducers: {
    clearUserError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchUsers.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchUsers.fulfilled, (state, action) => {
        state.loading = false;
        state.usersList = action.payload.items;
        state.total = action.payload.total;
        state.page = action.payload.page;
        state.pageSize = action.payload.pageSize;
      })
      .addCase(fetchUsers.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to fetch users";
      })
      .addCase(createUser.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(createUser.fulfilled, (state) => {
        state.loading = false;
      })
      .addCase(createUser.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to create user";
      })
      .addCase(updateUser.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(updateUser.fulfilled, (state) => {
        state.loading = false;
      })
      .addCase(updateUser.rejected, (state, action) => {
        state.loading = false;
        state.error =
          action.payload ?? action.error.message ?? "Unable to update user";
      });
  },
});

export const { clearUserError } = userSlice.actions;
export default userSlice.reducer;
