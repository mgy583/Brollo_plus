import React, { useEffect, useMemo, useState } from "react";
import { Button, Card, Col, DatePicker, Row, Select, Space, Spin, Statistic, Table, Typography } from "antd";
import dayjs from "dayjs";
import type { ColumnsType } from "antd/es/table";
import ReactECharts from "echarts-for-react";
import { accountsApi, type Account } from "../api/accounts";
import { reportsApi } from "../api/reports";
import { budgetsApi, type Budget } from "../api/budgets";
import { transactionsApi, type Transaction } from "../api/transactions";
import { categoriesApi, type Category } from "../api/categories";

type Overview = {
  total_income: number;
  total_expense: number;
  net: number;
  transaction_count: number;
};

export default function Dashboard() {
  const [loading, setLoading] = useState(true);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [categoriesStat, setCategoriesStat] = useState<{ categories: any[]; total: number } | null>(null);
  const [categoryList, setCategoryList] = useState<Category[]>([]);

  const categoryMap = useMemo(() => {
    const m: Record<string, string> = {};
    categoryList.forEach((c) => { m[c.id] = c.name; });
    return m;
  }, [categoryList]);

  const TX_TYPE_LABEL: Record<string, string> = { expense: "支出", income: "收入", transfer: "转账" };

  const [granularity, setGranularity] = useState<"day" | "week" | "month">("day");
  const [range, setRange] = useState<[dayjs.Dayjs, dayjs.Dayjs]>(() => {
    const now = dayjs();
    return [now.startOf("month"), now.endOf("month")];
  });

  async function load() {
    setLoading(true);
    try {
      const start_date = range[0].format("YYYY-MM-DD");
      const end_date = range[1].format("YYYY-MM-DD");
      const [accRes, ovRes, budRes, txRes, catRes, catListRes] = await Promise.all([
        accountsApi.list({ status: 'active' }),
        reportsApi.overview({ start_date, end_date }),
        budgetsApi.list(),
        transactionsApi.list({ page: 1, page_size: 500, start_date, end_date }),
        reportsApi.categories({ type: 'expense', start_date, end_date }),
        categoriesApi.list(),
      ]);
      setAccounts(accRes.data.accounts);
      setOverview(ovRes.data);
      setBudgets(budRes.data.budgets);
      setTransactions(txRes.data.transactions);
      setCategoriesStat(catRes.data);
      setCategoryList(catListRes.data.categories);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const assetSummary = useMemo(() => {
    let totalAssets = 0;
    let totalLiabilities = 0;
    accounts.forEach((a) => {
      if (a.type === "credit_card") {
        totalLiabilities += Math.abs(a.current_balance);
      } else if (a.current_balance >= 0) {
        totalAssets += a.current_balance;
      } else {
        totalLiabilities += Math.abs(a.current_balance);
      }
    });
    return {
      assets: totalAssets,
      liabilities: totalLiabilities,
      netWorth: totalAssets - totalLiabilities
    };
  }, [accounts]);

  const trendOption = useMemo(() => {
    const keyFor = (iso: string) => {
      const d = dayjs(iso.slice(0, 10));
      if (granularity === "day") return d.format("YYYY-MM-DD");
      if (granularity === "week") return d.startOf("week").format("YYYY-[W]WW");
      return d.startOf("month").format("YYYY-MM");
    };

    const map = new Map<string, { income: number; expense: number }>();
    transactions.forEach((tx) => {
      const key = keyFor(tx.transaction_date);
      const entry = map.get(key) ?? { income: 0, expense: 0 };
      if (tx.type === "income") entry.income += tx.amount;
      if (tx.type === "expense") entry.expense += tx.amount;
      map.set(key, entry);
    });
    const dates = Array.from(map.keys()).sort();
    const incomeSeries = dates.map((d) => map.get(d)!.income);
    const expenseSeries = dates.map((d) => map.get(d)!.expense);

    return {
      tooltip: { trigger: "axis" },
      legend: { data: ["收入", "支出"] },
      xAxis: { type: "category", data: dates },
      yAxis: { type: "value" },
      grid: { left: 40, right: 20, top: 40, bottom: 40 },
      series: [
        { name: "收入", type: "line", smooth: true, data: incomeSeries },
        { name: "支出", type: "line", smooth: true, data: expenseSeries }
      ]
    };
  }, [transactions]);

  const latestTxColumns: ColumnsType<Transaction> = useMemo(
    () => [
      { title: "时间", dataIndex: "transaction_date", width: 110, render: (v: string) => v?.slice(0, 10) ?? "-" },
      { title: "类型", dataIndex: "type", width: 65, render: (v: string) => <span style={{ color: v === "income" ? "#52c41a" : v === "expense" ? "#f5222d" : "#1677ff" }}>{({expense:"支出",income:"收入",transfer:"转账"} as any)[v] ?? v}</span> },
      { title: "金额", dataIndex: "amount", width: 95, render: (v: number, r: Transaction) => <span style={{ color: r.type === "income" ? "#52c41a" : r.type === "expense" ? "#f5222d" : undefined }}>{v.toFixed(2)}</span> },
      { title: "币种", dataIndex: "currency", width: 60 },
      { title: "分类", dataIndex: "category_id", width: 100, render: (v: string) => categoryMap[v] ?? "-" },
      { title: "描述", dataIndex: "description", render: (v: string | null) => v ?? "-" },
    ],
    [categoryMap]
  );

  const activeBudgets = useMemo(() => budgets.slice(0, 5), [budgets]);

  const categoryPieOption = useMemo(() => {
    if (!categoriesStat) return {};
    const data = categoriesStat.categories
      .map((c: any) => ({
        name: categoryMap[c.category_id] ?? c.category_id,
        value: +(c.amount ?? 0).toFixed(2)
      }))
      .sort((a: any, b: any) => b.value - a.value);
    return {
      tooltip: { trigger: "item" },
      legend: { orient: "vertical", left: "left" },
      series: [
        {
          name: "分类支出",
          type: "pie",
          radius: "70%",
          data
        }
      ]
    };
  }, [categoriesStat, categoryMap]);

  const accountBarOption = useMemo(() => {
    if (!accounts.length) return {};
    const sorted = [...accounts].sort((a, b) => b.current_balance - a.current_balance);
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: sorted.map((a) => a.name) },
      yAxis: { type: "value" },
      series: [
        {
          type: "bar",
          data: sorted.map((a) => a.current_balance)
        }
      ]
    };
  }, [accounts]);

  return (
    <div>
      <Typography.Title level={3}>仪表盘</Typography.Title>
      <Space style={{ marginBottom: 16 }}>
        <DatePicker.RangePicker
          value={range}
          onChange={(val) => {
            if (!val || val.length !== 2) return;
            setRange(val as [dayjs.Dayjs, dayjs.Dayjs]);
          }}
        />
        <Select
          style={{ width: 140 }}
          value={granularity}
          onChange={(v) => setGranularity(v)}
          options={[
            { value: "day", label: "按日" },
            { value: "week", label: "按周" },
            { value: "month", label: "按月" }
          ]}
        />
        <Button type="primary" onClick={load} loading={loading}>
          应用
        </Button>
      </Space>
      {loading ? (
        <div style={{ display: "grid", placeItems: "center", height: 260 }}>
          <Spin />
        </div>
      ) : (
        <Space direction="vertical" style={{ width: "100%" }} size="large">
          <Row gutter={16}>
            <Col xs={24} md={6}>
              <Card>
                <Statistic title="总资产" value={assetSummary.assets} precision={2} />
              </Card>
            </Col>
            <Col xs={24} md={6}>
              <Card>
                <Statistic title="总负债" value={assetSummary.liabilities} precision={2} />
              </Card>
            </Col>
            <Col xs={24} md={6}>
              <Card>
                <Statistic title="净资产" value={assetSummary.netWorth} precision={2} />
              </Card>
            </Col>
            <Col xs={24} md={6}>
              <Card>
                <Statistic
                  title="本月净收支"
                  value={overview ? overview.net : 0}
                  precision={2}
                />
              </Card>
            </Col>
          </Row>

          <Card title="收支趋势">
            {transactions.length === 0 ? (
              <Typography.Text type="secondary">暂无交易数据</Typography.Text>
            ) : (
              <ReactECharts option={trendOption} style={{ height: 320 }} />
            )}
          </Card>

          <Row gutter={16}>
            <Col xs={24} md={12}>
              <Card title="分类支出占比">
                {categoriesStat ? (
                  <ReactECharts option={categoryPieOption} style={{ height: 320 }} />
                ) : (
                  <Typography.Text type="secondary">暂无数据</Typography.Text>
                )}
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="账户资产分布">
                {accounts.length ? (
                  <ReactECharts option={accountBarOption} style={{ height: 320 }} />
                ) : (
                  <Typography.Text type="secondary">暂无账户数据</Typography.Text>
                )}
              </Card>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col xs={24} md={14}>
              <Card title="最近交易">
                <Table
                  rowKey="id"
                  size="small"
                  pagination={false}
                  dataSource={transactions.slice(0, 10)}
                  columns={latestTxColumns}
                />
              </Card>
            </Col>
            <Col xs={24} md={10}>
              <Card title="预算进度">
                {activeBudgets.length === 0 ? (
                  <Typography.Text type="secondary">暂无预算</Typography.Text>
                ) : (
                  <Space direction="vertical" style={{ width: "100%" }}>
                    {activeBudgets.map((b) => (
                      <div key={b.id}>
                        <div style={{ display: "flex", justifyContent: "space-between" }}>
                          <span>{b.name}</span>
                          <span>
                            {b.spent.toFixed(2)} / {b.amount.toFixed(2)}
                          </span>
                        </div>
                        <div
                          style={{
                            height: 8,
                            borderRadius: 4,
                            background: "#f0f0f0",
                            overflow: "hidden",
                            marginTop: 4
                          }}
                        >
                          <div
                            style={{
                              width: `${Math.min(b.progress, 100)}%`,
                              height: "100%",
                              background:
                                b.progress < 50
                                  ? "#52c41a"
                                  : b.progress < 80
                                  ? "#faad14"
                                  : "#f5222d"
                            }}
                          />
                        </div>
                      </div>
                    ))}
                  </Space>
                )}
              </Card>
            </Col>
          </Row>
        </Space>
      )}
    </div>
  );
}







