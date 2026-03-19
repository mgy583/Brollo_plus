import React, { useState } from "react";
import { Button, Form, Input, Typography } from "antd";
import { Link, useNavigate } from "react-router-dom";
import { authApi } from "../../api/auth";
import { useAuthStore } from "../../store/authStore";

type FormValues = { username: string; password: string };

export default function Login() {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const setSession = useAuthStore((s) => s.setSession);

  const onFinish = async (values: FormValues) => {
    setLoading(true);
    try {
      const r = await authApi.login({
        username: values.username,
        password: values.password,
        device_info: { device_type: "web", device_name: navigator.userAgent },
      });
      setSession({
        accessToken: r.data.tokens.access_token,
        refreshToken: r.data.tokens.refresh_token,
        user: r.data.user,
      });
      navigate("/dashboard", { replace: true });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <Typography.Title level={3} style={{ marginTop: 0, marginBottom: 20, color: "#1a1a1a", textAlign: "center" }}>
        📖 欢迎回来
      </Typography.Title>
      <Form layout="vertical" onFinish={onFinish} requiredMark={false}>
        <Form.Item name="username" label="用户名" rules={[{ required: true, message: "请填写用户名" }]}>
          <Input autoFocus placeholder="写下你的名字..." size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Form.Item name="password" label="密码" rules={[{ required: true, message: "请填写密码" }]}>
          <Input.Password placeholder="密码..." size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Button type="primary" htmlType="submit" block loading={loading} size="large"
          style={{ marginTop: 4, fontSize: 16, height: 44, letterSpacing: 2 }}>
          登录 →
        </Button>
      </Form>
      <div style={{ marginTop: 16, textAlign: "center", fontSize: 14, color: "#888", borderTop: "1.5px dashed rgba(0,0,0,0.15)", paddingTop: 14 }}>
        还没有账本？<Link to="/auth/register" style={{ color: "#2980b9", fontWeight: 700 }}>创建一本 ✏️</Link>
      </div>
    </div>
  );
}
