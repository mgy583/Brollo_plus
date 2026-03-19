import React from "react";
import { Layout, Menu, Dropdown, Button } from "antd";
import {
  DashboardOutlined,
  WalletOutlined,
  TransactionOutlined,
  PieChartOutlined,
  BarChartOutlined,
  SettingOutlined,
  LogoutOutlined,
  TeamOutlined,
} from "@ant-design/icons";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { useAuthStore } from "../../store/authStore";
import { authApi } from "../../api/auth";

const { Header, Sider, Content } = Layout;

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const user = useAuthStore((s) => s.user);
  const refreshToken = useAuthStore((s) => s.refreshToken);
  const clearSession = useAuthStore((s) => s.clearSession);

  const items = [
    { key: "/dashboard", icon: <DashboardOutlined />, label: "仪表盘" },
    { key: "/accounts", icon: <WalletOutlined />, label: "账户管理" },
    { key: "/transactions", icon: <TransactionOutlined />, label: "交易记录" },
    { key: "/budgets", icon: <PieChartOutlined />, label: "预算管理" },
    { key: "/reports", icon: <BarChartOutlined />, label: "统计报表" },
    { key: "/family", icon: <TeamOutlined />, label: "家庭管理" },
    { key: "/settings", icon: <SettingOutlined />, label: "系统设置" },
  ];

  const onLogout = async () => {
    try {
      if (refreshToken) await authApi.logout(refreshToken);
    } finally {
      clearSession();
      navigate("/auth/login", { replace: true });
    }
  };

  return (
    <Layout style={{ minHeight: "100%" }}>
      <Sider
        width={220}
        style={{
          background: "#1a1a1a",
          borderRight: "3px solid #000",
          boxShadow: "4px 0 0 #000",
        }}
      >
        <div
          style={{
            padding: "20px 16px 14px",
            borderBottom: "1.5px dashed rgba(255,255,255,0.15)",
            marginBottom: 8,
          }}
        >
          <div
            style={{
              fontSize: 20,
              fontWeight: 700,
              color: "#ffffff",
              letterSpacing: 1,
              lineHeight: 1.2,
            }}
          >
            Brollo+
          </div>
          <div style={{ fontSize: 11, color: "rgba(255,255,255,0.4)", marginTop: 3 }}>
            手账记账本
          </div>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[location.pathname]}
          items={items}
          onClick={({ key }) => navigate(key)}
          theme="dark"
          style={{ background: "transparent", border: "none" }}
        />
      </Sider>
      <Layout>
        <Header
          style={{
            background: "#eeeeee",
            padding: "0 20px",
            borderBottom: "2.5px solid #1a1a1a",
            boxShadow: "0 3px 0 #1a1a1a",
            height: 54,
            lineHeight: "54px",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", height: "100%" }}>
            <div style={{ fontWeight: 700, fontSize: 17, color: "#1a1a1a" }}>
              {items.find((i) => i.key === location.pathname)?.label ?? "记账本"}
            </div>
            <Dropdown
              menu={{
                items: [{ key: "logout", icon: <LogoutOutlined />, label: "退出登录", onClick: onLogout }],
              }}
            >
              <Button
                type="text"
                style={{
                  fontWeight: 700,
                  fontSize: 14,
                  color: "#1a1a1a",
                  border: "2px solid #1a1a1a",
                  borderRadius: 2,
                  boxShadow: "2px 2px 0 #1a1a1a",
                  padding: "0 12px",
                  height: 32,
                  background: "#fff",
                }}
              >
                {user?.username ?? "用户"}
              </Button>
            </Dropdown>
          </div>
        </Header>
        <Content style={{ margin: 16 }}>
          <div
            style={{
              background: "#ffffff",
              padding: 20,
              borderRadius: 2,
              minHeight: 360,
              border: "2px solid #1a1a1a",
              boxShadow: "5px 5px 0 #1a1a1a",
              position: "relative",
            }}
          >
            <div
              style={{
                position: "absolute",
                top: 0,
                right: 0,
                width: 0,
                height: 0,
                borderStyle: "solid",
                borderWidth: "0 24px 24px 0",
                borderColor: "transparent #1a1a1a transparent transparent",
              }}
            />
            <Outlet />
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
