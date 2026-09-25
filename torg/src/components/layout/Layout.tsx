import React, { useEffect } from 'react';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { selectCartItems } from '../../store';
import { enrichCartSeller } from '../../store/slices/cartSlice';
import { Outlet } from 'react-router-dom';
import Header from './Header';
import Footer from './Footer';
import CartDrawer from './CartDrawer';
import QuickViewModal from './QuickViewModal';
import ToastContainer from '../common/ToastContainer';

export const Layout: React.FC = () => {
  const dispatch = useAppDispatch();
  const items = useAppSelector(selectCartItems);
  const pending = useAppSelector(state => state.cart.sellerLookupPendingIds);
  const failed = useAppSelector(state => state.cart.sellerLookupFailedIds);
  useEffect(() => {
    const seen = new Set<string>();
    for (const item of items) {
      if (item.product.seller === undefined && !seen.has(item.product.id) &&
          !pending.includes(item.product.id) && !failed.includes(item.product.id)) {
        seen.add(item.product.id);
        dispatch(enrichCartSeller({ productId: item.product.id, slug: item.product.slug }));
      }
    }
  }, [items, pending, failed, dispatch]);
  return (
    <div className="min-h-screen flex flex-col bg-slate-50 text-slate-900 selection:bg-amber-500 selection:text-white">
      <Header />
      <main className="flex-1">
        <Outlet />
      </main>
      <Footer />
      <CartDrawer />
      <QuickViewModal />
      <ToastContainer />
    </div>
  );
};

export default Layout;
