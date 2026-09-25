import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Provider } from 'react-redux';
import { configureStore, combineReducers } from '@reduxjs/toolkit';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import CheckoutPage from '../CheckoutPage';
import authReducer, { type CustomerProfile } from '../../store/slices/authSlice';
import cartReducer from '../../store/slices/cartSlice';
import uiReducer from '../../store/slices/uiSlice';
import catalogReducer from '../../store/slices/catalogSlice';
import authService from '../../services/authService';
import catalogService from '../../services/catalogService';
import checkoutService, { type CheckoutQuote, type Purchase } from '../../services/checkoutService';
import type { CartItem } from '../../types';

const rootReducer = combineReducers({
  auth: authReducer,
  cart: cartReducer,
  ui: uiReducer,
  catalog: catalogReducer,
});

const createTestStore = (preloadedState?: any) => {
  return configureStore({
    reducer: rootReducer,
    preloadedState,
  });
};

const mockCartItem: CartItem = {
  id: 'cart-1',
  product: {
    id: 'prod-1',
    slug: 'vinho-tinto-reserva',
    name: 'Vinho Tinto Reserva',
    description: 'Vinho Tinto Reserva Especial',
    thumbnail: '',
    price: 100.0,
    currency: 'BRL',
    category: 'Vinhos',
    categorySlug: 'vinhos',
    images: [],
    rating: 5,
    reviewCount: 10,
    stock: 20,
    seller: { id: 10, businessName: 'Vinícola Serra Alta' },
  },
  quantity: 2,
  unitPrice: 100.0,
  totalPrice: 200.0,
};

const mockCustomer: CustomerProfile = {
  id: '1',
  name: 'Maria Silva',
  cpf: '12345678901',
  email: 'maria@test.com',
  phone: '11999998888',
  addresses: [
    {
      id: '10',
      label: 'Casa',
      recipient: 'Maria Silva',
      addressLine1: 'Rua das Flores, 123',
      locality: 'Porto Alegre',
      administrativeArea: 'RS',
      postalCode: '90000-000',
      countryCode: 'BR',
      isDefault: true,
    },
  ],
};

const mockCustomerWithoutCpf: CustomerProfile = {
  id: '2',
  name: 'João Santos',
  cpf: null,
  email: 'joao@test.com',
  phone: '11988887777',
  addresses: [],
};

const mockCatalogProduct = {
  ...mockCartItem.product,
  skus: [
    { id: 101, code: 'SKU-101', variantKey: 'Garrafa 750ml', priceCents: 10000, active: true, stock: 20 },
  ],
};

const mockQuote: CheckoutQuote = {
  id: 55,
  expiresAt: new Date(Date.now() + 15 * 60 * 1000).toISOString(),
  shippingAddress: {
    recipient: 'Maria Silva',
    addressLine1: 'Rua das Flores, 123',
    locality: 'Porto Alegre',
    administrativeArea: 'RS',
    postalCode: '90000000',
    countryCode: 'BR',
  },
  items: [
    { skuId: 101, tenantId: 10, name: 'Vinho Tinto Reserva', quantity: 2, unitPriceCents: 10000, totalCents: 20000 },
  ],
  sellers: [
    { tenantId: 10, subtotalCents: 20000, discountCents: 2000, shippingCents: 1500, totalCents: 19500, couponCode: 'VINHO10' },
  ],
  subtotalCents: 20000,
  discountCents: 2000,
  shippingCents: 1500,
  totalCents: 19500,
};

describe('CheckoutPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    sessionStorage.clear();
    vi.spyOn(catalogService, 'getProductBySlug').mockResolvedValue(mockCatalogProduct as any);
  });

  it('renders CheckoutAuth when customer is not authenticated', async () => {
    const store = createTestStore({
      auth: { token: null, user: null, customer: null, isAuthenticated: false, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    expect(screen.getByText('Acesse sua conta para continuar')).toBeInTheDocument();
  });

  it('hydrates customer profile, displays read-only identity, and saved delivery addresses', async () => {
    vi.spyOn(authService, 'me').mockResolvedValue(mockCustomer);

    const store = createTestStore({
      auth: { token: 'mock-jwt-token', user: null, customer: null, isAuthenticated: true, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getByDisplayValue('Maria Silva')).toBeInTheDocument();
      expect(screen.getByDisplayValue('12345678901')).toBeInTheDocument();
      expect(screen.getByDisplayValue('maria@test.com')).toBeInTheDocument();
      expect(screen.getByDisplayValue('11999998888')).toBeInTheDocument();
    });

    // Check read-only state for Name and CPF
    expect(screen.getByDisplayValue('Maria Silva')).toHaveAttribute('readonly');
    expect(screen.getByDisplayValue('12345678901')).toHaveAttribute('readonly');

    // Saved address is offered and chosen by default
    expect(screen.getByText(/Deseja usar um endereço salvo para entrega\?/i)).toBeInTheDocument();
    expect(screen.getByText(/Casa/i)).toBeInTheDocument();
    expect(screen.getByText(/Padrão/i)).toBeInTheDocument();
  });

  it('displays one-time CPF completion when customer has no CPF', async () => {
    vi.spyOn(authService, 'me').mockResolvedValue(mockCustomerWithoutCpf);
    const completeCpfSpy = vi.spyOn(authService, 'completeCpf').mockResolvedValue({
      ...mockCustomerWithoutCpf,
      cpf: '52998224725',
    });

    const store = createTestStore({
      auth: { token: 'mock-jwt-token', user: null, customer: null, isAuthenticated: true, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getByText(/Informe seu CPF para continuar/i)).toBeInTheDocument();
    });

    const cpfInput = screen.getByLabelText(/CPF para completar cadastro/i);
    fireEvent.change(cpfInput, { target: { value: '52998224725' } });

    const saveCpfBtn = screen.getByRole('button', { name: /Salvar CPF/i });
    fireEvent.click(saveCpfBtn);

    await waitFor(() => {
      expect(completeCpfSpy).toHaveBeenCalledWith('52998224725');
    });
  });

  it('calculates server quote with seller breakdown, shipping, and coupons', async () => {
    vi.spyOn(authService, 'me').mockResolvedValue(mockCustomer);
    vi.spyOn(checkoutService, 'quote').mockResolvedValue(mockQuote);
    vi.spyOn(checkoutService, 'paymentConfig').mockResolvedValue({ publicKey: 'TEST_MP_PUBLIC_KEY' });

    const store = createTestStore({
      auth: { token: 'mock-jwt-token', user: null, customer: null, isAuthenticated: true, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    const calcBtn = await screen.findByRole('button', { name: /Calcular total com frete/i });
    await waitFor(() => {
      expect(calcBtn).not.toBeDisabled();
    });
    fireEvent.click(calcBtn);

    await waitFor(() => {
      expect(checkoutService.quote).toHaveBeenCalled();
    });

    // Check seller summary breakdown
    await waitFor(() => {
      expect(screen.getByText('Vinícola Serra Alta')).toBeInTheDocument();
      expect(screen.getByText(/Total a pagar:/i)).toBeInTheDocument();
      expect(screen.getAllByText(/195,00/).length).toBeGreaterThan(0);
    });

    // Payment section is revealed offering Cartão de crédito
    expect(screen.getByText(/Forma de pagamento/i)).toBeInTheDocument();
    expect(screen.getAllByText(/Cartão de crédito/i).length).toBeGreaterThan(0);
  });

  it('displays seller-specific error when shipping rate is missing for destination', async () => {
    vi.spyOn(authService, 'me').mockResolvedValue(mockCustomer);
    vi.spyOn(checkoutService, 'quote').mockRejectedValue(
      new Error('O vendedor Vinícola Serra Alta não entrega para o estado AM.')
    );

    const store = createTestStore({
      auth: { token: 'mock-jwt-token', user: null, customer: null, isAuthenticated: true, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    const calcBtn = await screen.findByRole('button', { name: /Calcular total com frete/i });
    await waitFor(() => expect(calcBtn).not.toBeDisabled());
    fireEvent.click(calcBtn);

    await waitFor(() => {
      expect(screen.getByText(/O vendedor Vinícola Serra Alta não entrega para o estado AM/i)).toBeInTheDocument();
    });
  });

  it('completes zero-total purchase directly without provider card form', async () => {
    const zeroQuote: CheckoutQuote = {
      ...mockQuote,
      subtotalCents: 20000,
      discountCents: 20000,
      shippingCents: 0,
      totalCents: 0,
      sellers: [
        { tenantId: 10, subtotalCents: 20000, discountCents: 20000, shippingCents: 0, totalCents: 0, couponCode: 'FREE100' },
      ],
    };
    const createdPurchase: Purchase = {
      id: 999,
      totalCents: 0,
      status: 'completed',
      orders: [],
      payments: [],
    };

    vi.spyOn(authService, 'me').mockResolvedValue(mockCustomer);
    vi.spyOn(checkoutService, 'quote').mockResolvedValue(zeroQuote);
    vi.spyOn(checkoutService, 'createPurchase').mockResolvedValue(createdPurchase);

    const store = createTestStore({
      auth: { token: 'mock-jwt-token', user: null, customer: null, isAuthenticated: true, loading: false },
      cart: { items: [mockCartItem], isOpen: false },
    });

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/checkout']}>
          <Routes>
            <Route path="/checkout" element={<CheckoutPage />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    const calcBtn = await screen.findByRole('button', { name: /Calcular total com frete/i });
    await waitFor(() => expect(calcBtn).not.toBeDisabled());
    fireEvent.click(calcBtn);

    const zeroBtn = await screen.findByRole('button', { name: /Concluir pedido gratuito/i });
    fireEvent.click(zeroBtn);

    await waitFor(() => {
      expect(checkoutService.createPurchase).toHaveBeenCalledWith(55, 'maria@test.com', '11999998888');
      expect(screen.getByText(/Pagamento confirmado/i)).toBeInTheDocument();
      expect(screen.getByText(/#999/)).toBeInTheDocument();
    });
  });
});
