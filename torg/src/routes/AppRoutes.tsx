import React from 'react';
import { Routes, Route, Navigate, useLocation } from 'react-router-dom';
import Layout from '../components/layout/Layout';
import Catalog from '../pages/Catalog';
import ProductDetail from '../pages/ProductDetail';
import CartPage from '../pages/CartPage';
import CheckoutPage from '../pages/CheckoutPage';
import SignUpPage from '../pages/SignUpPage';
import NotFoundPage from '../pages/NotFoundPage';

const LegacyCatalogRedirect: React.FC = () => {
  const { search } = useLocation();
  return <Navigate to={`/${search}`} replace />;
};

export const AppRoutes: React.FC = () => (
  <Routes>
    <Route element={<Layout />}>
      <Route path="/" element={<Catalog />} />
      <Route path="/catalogo" element={<LegacyCatalogRedirect />} />
      <Route path="/produto/:slug" element={<ProductDetail />} />
      <Route path="/carrinho" element={<CartPage />} />
      <Route path="/checkout" element={<CheckoutPage />} />
      <Route path="/cadastro" element={<SignUpPage />} />
      <Route path="*" element={<NotFoundPage />} />
    </Route>
  </Routes>
);

export default AppRoutes;
