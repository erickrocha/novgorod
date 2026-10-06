import type React from "react";
import { Navigate } from "react-router-dom";
import type { Role } from "@/services/types";
import { useAppSelector } from "@/store/hooks";

interface RoleRouteGuardProps {
  allowedRoles: Role[];
  children: React.ReactNode;
}

export function RoleRouteGuard({ allowedRoles, children }: RoleRouteGuardProps) {
  const role = useAppSelector((state) => state.auth.user?.role);

  return role && allowedRoles.includes(role) ? (
    <>{children}</>
  ) : (
    <Navigate to="/" replace />
  );
}