import { render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';
import { store } from '../../../store';
import ProductCard from '../ProductCard';
import type { Product } from '../../../types';

const bare: Product = { id: '1', slug: 'vinho', name: 'Vinho Teste', description: '', price: 50, currency: 'BRL', category: '', categorySlug: '', images: [] };

const renderCard = (product: Product) =>
  render(<Provider store={store}><MemoryRouter><ProductCard product={product} /></MemoryRouter></Provider>);

describe('ProductCard shows only what the backend provides', () => {
  it('has no rating, review count or photo for a bare product', () => {
    const { container } = renderCard(bare);
    expect(screen.getByText('Vinho Teste')).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/4[.,]8|\(12\)|\(24\)/);
    expect(container.querySelector('svg.fill-amber-400')).toBeNull();
    expect(screen.getByRole('img', { name: /sem imagem/ })).toBeInTheDocument();
    expect(container.querySelector('img[src*="unsplash"]')).toBeNull();
  });

  it('shows the rating and the photo when they exist', () => {
    const { container } = renderCard({ ...bare, rating: 4.2, reviewCount: 3, thumbnail: 'https://img.test/a.jpg' });
    expect(container.textContent).toContain('4.2');
    expect(container.textContent).toContain('(3)');
    expect(container.querySelector('img')?.getAttribute('src')).toBe('https://img.test/a.jpg');
  });
});
