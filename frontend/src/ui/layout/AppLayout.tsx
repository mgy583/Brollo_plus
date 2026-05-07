import React from "react";
import { Layout, Menu, Dropdown, Button, Avatar, Space, Typography } from "antd";
import {
  BarChartOutlined,
  DashboardOutlined,
  LogoutOutlined,
  PieChartOutlined,
  SettingOutlined,
  TeamOutlined,
  TransactionOutlined,
  UserOutlined,
  WalletOutlined,
} from "@ant-design/icons";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { useAuthStore } from "../../store/authStore";
import { authApi } from "../../api/auth";

const { Header, Sider, Content } = Layout;

const items = [
  { key: "/dashboard", icon: <DashboardOutlined />, label: "仪表盘" },
  { key: "/accounts", icon: <WalletOutlined />, label: "账户管理" },
  { key: "/transactions", icon: <TransactionOutlined />, label: "交易记录" },
  { key: "/budgets", icon: <PieChartOutlined />, label: "预算管理" },
  { key: "/reports", icon: <BarChartOutlined />, label: "统计报表" },
  { key: "/family", icon: <TeamOutlined />, label: "家庭管理" },
  { key: "/settings", icon: <SettingOutlined />, label: "系统设置" },
];

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const user = useAuthStore((s) => s.user);
  const refreshToken = useAuthStore((s) => s.refreshToken);
  const clearSession = useAuthStore((s) => s.clearSession);
  const activeItem = items.find((i) => i.key === location.pathname);

  const onLogout = async () => {
    try {
      if (refreshToken) await authApi.logout(refreshToken);
    } finally {
      clearSession();
      navigate("/auth/login", { replace: true });
    }
  };

  return (
    <Layout className="app-shell">
      <Sider width={232} breakpoint="lg" collapsedWidth={0}>
        <div style={{ padding: "24px 20px 18px" }}>
          <Typography.Title level={4} style={{ color: "#fff", margin: 0, letterSpacing: 0 }}>
            Brollo+
          </Typography.Title>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[activeItem?.key ?? "/dashboard"]}
          items={items}
          onClick={({ key }) => navigate(key)}
          theme="dark"
          style={{ border: "none" }}
        />
      </Sider>
      <Layout>
        <Header className="app-header">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", height: "100%" }}>
            <div>
              <Typography.Title level={4} style={{ margin: 0, lineHeight: 1.2 }}>
                {activeItem?.label ?? "仪表盘"}
              </Typography.Title>
            </div>
            <Dropdown
              menu={{
                items: [{ key: "logout", icon: <LogoutOutlined />, label: "退出登录", onClick: onLogout }],
              }}
            >
              <Button type="text" style={{ height: 40, padding: "0 10px" }}>
                <Space>
                  <Avatar size={28} icon={<UserOutlined />} style={{ background: "#111827" }} />
                  <span style={{ fontWeight: 500 }}>{user?.username ?? "用户"}</span>
                </Space>
              </Button>
            </Dropdown>
          </div>
        </Header>
        <Content className="app-content">
          <div className="app-content-inner">
            <Outlet />
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
