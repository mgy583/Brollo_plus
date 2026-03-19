import React, { useEffect, useState } from "react";
import {
  Button, Card, Col, Form, Input, InputNumber, Modal,
  Row, Select, Space, Spin, Table, Tag, Typography, message,
} from "antd";
import { PlusOutlined, DeleteOutlined, EditOutlined, BankOutlined } from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import { accountsApi, type Account } from "../api/accounts";

const ACCOUNT_TYPES = [
  { value: "cash", label: "现金" },
  { value: "debit_card", label: "储蓄卡" },
  { value: "credit_card", label: "信用卡" },
  { value: "investment", label: "投资账户" },
  { value: "loan", label: "贷款" },
  { value: "ewallet", label: "电子钱包" },
];

const CURRENCIES = ["CNY", "USD", "EUR", "JPY", "GBP", "HKD"];

export default function Accounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [summary, setSummary] = useState<{ total_assets: number; total_liabilities: number; net_worth: number } | null>(null);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editAccount, setEditAccount] = useState<Account | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [form] = Form.useForm();

  const load = async () => {
    setLoading(true);
    try {
      const res = await accountsApi.list();
      setAccounts(res.data.accounts);
      setSummary(res.data.summary);
    } catch {
      message.error("加载账户失败");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { void load(); }, []);

  const openCreate = () => {
    setEditAccount(null);
    form.resetFields();
    form.setFieldsValue({ currency: "CNY", type: "debit_card", initial_balance: 0 });
    setModalOpen(true);
  };

  const openEdit = (a: Account) => {
    setEditAccount(a);
    form.setFieldsValue({ name: a.name, icon: a.icon, color: a.color, description: a.description });
    setModalOpen(true);
  };

  const onSubmit = async (values: Record<string, unknown>) => {
    setSubmitting(true);
    try {
      if (editAccount) {
        await accountsApi.update(editAccount.id, values as Parameters<typeof accountsApi.update>[1]);
        message.success("账户已更新");
      } else {
        await accountsApi.create(values as Parameters<typeof accountsApi.create>[0]);
        message.success("账户创建成功");
      }
      setModalOpen(false);
      void load();
    } catch {
      // error handled by interceptor
    } finally {
      setSubmitting(false);
    }
  };

  const onDelete = async (id: string) => {
    try {
      await accountsApi.remove(id);
      message.success("已删除");
      void load();
    } catch {
      // handled
    }
  };

  const columns: ColumnsType<Account> = [
    { title: "账户名称", dataIndex: "name", render: (v) => <span><BankOutlined style={{ marginRight: 6 }} />{v}</span> },
    { title: "类型", dataIndex: "type", render: (v) => ACCOUNT_TYPES.find(t => t.value === v)?.label ?? v },
    { title: "币种", dataIndex: "currency", width: 80 },
    { title: "当前余额", dataIndex: "current_balance", render: (v: number) => v.toFixed(2) },
    { title: "状态", dataIndex: "status", width: 80, render: (v) => <Tag color={v === "active" ? "green" : "default"}>{v === "active" ? "正常" : "已归档"}</Tag> },
    { title: "备注", dataIndex: "description" },
    {
      title: "操作", width: 120,
      render: (_, r) => (
        <Space>
          <Button size="small" icon={<EditOutlined />} onClick={() => openEdit(r)}>编辑</Button>
          <Button size="small" danger icon={<DeleteOutlined />} onClick={() => onDelete(r.id)}>删除</Button>
        </Space>
      )
    },
  ];

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
        <Typography.Title level={3} style={{ margin: 0 }}>账户管理</Typography.Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreate}>新建账户</Button>
      </div>

      {summary && (
        <Row gutter={16} style={{ marginBottom: 16 }}>
          <Col xs={24} md={8}>
            <Card size="small"><div style={{ textAlign: "center" }}><div style={{ color: "#888", fontSize: 12 }}>总资产</div><div style={{ fontSize: 22, fontWeight: 700, color: "#52c41a" }}>{summary.total_assets.toFixed(2)}</div></div></Card>
          </Col>
          <Col xs={24} md={8}>
            <Card size="small"><div style={{ textAlign: "center" }}><div style={{ color: "#888", fontSize: 12 }}>总负债</div><div style={{ fontSize: 22, fontWeight: 700, color: "#f5222d" }}>{summary.total_liabilities.toFixed(2)}</div></div></Card>
          </Col>
          <Col xs={24} md={8}>
            <Card size="small"><div style={{ textAlign: "center" }}><div style={{ color: "#888", fontSize: 12 }}>净资产</div><div style={{ fontSize: 22, fontWeight: 700, color: "#1890ff" }}>{summary.net_worth.toFixed(2)}</div></div></Card>
          </Col>
        </Row>
      )}

      <Card>
        {loading ? <div style={{ textAlign: "center", padding: 40 }}><Spin /></div> : (
          <Table rowKey="id" dataSource={accounts} columns={columns} pagination={{ pageSize: 20 }} />
        )}
      </Card>

      <Modal
        title={editAccount ? "编辑账户" : "新建账户"}
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        onOk={() => form.submit()}
        confirmLoading={submitting}
        destroyOnClose
      >
        <Form form={form} layout="vertical" onFinish={onSubmit}>
          <Form.Item name="name" label="账户名称" rules={[{ required: true, message: "请输入账户名称" }]}>
            <Input placeholder="如：招商银行储蓄卡" />
          </Form.Item>
          {!editAccount && (
            <>
              <Form.Item name="type" label="账户类型" rules={[{ required: true }]}>
                <Select options={ACCOUNT_TYPES} />
              </Form.Item>
              <Form.Item name="currency" label="币种" rules={[{ required: true }]}>
                <Select options={CURRENCIES.map(c => ({ value: c, label: c }))} />
              </Form.Item>
              <Form.Item name="initial_balance" label="初始余额" rules={[{ required: true }]}>
                <InputNumber style={{ width: "100%" }} precision={2} />
              </Form.Item>
            </>
          )}
          <Form.Item name="description" label="备注">
            <Input.TextArea rows={2} />
          </Form.Item>
          <Form.Item name="icon" label="图标">
            <Input placeholder="bank / card / wallet ..." />
          </Form.Item>
          <Form.Item name="color" label="颜色">
            <Input placeholder="#1890ff" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
