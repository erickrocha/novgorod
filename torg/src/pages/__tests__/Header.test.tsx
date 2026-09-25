import { render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';
import { store } from '../../store';
import Header from '../../components/layout/Header';
import '../../i18n';

describe('catalog header', () => {
  it('shows the brand, desktop and mobile search, and the four account actions', () => {
    render(<Provider store={store}><MemoryRouter><Header /></MemoryRouter></Provider>);
    expect(screen.getByRole('link', { name: 'Torg: catálogo' })).toHaveAttribute('href', '/');
    expect(screen.getAllByRole('textbox', { name: 'Buscar produtos' })).toHaveLength(2);
    expect(screen.getByRole('link', { name: 'Favoritos' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Selecionar idioma' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Minha Conta' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Abrir carrinho' })).toBeInTheDocument();
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
  });
});
