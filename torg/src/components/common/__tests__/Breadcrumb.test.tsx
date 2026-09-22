import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { Breadcrumb } from '../Breadcrumb';


describe('Breadcrumb Component', () => {
  it('renders root home link by default', () => {
    render(
      <MemoryRouter>
        <Breadcrumb items={[{ label: 'Catálogo' }]} />
      </MemoryRouter>
    );

    const homeLink = screen.getByRole('link', { name: /início/i });
    expect(homeLink).toBeInTheDocument();
    expect(homeLink).toHaveAttribute('href', '/');
  });

  it('renders intermediate links and active leaf element', () => {
    render(
      <MemoryRouter>
        <Breadcrumb
          items={[
            { label: 'Carrinho', href: '/carrinho' },
            { label: 'Checkout' },
          ]}
        />
      </MemoryRouter>
    );

    const cartLink = screen.getByRole('link', { name: /carrinho/i });
    expect(cartLink).toBeInTheDocument();
    expect(cartLink).toHaveAttribute('href', '/carrinho');

    const checkoutText = screen.getByText('Checkout');
    expect(checkoutText).toBeInTheDocument();
    expect(checkoutText.tagName).toBe('SPAN');
  });

  it('supports hiding home link', () => {
    render(
      <MemoryRouter>
        <Breadcrumb items={[{ label: 'Vitrine' }]} showHome={false} />
      </MemoryRouter>
    );

    expect(screen.queryByRole('link', { name: /início/i })).not.toBeInTheDocument();
    expect(screen.getByText('Vitrine')).toBeInTheDocument();
  });
});
