import { fireEvent, render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { describe, expect, it, vi } from 'vitest';
import { store } from '../../../store';
import ProductFilters from '../ProductFilters';

const filters = { searchQuery: '', category: 'all', sortBy: 'newest' as const, minPrice: undefined, maxPrice: undefined };
describe('ProductFilters', () => {
  it('commits valid prices only when Apply is pressed', () => {
    const onApply = vi.fn();
    render(<Provider store={store}><ProductFilters filters={filters} onApply={onApply} onClear={vi.fn()} /></Provider>);
    fireEvent.change(screen.getByLabelText('Mínimo'), { target: { value: '10' } });
    fireEvent.change(screen.getByLabelText('Máximo'), { target: { value: '50' } });
    expect(onApply).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar filtros' }));
    expect(onApply).toHaveBeenCalledWith({ category: 'all', minPrice: 10, maxPrice: 50 });
  });
  it('rejects reversed bounds', () => {
    const onApply = vi.fn();
    render(<Provider store={store}><ProductFilters filters={filters} onApply={onApply} onClear={vi.fn()} /></Provider>);
    fireEvent.change(screen.getByLabelText('Mínimo'), { target: { value: '50' } });
    fireEvent.change(screen.getByLabelText('Máximo'), { target: { value: '10' } });
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar filtros' }));
    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(onApply).not.toHaveBeenCalled();
  });
});
