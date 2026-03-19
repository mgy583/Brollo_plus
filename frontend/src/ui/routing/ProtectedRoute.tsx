import React from "react";
import { Navigate, useLocation } from "react-router-dom";
import { useAuthStore } from "../../store/authStore";

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => !!s.accessToken);
  const location = useLocation();

  if (!isAuthenticated) {
    return <Navigate to="/auth/login" replace state={{ from: location.pathname }} />;
  }

  return <>{children}</>;
}

