import React, { useEffect, useState } from "react";
import { Button, Card, Col, Divider, Form, Input, Row, Select, Spin, Typography, message } from "antd";
import { authApi, type User } from "../api/auth";
import { useAuthStore } from "../store/authStore";

const CURRENCIES = ["CNY", "USD", "EUR", "JPY", "GBP", "HKD"];
const TIMEZONES = ["Asia/Shanghai", "Asia/Tokyo", "America/New_York", "Europe/London", "UTC"];
const LANGUAGES = [{ value: "zh-CN", label: "简体中文" }, { value: "en-US", label: "English" }];
const THEMES = [{ value: "light", label: "浅色" }, { value: "dark", label: "深色" }];

export default function Settings() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
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

  useEffect(() => {
    authApi.getMe()
      .then(r => {
        setUser(r.data);
        profileForm.setFieldsValue({ full_name: r.data.full_name });
        settingsForm.setFieldsValue(r.data.settings ?? {});
      })
      .catch(() => message.error("加载用户信息失败"))
      .finally(() => setLoading(false));
  }, [profileForm, settingsForm]);

  const onSaveProfile = async (values: { full_name: string }) => {
    setSavingProfile(true);
    try {
      const r = await authApi.updateMe({ full_name: values.full_name });
      setUser(r.data);
      if (storeUser && accessToken && refreshToken) {
        setSession({ accessToken, refreshToken, user: { ...storeUser, full_name: values.full_name } });
      }
      message.success("资料已更新");
    } catch { /**/ } finally { setSavingProfile(false); }
  };

  const onSaveSettings = async (values: Record<string, string>) => {
    setSavingSettings(true);
    try {
      await authApi.updateSettings(values);
      message.success("偏好设置已保存");
    } catch { /**/ } finally { setSavingSettings(false); }
  };

  const onChangePassword = async (values: { old_password: string; new_password: string; confirm_password: string }) => {
    if (values.new_password !== values.confirm_password) {
      message.error("两次输入的密码不一致");
      return;
    }
    setSavingPassword(true);
    try {
      await authApi.changePassword({ old_password: values.old_password, new_password: values.new_password });
      message.success("密码修改成功");
      pwdForm.resetFields();
    } catch { /**/ } finally { setSavingPassword(false); }
  };

  if (loading) return <div style={{ textAlign: "center", padding: 60 }}><Spin size="large" /></div>;

  return (
    <div>
      <Typography.Title level={3}>系统设置</Typography.Title>
      <Row gutter={24}>
        <Col xs={24} md={12}>
          <Card title="个人资料" style={{ marginBottom: 24 }}>
            <div style={{ marginBottom: 12 }}>
              <Typography.Text type="secondary">用户名：</Typography.Text>
              <Typography.Text strong>{user?.username}</Typography.Text>
            </div>
            <div style={{ marginBottom: 16 }}>
              <Typography.Text type="secondary">邮符1：</Typography.Text>
              <Typography.Text>{user?.email}</Typography.Text>
            </div>
            <Form form={profileForm} layout="vertical" onFinish={onSaveProfile}>
              <Form.Item name="full_name" label="姓名">
                <Input placeholder="请输入姓名" />
              </Form.Item>
              <Button type="primary" htmlType="submit" loading={savingProfile}>保存资料</Button>
            </Form>
          </Card>
          <Card title="修改密码">
            <Form form={pwdForm} layout="vertical" onFinish={onChangePassword}>
              <Form.Item name="old_password" label="当前密码" rules={[{ required: true }]}>
                <Input.Password />
              </Form.Item>
              <Form.Item name="new_password" label="新密码" rules={[{ required: true, min: 6 }]}>
                <Input.Password />
              </Form.Item>
              <Form.Item name="confirm_password" label="确认新密码" rules={[{ required: true }]}>
                <Input.Password />
              </Form.Item>
              <Button type="primary" htmlType="submit" loading={savingPassword}>修改密码</Button>
            </Form>
          </Card>
        </Col>
        <Col xs={24} md={12}>
          <Card title="偏好设置">
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
      </Row>
    </div>
  );
}
