import { BrowserRouter } from "react-router";
import { Sparkles } from "lucide-react";
import "./styles/main.scss";
import { store } from "@/store";
import { fetchTenantById } from "@/store/tenantSlice";
import { validateOrRefreshToken } from "@/store/authSlice";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { useTranslation } from "react-i18next";
import { useEffect } from "react";
import { ProtectedRoutes } from "@/router/ProtectedRoutes.tsx";
import { PublicRoutes } from "@/router/PublicRoutes.tsx";
import { Provider } from "react-redux";

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
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          height: "100vh",
          backgroundColor: "var(--bg-primary)",
          color: "var(--text-body)",
        }}
      >
        <Sparkles
          size={36}
          style={{
            color: "var(--accent-primary)",
            marginBottom: "1rem",
            animation: "spin 2s linear infinite",
          }}
        />
        <p
          style={{
            fontSize: "0.9rem",
            letterSpacing: "0.08em",
            color: "var(--text-body)",
            textTransform: "uppercase",
          }}
        >
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

export default function App() {
  return (
    <Provider store={store}>
      <BrowserRouter>
        <AppContent />
      </BrowserRouter>
    </Provider>
  );
}
