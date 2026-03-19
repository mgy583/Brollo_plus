import React, { useEffect, useState } from "react";
import { Button, Card, Col, DatePicker, Row, Select, Space, Spin, Statistic, Typography } from "antd";
import dayjs from "dayjs";
import ReactECharts from "echarts-for-react";
import { reportsApi } from "../api/reports";
import { categoriesApi, type Category } from "../api/categories";

type Overview = {
  total_income: number;
  total_expense: number;
  net: number;
  transaction_count: number;
};
type CatStat = { category_id: string; amount: number; count: number; percentage: number };
type TrendPoint = { date: string; income: number; expense: number; net: number };

export default function Reports() {
  const [loading, setLoading] = useState(false);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [catStats, setCatStats] = useState<CatStat[]>([]);
  const [trend, setTrend] = useState<TrendPoint[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [granularity, setGranularity] = useState("month");
  const [catType, setCatType] = useState("expense");
  const [range, setRange] = useState<[dayjs.Dayjs, dayjs.Dayjs]>([
    dayjs().startOf("month"),
    dayjs().endOf("month"),
  ]);

  useEffect(() => {
    categoriesApi.list().then(r => setCategories(r.data.categories)).catch(() => {});
    void doLoad(range, granularity, catType);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const doLoad = async (
    r: [dayjs.Dayjs, dayjs.Dayjs],
    iv: string,
    ct: string,
  ) => {
    setLoading(true);
    const start_date = r[0].format("YYYY-MM-DD");
    const end_date = r[1].format("YYYY-MM-DD");
    try {
      const [ov, cs, tr] = await Promise.all([
        reportsApi.overview({ start_date, end_date }),
        reportsApi.categories({ type: ct, start_date, end_date }),
        reportsApi.trend({ start_date, end_date, interval: iv }),
      ]);
      setOverview(ov.data);
      setCatStats(cs.data.categories);
      setTrend(tr.data.series);
    } catch { /**/ } finally { setLoading(false); }
  };

  const handleQuery = () => doLoad(range, granularity, catType);

  const allCats = categories.flatMap(c => [c, ...c.children]);
  const catName = (id: string) => allCats.find(c => c.id === id)?.name ?? id;

  const pieOption = {
    tooltip: { trigger: "item", formatter: "{b}: {c} ({d}%)" },
    legend: { orient: "vertical", left: "left", type: "scroll", top: "middle" },
    series: [{
      name: catType === "expense" ? "支出" : "收入",
      type: "pie",
      radius: ["40%", "70%"],
      center: ["60%", "50%"],
      data: catStats.map(c => ({ name: catName(c.category_id), value: Number(c.amount.toFixed(2)) })),
    }],
  };

  const trendOption = {
    tooltip: { trigger: "axis" },
    legend: { data: ["收入", "支出", "净额"] },
    xAxis: { type: "category", data: trend.map(t => t.date) },
    yAxis: { type: "value" },
    grid: { left: 50, right: 20, top: 40, bottom: 40 },
    series: [
      { name: "收入", type: "bar", data: trend.map(t => t.income), itemStyle: { color: "#52c41a" } },
      { name: "支出", type: "bar", data: trend.map(t => t.expense), itemStyle: { color: "#f5222d" } },
      { name: "净额", type: "line", smooth: true, data: trend.map(t => t.net), itemStyle: { color: "#1890ff" } },
    ],
  };

  return (
    <div>
      <Typography.Title level={3}>统计报表</Typography.Title>

      <Card style={{ marginBottom: 16 }}>
        <Space wrap>
          <DatePicker.RangePicker
            value={range}
            onChange={v => v && setRange(v as [dayjs.Dayjs, dayjs.Dayjs])}
          />
          <Select
            style={{ width: 120 }}
            value={granularity}
            onChange={setGranularity}
            options={[
              { value: "day", label: "按日" },
              { value: "week", label: "按周" },
              { value: "month", label: "按月" },
            ]}
          />
          <Select
            style={{ width: 130 }}
            value={catType}
            onChange={setCatType}
            options={[
              { value: "expense", label: "支出分类" },
              { value: "income", label: "收入分类" },
            ]}
          />
          <Button type="primary" loading={loading} onClick={handleQuery}>查询</Button>
        </Space>
      </Card>

      {loading ? (
        <div style={{ textAlign: "center", padding: 60 }}><Spin size="large" /></div>
      ) : (
        <Space direction="vertical" style={{ width: "100%" }} size="large">
          {overview && (
            <Row gutter={16}>
              <Col xs={24} sm={12} md={6}>
                <Card><Statistic title="总收入" value={overview.total_income} precision={2} valueStyle={{ color: "#52c41a" }} /></Card>
              </Col>
              <Col xs={24} sm={12} md={6}>
                <Card><Statistic title="总支出" value={overview.total_expense} precision={2} valueStyle={{ color: "#f5222d" }} /></Card>
              </Col>
              <Col xs={24} sm={12} md={6}>
                <Card><Statistic title="净收支" value={overview.net} precision={2} valueStyle={{ color: overview.net >= 0 ? "#52c41a" : "#f5222d" }} /></Card>
              </Col>
              <Col xs={24} sm={12} md={6}>
                <Card><Statistic title="交易笔数" value={overview.transaction_count} /></Card>
              </Col>
            </Row>
          )}
          <Row gutter={16}>
            <Col xs={24} md={10}>
              <Card title={catType === "expense" ? "支出分类占比" : "收入分类占比"}>
                {catStats.length > 0
                  ? <ReactECharts option={pieOption} style={{ height: 340 }} />
                  : <Typography.Text type="secondary">暂无数据</Typography.Text>}
              </Card>
            </Col>
            <Col xs={24} md={14}>
              <Card title="收支趋势">
                {trend.length > 0
                  ? <ReactECharts option={trendOption} style={{ height: 340 }} />
                  : <Typography.Text type="secondary">暂无数据</Typography.Text>}
              </Card>
            </Col>
          </Row>
        </Space>
      )}
    </div>
  );
}
