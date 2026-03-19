import React, { Suspense, lazy } from "react";
import { createBrowserRouter, Navigate } from "react-router-dom";
import { AppLayout } from "./ui/layout/AppLayout";
import { AuthLayout } from "./ui/layout/AuthLayout";
import { ProtectedRoute } from "./ui/routing/ProtectedRoute";
import { PageLoading } from "./ui/state/PageLoading";

const Dashboard = lazy(() => import("./views/Dashboard"));
const Accounts = lazy(() => import("./views/Accounts"));
const Transactions = lazy(() => import("./views/Transactions"));
const Budgets = lazy(() => import("./views/Budgets"));
const Reports = lazy(() => import("./views/Reports"));
const Settings = lazy(() => import("./views/Settings"));
const Login = lazy(() => import("./views/auth/Login"));
const Register = lazy(() => import("./views/auth/Register"));

const withSuspense = (el: React.ReactNode) => (
  <Suspense fallback={<PageLoading />}>{el}</Suspense>
);

export const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <ProtectedRoute>
        <AppLayout />
      </ProtectedRoute>
    ),
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      { path: "dashboard", element: withSuspense(<Dashboard />) },
      { path: "accounts", element: withSuspense(<Accounts />) },
      { path: "transactions", element: withSuspense(<Transactions />) },
      { path: "budgets", element: withSuspense(<Budgets />) },
      { path: "reports", element: withSuspense(<Reports />) },
      { path: "settings", element: withSuspense(<Settings />) }
    ]
  },
  {
    path: "/auth",
    element: <AuthLayout />,
    children: [
      { path: "login", element: withSuspense(<Login />) },
      { path: "register", element: withSuspense(<Register />) }
    ]
  },
  { path: "*", element: <Navigate to="/dashboard" replace /> }
]);

