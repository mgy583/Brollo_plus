import React, { useEffect, useState } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { Spin } from "antd";
import { useAuthStore } from "../../store/authStore";

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const accessToken = useAuthStore((s) => s.accessToken);
  const user = useAuthStore((s) => s.user);
  const clearSession = useAuthStore((s) => s.clearSession);
  const location = useLocation();
  const [checking, setChecking] = useState(!!accessToken);
  const [valid, setValid] = useState(false);

  useEffect(() => {
    if (!accessToken) {
      setValid(false);
      setChecking(false);
      return;
    }

    if (!user) {
      clearSession();
      setValid(false);
      setChecking(false);
      return;
    }

    setValid(true);
    setChecking(false);
  }, [accessToken, user, clearSession]);

  if (!accessToken && !checking) {
    return <Navigate to="/auth/login" replace state={{ from: location.pathname }} />;
  }

  if (checking) {
    return (
      <div style={{ display: "grid", placeItems: "center", height: "100vh" }}>
        <Spin size="large" />
      </div>
    );
  }

  if (!valid) {
    return <Navigate to="/auth/login" replace state={{ from: location.pathname }} />;
  }

  return <>{children}</>;
}
