import React, { useEffect, useState, useCallback } from "react";
import {
  Button, Card, Col, DatePicker, Form, InputNumber, Modal,
  Progress, Row, Select, Table, Tag, Typography, message, Input,
} from "antd";
import { PlusOutlined, DeleteOutlined } from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import dayjs from "dayjs";
import { budgetsApi, type Budget } from "../api/budgets";
import { categoriesApi, type Category } from "../api/categories";

const BUDGET_TYPES = [
  { value: "daily", label: "每日" },
  { value: "weekly", label: "每周" },
  { value: "monthly", label: "每月" },
  { value: "yearly", label: "每年" },
  { value: "custom", label: "自定义" },
];
const CURRENCIES = ["CNY", "USD", "EUR", "JPY", "GBP"];

export default function Budgets() {
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [form] = Form.useForm();

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const res = await budgetsApi.list();
      setBudgets(res.data.budgets);
    } catch { message.error("加载预算失败"); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => {
    void load();
    categoriesApi.list({ type: "expense" })
      .then(r => setCategories(r.data.categories))
      .catch(() => {});
  }, [load]);

  const flatCats = categories.flatMap(c => [c, ...c.children]);

  const openModal = () => {
    form.resetFields();
    const now = dayjs();
    form.setFieldsValue({
      type: "monthly",
      currency: "CNY",
      start_date: now.startOf("month"),
      end_date: now.endOf("month"),
    });
    setModalOpen(true);
  };

  const onSubmit = async (values: Record<string, unknown>) => {
    setSubmitting(true);
    try {
      await budgetsApi.create({
        name: values.name as string,
        type: values.type as string,
        start_date: (values.start_date as dayjs.Dayjs).format("YYYY-MM-DD"),
        end_date: (values.end_date as dayjs.Dayjs).format("YYYY-MM-DD"),
        amount: values.amount as number,
        currency: values.currency as string,
        category_ids: (values.category_ids as string[] | undefined) ?? [],
      });
      message.success("预算创建成功");
      setModalOpen(false);
      void load();
    } catch { /**/ } finally { setSubmitting(false); }
  };

  const onDelete = async (id: string) => {
    try { await budgetsApi.remove(id); message.success("已删除"); void load(); }
    catch { /**/ }
  };

  const statusColor: Record<string, string> = { active: "green", completed: "blue", paused: "orange" };
  const statusLabel: Record<string, string> = { active: "进行中", completed: "已完成", paused: "已暂停" };

  const columns: ColumnsType<Budget> = [
    { title: "预算名称", dataIndex: "name" },
    { title: "周期", dataIndex: "type", width: 80, render: v => BUDGET_TYPES.find(t => t.value === v)?.label ?? v },
    { title: "预算额", dataIndex: "amount", width: 100, render: (v: number) => v.toFixed(2) },
    { title: "已支出", dataIndex: "spent", width: 100, render: (v: number) => <span style={{ color: "#f5222d" }}>{v.toFixed(2)}</span> },
    { title: "剩余", dataIndex: "remaining", width: 100, render: (v: number) => <span style={{ color: "#52c41a" }}>{v.toFixed(2)}</span> },
    {
      title: "执行进度", dataIndex: "progress", width: 180,
      render: (v: number) => (
        <Progress
          percent={Math.min(Math.round(v), 100)}
          size="small"
          strokeColor={v < 50 ? "#52c41a" : v < 80 ? "#faad14" : "#f5222d"}
        />
      ),
    },
    { title: "开始", dataIndex: "start_date", width: 105 },
    { title: "结束", dataIndex: "end_date", width: 105 },
    { title: "状态", dataIndex: "status", width: 90, render: v => <Tag color={statusColor[v]}>{statusLabel[v] ?? v}</Tag> },
    {
      title: "操作", width: 80,
      render: (_, r) => (
        <Button size="small" danger icon={<DeleteOutlined />} onClick={() => onDelete(r.id)}>删除</Button>
      ),
    },
  ];

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
        <Typography.Title level={3} style={{ margin: 0 }}>预算管理</Typography.Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={openModal}>新建预算</Button>
      </div>
      <Card>
        <Table rowKey="id" loading={loading} dataSource={budgets} columns={columns} pagination={{ pageSize: 20 }} />
      </Card>
      <Modal
        title="新建预算"
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        onOk={() => form.submit()}
        confirmLoading={submitting}
        destroyOnClose
      >
        <Form form={form} layout="vertical" onFinish={onSubmit}>
          <Form.Item name="name" label="预算名称" rules={[{ required: true, message: "请填写" }]}>
            <Input placeholder="如：12月餐饮预算" />
          </Form.Item>
          <Form.Item name="type" label="预算周期" rules={[{ required: true }]}>
            <Select options={BUDGET_TYPES} />
          </Form.Item>
          <Row gutter={8}>
            <Col span={12}>
              <Form.Item name="start_date" label="开始日期" rules={[{ required: true }]}>
                <DatePicker style={{ width: "100%" }} />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item name="end_date" label="结束日期" rules={[{ required: true }]}>
                <DatePicker style={{ width: "100%" }} />
              </Form.Item>
            </Col>
          </Row>
          <Row gutter={8}>
            <Col span={14}>
              <Form.Item name="amount" label="预算金额" rules={[{ required: true, type: "number", min: 0.01 }]}>
                <InputNumber style={{ width: "100%" }} precision={2} min={0.01} />
              </Form.Item>
            </Col>
            <Col span={10}>
              <Form.Item name="currency" label="币种" rules={[{ required: true }]}>
                <Select options={CURRENCIES.map(c => ({ value: c, label: c }))} />
              </Form.Item>
            </Col>
          </Row>
          <Form.Item name="category_ids" label="关联分类（留空=全部）">
            <Select
              mode="multiple"
              allowClear
              placeholder="选择一个或多个分类"
              options={flatCats.map(c => ({ value: c.id, label: c.name }))}
            />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
