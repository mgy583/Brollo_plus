import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { ConfigProvider } from "antd";
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
          colorPrimary: "#1a1a1a",
          colorBgBase: "#f7f7f5",
          colorTextBase: "#1a1a1a",
          borderRadius: 2,
          fontFamily: "'Caveat', 'ZCOOL KuaiLe', cursive",
          fontSize: 15,
          colorBorder: "#1a1a1a",
          colorBgContainer: "#ffffff",
          colorBgElevated: "#ffffff",
          colorFillAlter: "#eeeeee",
          colorLink: "#2980b9",
          colorSuccess: "#27ae60",
          colorWarning: "#f39c12",
          colorError: "#c0392b",
          boxShadow: "3px 3px 0 #1a1a1a",
          boxShadowSecondary: "2px 2px 0 rgba(0,0,0,0.2)",
        },
        components: {
          Menu: {
            darkItemBg: "transparent",
            darkItemSelectedBg: "rgba(255,255,255,0.12)",
            darkItemColor: "#cccccc",
            darkItemSelectedColor: "#ffffff",
            itemBorderRadius: 2,
            fontSize: 15,
          },
          Table: {
            headerBg: "#eeeeee",
            headerColor: "#1a1a1a",
            rowHoverBg: "rgba(0,0,0,0.03)",
            borderColor: "#1a1a1a",
          },
          Card: { headerBg: "#fafafa" },
          Button: { borderRadius: 2, fontWeight: 600 },
          Input: { borderRadius: 2 },
          Select: { borderRadius: 2 },
          Modal: { borderRadiusLG: 2 },
        },
      }}
    >
      <RouterProvider router={router} />
    </ConfigProvider>
  </React.StrictMode>
);
