import { render, screen } from '@testing-library/react';
import { MemoryRouter, useLocation, Outlet } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import AppRoutes from './AppRoutes';

vi.mock('../components/layout/Layout', () => ({ default: () => <Outlet /> }));
vi.mock('../pages/Catalog', () => ({ default: () => { const location = useLocation(); return <p>Catalog at {location.pathname}{location.search}</p>; } }));

describe('catalog routes', () => {
  it('opens the catalog at the root', () => {
    render(<MemoryRouter initialEntries={['/']}><AppRoutes /></MemoryRouter>);
    expect(screen.getByText('Catalog at /')).toBeInTheDocument();
  });
  it('preserves filters when redirecting old catalog links', () => {
    render(<MemoryRouter initialEntries={['/catalogo?cat=wine&q=red']}><AppRoutes /></MemoryRouter>);
    expect(screen.getByText('Catalog at /?cat=wine&q=red')).toBeInTheDocument();
  });
});
