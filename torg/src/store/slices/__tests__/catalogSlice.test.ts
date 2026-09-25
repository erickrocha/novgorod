import { describe, expect, it } from 'vitest';
import catalogReducer, { fetchProducts, setFilters } from '../catalogSlice';

const result = (ids: string[]) => ({ items: ids.map(id => ({ id })), total: ids.length, nextCursor: null });
describe('catalog requests', () => {
  it('ignores an older response after filters change', () => {
    let state = catalogReducer(undefined, fetchProducts.pending('old', undefined));
    state = catalogReducer(state, setFilters({ searchQuery: 'wine', category: 'all', sortBy: 'newest' }));
    state = catalogReducer(state, fetchProducts.pending('new', undefined));
    state = catalogReducer(state, fetchProducts.fulfilled(result(['new']) as never, 'new', undefined));
    state = catalogReducer(state, fetchProducts.fulfilled(result(['old']) as never, 'old', undefined));
    expect(state.products.map(p => p.id)).toEqual(['new']);
  });
});
