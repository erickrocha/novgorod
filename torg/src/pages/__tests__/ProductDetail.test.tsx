import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import ProductDetail from '../ProductDetail';
import catalogReducer from '../../store/slices/catalogSlice';
import cartReducer from '../../store/slices/cartSlice';
import uiReducer from '../../store/slices/uiSlice';
import authReducer from '../../store/slices/authSlice';
import catalogService from '../../services/catalogService';
import type { Product } from '../../types';

const mockProduct: Product = {
  id: '1',
  slug: 'miolo-lote-43-tinto',
  name: 'Miolo Lote 43 Tinto',
  description: 'Ícone da vitivinicultura brasileira elaborado apenas em safras excepcionais.',
  price: 259.0,
  originalPrice: 289.0,
  currency: 'BRL',
  category: 'Vinhos Tintos',
  categorySlug: 'vinhos-tintos',
  images: [
    'http://localhost:4566/novgorod-media-dev/tenants/1/products/1/images/img1.jpg',
    'http://localhost:4566/novgorod-media-dev/tenants/1/products/1/images/img2.jpg',
    'http://localhost:4566/novgorod-media-dev/tenants/1/products/1/images/img3.jpg',
  ],
  thumbnail: 'http://localhost:4566/novgorod-media-dev/tenants/1/products/1/images/img1.jpg',
  rating: 4.8,
  reviewCount: 24,
  stock: 48,
  seller: {
    id: 1,
    businessName: 'Sr Rocha winery',
    locality: 'Florianópolis',
    administrativeArea: 'SC',
  },
  attributes: {
    Marca: 'Miolo Wine Group',
    País: 'Brasil',
    Região: 'Vale dos Vinhedos',
    Uva: 'Cabernet Sauvignon, Merlot',
    Safra: '2020',
  },
  skus: [
    {
      id: 1,
      uuid: 'uuid-1',
      code: 'WINE-001-750',
      variantKey: '750ml',
      priceCents: 25900,
      compareAtPriceCents: 28900,
      active: true,
      stock: 48,
      attributes: [
        { attributeId: 1, name: 'Volume', value: '750ml' },
      ],
    },
  ],
  variants: [
    {
      id: '1',
      name: '750ml',
      sku: 'WINE-001-750',
      price: 259.0,
      stock: 48,
    },
  ],
};

import { combineReducers } from '@reduxjs/toolkit';

const rootReducer = combineReducers({
  catalog: catalogReducer,
  cart: cartReducer,
  ui: uiReducer,
  auth: authReducer,
});

const createTestStore = (preloadedState?: any) => {
  return configureStore({
    reducer: rootReducer,
    preloadedState,
  });
};

describe('ProductDetail Page', () => {
  beforeEach(() => {
    window.scrollTo = vi.fn();
    vi.spyOn(catalogService, 'getProductBySlug').mockResolvedValue(mockProduct);
    vi.spyOn(catalogService, 'getFeaturedProducts').mockResolvedValue([]);
  });

  it('renders product information, seller card, and specifications table', async () => {
    const store = createTestStore();

    render(
      <Provider store={store}>
        <MemoryRouter initialEntries={['/produto/miolo-lote-43-tinto']}>
          <Routes>
            <Route path="/produto/:slug" element={<ProductDetail />} />
          </Routes>
        </MemoryRouter>
      </Provider>
    );

    // Wait for product details to load using heading selector
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: 'Miolo Lote 43 Tinto' })).toBeInTheDocument();
    });

    // Brand and details
    expect(screen.getByText(/Miolo Wine Group/i)).toBeInTheDocument();

    // Seller / Tenant display
    expect(screen.getByText(/Vendido e entregue por/i)).toBeInTheDocument();
    expect(screen.getByText('Sr Rocha winery')).toBeInTheDocument();
    expect(screen.getByText(/Florianópolis - SC/i)).toBeInTheDocument();

    // Price & Stock
    expect(screen.getAllByText(/259,00/i).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText(/Em Estoque \(48 garrafas disponíveis\)/i)).toBeInTheDocument();

    // Specifications
    expect(screen.getByText(/Ficha Técnica & Especificações/i)).toBeInTheDocument();
    expect(screen.getByText('Vale dos Vinhedos')).toBeInTheDocument();
    expect(screen.getByText('Cabernet Sauvignon, Merlot')).toBeInTheDocument();

    // Thumbnails for multi-image gallery
    const thumbnails = screen.getAllByRole('button', { name: /ver imagem/i });
    expect(thumbnails).toHaveLength(3);
  });
});
