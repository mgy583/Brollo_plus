import React, { useEffect, useState } from "react";
import { Alert, Button, Card, Col, Form, Input, Result, Row, Select, Space, Spin, Typography, message } from "antd";
import { AxiosError } from "axios";
import { useNavigate } from "react-router-dom";
import { authApi, type User } from "../api/auth";
import { useAuthStore } from "../store/authStore";

const CURRENCIES = ["CNY", "USD", "EUR", "JPY", "GBP", "HKD"];
const TIMEZONES = ["Asia/Shanghai", "Asia/Tokyo", "America/New_York", "Europe/London", "UTC"];
const LANGUAGES = [{ value: "zh-CN", label: "简体中文" }, { value: "en-US", label: "English" }];
const THEMES = [{ value: "light", label: "浅色" }, { value: "dark", label: "深色" }];

export default function Settings() {
  const navigate = useNavigate();
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [partialMode, setPartialMode] = useState(false);
  const [savingProfile, setSavingProfile] = useState(false);
  const [savingSettings, setSavingSettings] = useState(false);
  const [savingPassword, setSavingPassword] = useState(false);
  const [profileForm] = Form.useForm();
  const [settingsForm] = Form.useForm();
  const [pwdForm] = Form.useForm();
  const storeUser = useAuthStore(s => s.user);
  const accessToken = useAuthStore(s => s.accessToken);
  const refreshToken = useAuthStore(s => s.refreshToken);
  const setSession = useAuthStore(s => s.setSession);
  const clearSession = useAuthStore(s => s.clearSession);

  useEffect(() => {
    if (storeUser) {
      setUser(storeUser as User);
      profileForm.setFieldsValue({ full_name: storeUser.full_name, phone: storeUser.phone });
      settingsForm.setFieldsValue(storeUser.settings ?? {});
    }
    authApi.getMe()
      .then(r => {
        setUser(r.data);
        profileForm.setFieldsValue({ full_name: r.data.full_name, phone: r.data.phone });
        settingsForm.setFieldsValue(r.data.settings ?? {});
        setPartialMode(false);
        setLoadError(false);
      })
      .catch((err) => {
        const status = (err as AxiosError)?.response?.status;
        if (status === 401) {
          clearSession();
          navigate("/auth/login", { replace: true });
          return;
        }
        if (storeUser) {
          setPartialMode(true);
          return;
        }
        setLoadError(true);
      })
      .finally(() => setLoading(false));
  }, [profileForm, settingsForm, storeUser, clearSession, navigate]);

  const syncSessionUser = (nextUser: User) => {
    if (accessToken && refreshToken) {
      setSession({ accessToken, refreshToken, user: { ...(storeUser ?? nextUser), ...nextUser } });
    }
  };

  const onSaveProfile = async (values: { full_name?: string; phone?: string }) => {
    setSavingProfile(true);
    try {
      const r = await authApi.updateMe(values);
      setUser(r.data);
      syncSessionUser(r.data);
      message.success("资料已更新");
    } finally {
      setSavingProfile(false);
    }
  };

  const onSaveSettings = async (values: Record<string, string>) => {
    setSavingSettings(true);
    try {
      const r = await authApi.updateSettings(values);
      setUser(r.data);
      syncSessionUser(r.data);
      settingsForm.setFieldsValue(r.data.settings ?? values);
      message.success("偏好设置已保存");
    } finally {
      setSavingSettings(false);
    }
  };

  const onChangePassword = async (values: { old_password: string; new_password: string; confirm_password: string }) => {
    if (values.new_password !== values.confirm_password) {
      message.error("两次输入的新密码不一致");
      return;
    }
    setSavingPassword(true);
    try {
      await authApi.changePassword({ old_password: values.old_password, new_password: values.new_password });
      message.success("密码已修改，请重新登录");
      pwdForm.resetFields();
      clearSession();
      navigate("/auth/login", { replace: true });
    } finally {
      setSavingPassword(false);
    }
  };

  if (loading) return <div style={{ textAlign: "center", padding: 60 }}><Spin size="large" /></div>;

  if (loadError) return (
    <Result
      status="error"
      title="加载失败"
      subTitle="无法加载用户信息，请检查网络后重试"
      extra={<Button type="primary" onClick={() => window.location.reload()}>重新加载</Button>}
    />
  );

  return (
    <Space direction="vertical" size={16} style={{ width: "100%" }}>
      {partialMode && (
        <Alert
          type="warning"
          showIcon
          message="当前为离线降级模式"
          description="用户详情接口暂时不可用，页面已使用本地会话信息。你仍可尝试保存设置。"
        />
      )}

      <Row gutter={[20, 20]}>
        <Col xs={24} lg={12}>
          <Card title="个人资料" bordered={false} style={{ borderRadius: 8 }}>
            <Space direction="vertical" size={16} style={{ width: "100%" }}>
              <div>
                <Typography.Text type="secondary">用户名</Typography.Text>
                <div style={{ fontWeight: 600 }}>{user?.username}</div>
              </div>
              <div>
                <Typography.Text type="secondary">邮箱</Typography.Text>
                <div style={{ fontWeight: 600 }}>{user?.email}</div>
              </div>
              <Form form={profileForm} layout="vertical" onFinish={onSaveProfile}>
                <Form.Item name="full_name" label="姓名">
                  <Input placeholder="请输入姓名" />
                </Form.Item>
                <Form.Item name="phone" label="手机号">
                  <Input placeholder="请输入手机号" />
                </Form.Item>
                <Button type="primary" htmlType="submit" loading={savingProfile}>保存资料</Button>
              </Form>
            </Space>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card title="偏好设置" bordered={false} style={{ borderRadius: 8 }}>
            <Form form={settingsForm} layout="vertical" onFinish={onSaveSettings}>
              <Form.Item name="default_currency" label="默认货币">
                <Select options={CURRENCIES.map(c => ({ value: c, label: c }))} />
              </Form.Item>
              <Form.Item name="timezone" label="时区">
                <Select options={TIMEZONES.map(t => ({ value: t, label: t }))} />
              </Form.Item>
              <Form.Item name="language" label="语言">
                <Select options={LANGUAGES} />
              </Form.Item>
              <Form.Item name="theme" label="主题">
                <Select options={THEMES} />
              </Form.Item>
              <Button type="primary" htmlType="submit" loading={savingSettings}>保存设置</Button>
            </Form>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card title="修改密码" bordered={false} style={{ borderRadius: 8 }}>
            <Form form={pwdForm} layout="vertical" onFinish={onChangePassword}>
              <Form.Item name="old_password" label="当前密码" rules={[{ required: true, message: "请输入当前密码" }]}>
                <Input.Password autoComplete="current-password" />
              </Form.Item>
              <Form.Item name="new_password" label="新密码" rules={[{ required: true, min: 8, message: "新密码至少 8 位" }]}>
                <Input.Password autoComplete="new-password" />
              </Form.Item>
              <Form.Item name="confirm_password" label="确认新密码" rules={[{ required: true, message: "请再次输入新密码" }]}>
                <Input.Password autoComplete="new-password" />
              </Form.Item>
              <Button type="primary" htmlType="submit" loading={savingPassword}>修改密码</Button>
            </Form>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
