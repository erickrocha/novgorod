import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { orderService } from "@/services/orderService";
import { customerService } from "@/services/customerService";
import { getApiErrorMessage } from "@/services/api";
import type {
  Orders,
  OrderItem,
  OrderStatusHistory,
  OrderStatusHistoryInput,
  OrdersInput,
  Customer,
  PagedResult,
  PageQueryParams,
} from "@/services/types";

export interface OrderState {
  ordersList: Orders[];
  paged: PagedResult<Orders> | null;
  selectedOrder: Orders | null;
  orderItems: OrderItem[];
  statusHistories: OrderStatusHistory[];
  orderCustomer: Customer | null;
  loading: boolean;
  error: string | null;
}

const initialState: OrderState = {
  ordersList: [],
  paged: null,
  selectedOrder: null,
  orderItems: [],
  statusHistories: [],
  orderCustomer: null,
  loading: false,
  error: null,
};

export const fetchOrdersPaged = createAsyncThunk(
  "order/fetchOrdersPaged",
  async (
    params: (PageQueryParams & { customerId?: number; status?: string }) | undefined,
    { rejectWithValue },
  ) => {
    try {
      return await orderService.paged(params);
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load orders"));
    }
  },
);

export const fetchOrderDetails = createAsyncThunk(
  "order/fetchOrderDetails",
  async (order: Orders, { rejectWithValue }) => {
    try {
      const [itemsRes, historyRes, customer] = await Promise.all([
        orderService.itemsPaged({ orderId: order.id, pageSize: 50 }),
        orderService.statusHistoriesPaged({ orderId: order.id, pageSize: 50 }),
        customerService.getById(order.customerId).catch(() => null),
      ]);
      return {
        order,
        items: itemsRes.items,
        histories: historyRes.items,
        customer,
      };
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to load order details"));
    }
  },
);

export const updateOrderStatus = createAsyncThunk(
  "order/updateOrderStatus",
  async (
    {
      orderId,
      status,
      historyInput,
    }: {
      orderId: number;
      status: string;
      historyInput: OrderStatusHistoryInput;
    },
    { rejectWithValue },
  ) => {
    try {
      await orderService.addStatusHistory(historyInput);
      const updatedOrder = await orderService.update(orderId, { status } as OrdersInput);
      const updatedHistories = await orderService.statusHistoriesPaged({
        orderId,
        pageSize: 50,
      });
      return {
        updatedOrder,
        updatedHistories: updatedHistories.items,
      };
    } catch (error) {
      return rejectWithValue(getApiErrorMessage(error, "Failed to update order status"));
    }
  },
);

export const orderSlice = createSlice({
  name: "order",
  initialState,
  reducers: {
    clearSelectedOrder(state) {
      state.selectedOrder = null;
      state.orderItems = [];
      state.statusHistories = [];
      state.orderCustomer = null;
    },
    clearOrderError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // fetchOrdersPaged
      .addCase(fetchOrdersPaged.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchOrdersPaged.fulfilled, (state, action) => {
        state.loading = false;
        state.paged = action.payload;
        state.ordersList = action.payload.items;
      })
      .addCase(fetchOrdersPaged.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload as string;
      })
      // fetchOrderDetails
      .addCase(fetchOrderDetails.fulfilled, (state, action) => {
        state.selectedOrder = action.payload.order;
        state.orderItems = action.payload.items;
        state.statusHistories = action.payload.histories;
        state.orderCustomer = action.payload.customer;
      })
      // updateOrderStatus
      .addCase(updateOrderStatus.fulfilled, (state, action) => {
        state.selectedOrder = action.payload.updatedOrder;
        state.statusHistories = action.payload.updatedHistories;
        state.ordersList = state.ordersList.map((o) =>
          o.id === action.payload.updatedOrder.id ? action.payload.updatedOrder : o,
        );
      });
  },
});

export const { clearSelectedOrder, clearOrderError } = orderSlice.actions;
export default orderSlice.reducer;
