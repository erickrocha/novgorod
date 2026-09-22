import { BrowserRouter } from "react-router-dom";
import { Sparkles } from "lucide-react";
import "./styles/main.scss";
import { fetchTenantById } from "@/store/tenantSlice";
import { validateOrRefreshToken } from "@/store/authSlice";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { useTranslation } from "react-i18next";
import { useEffect } from "react";
import { ProtectedRoutes } from "@/router/ProtectedRoutes.tsx";
import { PublicRoutes } from "@/router/PublicRoutes.tsx";

function AppContent() {
  const dispatch = useAppDispatch();
  const { t } = useTranslation();
  const { isAuthenticated, isInitializing, user } = useAppSelector(
    (state) => state.auth,
  );

  useEffect(() => {
    dispatch(validateOrRefreshToken());
  }, [dispatch]);

  useEffect(() => {
    if (isAuthenticated && user) {
      const tenantId = user.tenantId || user.tenant_id;
      if (tenantId) {
        dispatch(fetchTenantById(tenantId));
      }
    }
  }, [isAuthenticated, user, dispatch]);

  if (isInitializing) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-[var(--bg-primary)] text-[var(--text-body)]">
        <Sparkles
          size={36}
          className="text-[var(--accent-primary)] mb-4 animate-spin"
        />
        <p className="text-[0.9rem] tracking-[0.08em] text-[var(--text-body)] uppercase">
          {t("common.validatingSession")}
        </p>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <PublicRoutes />;
  }

  return <ProtectedRoutes />;
}

export function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  );
}

export default App;
