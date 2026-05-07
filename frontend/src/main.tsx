import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { App, ConfigProvider } from "antd";
import zhCN from "antd/locale/zh_CN";
import "antd/dist/reset.css";
import "./styles.css";
import { router } from "./router";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ConfigProvider
      locale={zhCN}
      theme={{
        token: {
          colorPrimary: "#111827",
          colorBgBase: "#f6f7f9",
          colorTextBase: "#111827",
          borderRadius: 8,
          fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
          fontSize: 14,
          lineHeight: 1.5,
          colorBorder: "#e5e7eb",
          colorBgContainer: "#ffffff",
          colorBgElevated: "#ffffff",
          colorFillAlter: "#f3f4f6",
          colorLink: "#2563eb",
          colorSuccess: "#059669",
          colorWarning: "#d97706",
          colorError: "#dc2626",
          boxShadow: "0 1px 2px rgba(15, 23, 42, 0.06)",
          boxShadowSecondary: "0 12px 32px rgba(15, 23, 42, 0.08)",
        },
        components: {
          Menu: {
            darkItemBg: "transparent",
            darkItemSelectedBg: "rgba(255,255,255,0.12)",
            darkItemColor: "#cccccc",
            darkItemSelectedColor: "#ffffff",
            itemBorderRadius: 8,
            fontSize: 14,
          },
          Table: {
            headerBg: "#f9fafb",
            headerColor: "#6b7280",
            rowHoverBg: "rgba(0,0,0,0.03)",
            borderColor: "#e5e7eb",
          },
          Card: { headerBg: "#ffffff" },
          Button: { borderRadius: 8, fontWeight: 600 },
          Input: { borderRadius: 8 },
          Select: { borderRadius: 8 },
          Modal: { borderRadiusLG: 10 },
        },
      }}
    >
      <App>
        <RouterProvider router={router} />
      </App>
    </ConfigProvider>
  </React.StrictMode>
);
