import React, { useEffect, useState, useCallback } from "react";
import {
  Button, Card, DatePicker, Form, Input, InputNumber, Modal,
  Select, Space, Spin, Table, Tag, Typography, message,
} from "antd";
import { PlusOutlined, DeleteOutlined } from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import dayjs from "dayjs";
import { transactionsApi, type Transaction } from "../api/transactions";
import { accountsApi, type Account } from "../api/accounts";
import { categoriesApi, type Category } from "../api/categories";

const TX_TYPES = [
  { value: "expense", label: "支出" },
  { value: "income", label: "收入" },
  { value: "transfer", label: "转账" },
];
const CURRENCIES = ["CNY", "USD", "EUR", "JPY", "GBP", "HKD"];

export default function Transactions() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [form] = Form.useForm();
  const [filterType, setFilterType] = useState<string | undefined>();
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs, dayjs.Dayjs] | null>(null);
  const [searchText, setSearchText] = useState("");
  const [formType, setFormType] = useState("expense");

  const load = useCallback(async (p = 1) => {
    setLoading(true);
    try {
      const params: Record<string, unknown> = { page: p, page_size: 20 };
      if (filterType) params.type = filterType;
      if (dateRange) {
        params.start_date = dateRange[0].format("YYYY-MM-DD");
        params.end_date = dateRange[1].format("YYYY-MM-DD");
      }
      if (searchText) params.search = searchText;
      const res = await transactionsApi.list(params);
      setTransactions(res.data.transactions);
      setTotal(res.data.pagination.total);
      setPage(p);
    } catch {
      message.error("加载失败");
    } finally {
      setLoading(false);
    }
  }, [filterType, dateRange, searchText]);

  useEffect(() => {
    void load();
    accountsApi.list().then(r => setAccounts(r.data.accounts)).catch(() => {});
    categoriesApi.list().then(r => setCategories(r.data.categories)).catch(() => {});
  }, [load]);

  const flatCats = categories.flatMap(c => [c, ...c.children]);
  const expenseCats = flatCats.filter(c => c.type === "expense");
  const incomeCats = flatCats.filter(c => c.type === "income");
  const formCats = formType === "income" ? incomeCats : expenseCats;

  const openModal = () => {
    form.resetFields();
    form.setFieldsValue({ type: "expense", currency: "CNY", transaction_date: dayjs() });
    setFormType("expense");
    setModalOpen(true);
  };

  const onSubmit = async (values: Record<string, unknown>) => {
    setSubmitting(true);
    try {
      const date = values.transaction_date as dayjs.Dayjs;
      await transactionsApi.create({
        type: values.type as "expense" | "income" | "transfer",
        amount: values.amount as number,
        currency: values.currency as string,
        account_id: values.account_id as string,
        category_id: values.category_id as string,
        description: values.description as string | undefined,
        transaction_date: date.format("YYYY-MM-DD"),
        tags: [],
      });
      message.success("记录成功");
      setModalOpen(false);
      void load();
    } catch { /**/ } finally {
      setSubmitting(false);
    }
  };

  const onDelete = async (id: string) => {
    try {
      await transactionsApi.delete(id);
      message.success("已删除");
      void load();
    } catch { /**/ }
  };

  const typeColor: Record<string, string> = { expense: "red", income: "green", transfer: "blue" };
  const typeLabel: Record<string, string> = { expense: "支出", income: "收入", transfer: "转账" };

  const columns: ColumnsType<Transaction> = [
    { title: "日期", dataIndex: "transaction_date", width: 120 },
    { title: "类型", dataIndex: "type", width: 80, render: v => <Tag color={typeColor[v]}>{typeLabel[v] ?? v}</Tag> },
    {
      title: "金额", dataIndex: "amount", width: 120,
      render: (v: number, r) => (
        <span style={{ color: r.type === "expense" ? "#f5222d" : "#52c41a", fontWeight: 600 }}>
          {r.type === "expense" ? "-" : "+"}{v.toFixed(2)} {r.currency}
        </span>
      ),
    },
    { title: "账户", dataIndex: "account_id", render: v => accounts.find(a => a.id === v)?.name ?? v },
    { title: "分类", dataIndex: "category_id", render: v => flatCats.find(c => c.id === v)?.name ?? v },
    { title: "描述", dataIndex: "description", ellipsis: true },
    {
      title: "操作", width: 80,
      render: (_, r) => <Button size="small" danger icon={<DeleteOutlined />} onClick={() => onDelete(r.id)}>删除</Button>,
    },
  ];

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
        <Typography.Title level={3} style={{ margin: 0 }}>交易记录</Typography.Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={openModal}>新建交易</Button>
      </div>

      <Card style={{ marginBottom: 16 }}>
        <Space wrap>
          <DatePicker.RangePicker
            value={dateRange}
            onChange={v => setDateRange(v as [dayjs.Dayjs, dayjs.Dayjs] | null)}
          />
          <Select
            allowClear
            placeholder="交易类型"
            style={{ width: 120 }}
            options={TX_TYPES}
            value={filterType}
            onChange={setFilterType}
          />
          <Input.Search
            placeholder="搜索描述"
            style={{ width: 200 }}
            value={searchText}
            onChange={e => setSearchText(e.target.value)}
            onSearch={() => load()}
          />
          <Button type="primary" onClick={() => load()}>查询</Button>
          <Button onClick={() => { setFilterType(undefined); setDateRange(null); setSearchText(""); }}>重置</Button>
        </Space>
      </Card>

      <Card>
        <Table
          rowKey="id"
          loading={loading}
          dataSource={transactions}
          columns={columns}
          pagination={{ current: page, total, pageSize: 20, onChange: p => load(p) }}
        />
      </Card>

      <Modal
        title="新建交易"
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        onOk={() => form.submit()}
        confirmLoading={submitting}
        destroyOnClose
      >
        <Form form={form} layout="vertical" onFinish={onSubmit}>
          <Form.Item name="type" label="交易类型" rules={[{ required: true }]}>
            <Select options={TX_TYPES} onChange={v => { setFormType(v); form.setFieldValue("category_id", undefined); }} />
          </Form.Item>
          <Form.Item name="amount" label="金额" rules={[{ required: true, type: "number", min: 0.01 }]}>
            <InputNumber style={{ width: "100%" }} precision={2} min={0.01} />
          </Form.Item>
          <Form.Item name="currency" label="币种" rules={[{ required: true }]}>
            <Select options={CURRENCIES.map(c => ({ value: c, label: c }))} />
          </Form.Item>
          <Form.Item name="account_id" label="账户" rules={[{ required: true }]}>
            <Select
              options={accounts.map(a => ({ value: a.id, label: `${a.name} (${a.currency} ${a.current_balance.toFixed(2)})` }))}
              placeholder="请选择账户"
            />
          </Form.Item>
          <Form.Item name="category_id" label="分类" rules={[{ required: true }]}>
            <Select
              options={formCats.map(c => ({ value: c.id, label: c.name }))}
              placeholder="请选择分类"
            />
          </Form.Item>
          <Form.Item name="transaction_date" label="交易日期" rules={[{ required: true }]}>
            <DatePicker style={{ width: "100%" }} />
          </Form.Item>
          <Form.Item name="description" label="描述">
            <Input placeholder="可选备注" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
