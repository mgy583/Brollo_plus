import React, { useState } from "react";
import { Button, Form, Input, Typography } from "antd";
import { Link, useNavigate } from "react-router-dom";
import { authApi } from "../../api/auth";
import { useAuthStore } from "../../store/authStore";

type FormValues = { username: string; email: string; full_name?: string; password: string };

export default function Register() {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const setSession = useAuthStore((s) => s.setSession);

  const onFinish = async (values: FormValues) => {
    setLoading(true);
    try {
      const r = await authApi.register(values);
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
        📒 新建账本
      </Typography.Title>
      <Form layout="vertical" onFinish={onFinish} requiredMark={false}>
        <Form.Item name="username" label="用户名" rules={[{ required: true, message: "请填写用户名" }]}>
          <Input autoFocus placeholder="起个好名字..." size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Form.Item name="email" label="邮箱" rules={[{ required: true, type: "email", message: "请填写有效邮箱" }]}>
          <Input placeholder="your@email.com" size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Form.Item name="full_name" label="姓名（可选）">
          <Input placeholder="你叫什么..." size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Form.Item name="password" label="密码" rules={[{ required: true, min: 8, message: "至少 8 位哦" }]}>
          <Input.Password placeholder="设置密码..." size="large" style={{ fontSize: 16 }} />
        </Form.Item>
        <Button type="primary" htmlType="submit" block loading={loading} size="large"
          style={{ marginTop: 4, fontSize: 16, height: 44, letterSpacing: 2 }}>
          创建账本 🎉
        </Button>
      </Form>
      <div style={{ marginTop: 16, textAlign: "center", fontSize: 14, color: "#888", borderTop: "1.5px dashed rgba(0,0,0,0.15)", paddingTop: 14 }}>
        已有账本？<Link to="/auth/login" style={{ color: "#2980b9", fontWeight: 700 }}>直接登录 →</Link>
      </div>
    </div>
  );
}
