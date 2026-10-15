import { beforeEach, describe, expect, it, vi } from 'vitest';
import apiClient from '../../api/client';
import catalogService from '../catalogService';

vi.mock('../../api/client');

describe('catalogService does not invent product data', () => {
  beforeEach(() => { vi.mocked(apiClient.get).mockReset(); });

  it('maps a bare summary product without rating, stock, category or image', async () => {
    vi.mocked(apiClient.get).mockResolvedValue({ data: { items: [{ id: 1, slug: 'vinho', name: 'Vinho', priceCents: 5000, categorySlugs: [] }] } });
    const { items } = await catalogService.getProducts();
    expect(items[0]).toMatchObject({ category: '', categorySlug: '' });
    for (const field of ['rating', 'reviewCount', 'stock', 'thumbnail'] as const) expect(items[0][field], field).toBeUndefined();
    expect(JSON.stringify(items[0])).not.toMatch(/unsplash|Vinhos/);
  });

  it('keeps the rating and review count only when kremlin sends them', async () => {
    vi.mocked(apiClient.get).mockResolvedValue({ data: { items: [{ id: 1, slug: 'v', name: 'V', priceCents: 1, categorySlugs: ['tintos'], rating: 4.2, reviewCount: 3, primaryImageUrl: 'https://img/x.jpg' }] } });
    const { items } = await catalogService.getProducts();
    expect(items[0]).toMatchObject({ category: 'tintos', rating: 4.2, reviewCount: 3, thumbnail: 'https://img/x.jpg' });
  });

  it('builds the detail stock from the SKUs and has no stock photo', async () => {
    vi.mocked(apiClient.get).mockResolvedValue({ data: { id: 1, slug: 'v', name: 'V', images: [], skus: [{ id: 1, priceCents: 1000, stock: 4, active: true, attributes: [] }, { id: 2, priceCents: 1000, stock: 2, active: true, attributes: [] }] } });
    const product = await catalogService.getProductBySlug('v');
    expect(product).toMatchObject({ stock: 6, images: [] });
    expect(product?.thumbnail).toBeUndefined();
    expect(product?.rating).toBeUndefined();
  });

  it('treats a SKU with missing stock as zero', async () => {
    vi.mocked(apiClient.get).mockResolvedValue({ data: { id: 1, slug: 'v', name: 'V', images: [], skus: [{ id: 1, priceCents: 1000, active: true, attributes: [] }] } });
    const product = await catalogService.getProductBySlug('v');
    expect(product?.skus?.[0].stock).toBe(0);
    expect(product?.stock).toBe(0);
  });
});

describe('catalogService does not hide errors', () => {
  beforeEach(() => { vi.mocked(apiClient.get).mockReset(); });

  it('answers null only for a 404 on the product slug and never searches for another product', async () => {
    vi.mocked(apiClient.get).mockRejectedValue({ status: 404, message: 'not found' });
    expect(await catalogService.getProductBySlug('nope')).toBeNull();
    expect(apiClient.get).toHaveBeenCalledTimes(1);
  });

  it('throws any other failure of the product lookup', async () => {
    vi.mocked(apiClient.get).mockRejectedValue({ status: 500, message: 'boom' });
    await expect(catalogService.getProductBySlug('v')).rejects.toMatchObject({ status: 500 });
    vi.mocked(apiClient.get).mockRejectedValue({ message: 'Network Error' });
    await expect(catalogService.getProductBySlug('v')).rejects.toMatchObject({ message: 'Network Error' });
  });

  it('throws when the featured products cannot be loaded', async () => {
    vi.mocked(apiClient.get).mockRejectedValue({ status: 500, message: 'boom' });
    await expect(catalogService.getFeaturedProducts()).rejects.toMatchObject({ status: 500 });
  });
});
