import React, { useEffect, useState, useCallback } from "react";
import { Button, Card, Col, DatePicker, Form, Input, InputNumber, Modal, Progress, Row, Select, Space, Spin, Table, Tabs, Tag, Typography, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { CopyOutlined, ReloadOutlined, DeleteOutlined, PlusOutlined, BankOutlined, UserAddOutlined, TeamOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { familiesApi, type Family, type FamilyMember } from "../api/families";
import { accountsApi, type Account } from "../api/accounts";
import { transactionsApi, type Transaction } from "../api/transactions";
import { budgetsApi, type Budget } from "../api/budgets";
import { categoriesApi, type Category } from "../api/categories";
import { useAuthStore } from "../store/authStore";

const ROLE_LABEL: Record<string,string> = { owner:"家长", admin:"管理员", member:"成员", readonly:"只读" };
const ROLE_COLOR: Record<string,string> = { owner:"#1a1a1a", admin:"#2980b9", member:"#27ae60", readonly:"#888" };
const CURRENCIES = ["CNY","USD","EUR","JPY","GBP","HKD"];
const ACCT_TYPES = [{value:"cash",label:"现金"},{value:"debit_card",label:"储蓄卡"},{value:"credit_card",label:"信用卡"},{value:"investment",label:"投资账户"},{value:"loan",label:"贷款"},{value:"ewallet",label:"电子钱包"}];
const TX_TYPES = [{value:"expense",label:"支出"},{value:"income",label:"收入"},{value:"transfer",label:"转账"}];
const BUDGET_TYPES = [{value:"daily",label:"每日"},{value:"weekly",label:"每周"},{value:"monthly",label:"每月"},{value:"yearly",label:"每年"},{value:"custom",label:"自定义"}];

// Family Accounts Tab
function FamilyAccounts({ family, canWrite }: { family: Family; canWrite: boolean }) {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [summary, setSummary] = useState<{total_assets:number;total_liabilities:number;net_worth:number}|null>(null);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [sub, setSub] = useState(false);
  const [form] = Form.useForm();
  const load = useCallback(async () => {
    setLoading(true);
    try { const r = await accountsApi.listFamily(family.id); setAccounts(r.data.accounts); setSummary(r.data.summary); }
    catch { message.error("加载家庭账户失败"); } finally { setLoading(false); }
  }, [family.id]);
  useEffect(() => { void load(); }, [load]);
  const onOk = async (v: Record<string,unknown>) => {
    setSub(true);
    try {
      await accountsApi.create({ name:v.name as string, type:v.type as string, currency:v.currency as string, initial_balance:v.initial_balance as number, description:v.description as string|undefined, family_id:family.uuid, visibility:"family" });
      message.success("创建成功"); setOpen(false); form.resetFields(); void load();
    } catch { } finally { setSub(false); }
  };
  const cols: ColumnsType<Account> = [
    { title:"账户", dataIndex:"name", render:v=><span><BankOutlined style={{marginRight:6}}/>{v}</span> },
    { title:"类型", dataIndex:"type", render:v=>ACCT_TYPES.find(t=>t.value===v)?.label??v },
    { title:"币种", dataIndex:"currency", width:80 },
    { title:"余额", dataIndex:"current_balance", render:(v:number)=>v.toFixed(2) },
    { title:"状态", dataIndex:"status", width:80, render:v=><Tag color={v==="active"?"green":"default"}>{v==="active"?"正常":"已归档"}</Tag> },
    { title:"备注", dataIndex:"description", ellipsis:true },
  ];
  return (
    <div>
      {summary && (<Row gutter={16} style={{marginBottom:16}}>
        {([{label:"总资产",val:summary.total_assets,color:"#52c41a"},{label:"总负债",val:summary.total_liabilities,color:"#f5222d"},{label:"净资产",val:summary.net_worth,color:"#1890ff"}] as const).map(x=>(
          <Col xs={24} md={8} key={x.label}><Card size="small"><div style={{textAlign:"center"}}><div style={{color:"#888",fontSize:12}}>{x.label}</div><div style={{fontSize:22,fontWeight:700,color:x.color}}>{x.val.toFixed(2)}</div></div></Card></Col>
        ))}
      </Row>)}
      <Card title="家庭共享账户" extra={canWrite&&<Button type="primary" size="small" icon={<PlusOutlined/>} onClick={()=>{form.resetFields();form.setFieldsValue({currency:"CNY",type:"debit_card",initial_balance:0});setOpen(true);}}>新建</Button>}>
        {loading?<Spin/>:<Table rowKey="id" dataSource={accounts} columns={cols} pagination={{pageSize:20}} size="small"/>}
      </Card>
      <Modal title="新建家庭账户" open={open} onCancel={()=>setOpen(false)} onOk={()=>form.submit()} confirmLoading={sub} destroyOnClose>
        <Form form={form} layout="vertical" onFinish={onOk}>
          <Form.Item name="name" label="账户名称" rules={[{required:true}]}><Input/></Form.Item>
          <Form.Item name="type" label="类型" rules={[{required:true}]}><Select options={ACCT_TYPES}/></Form.Item>
          <Form.Item name="currency" label="币种" rules={[{required:true}]}><Select options={CURRENCIES.map(c=>({value:c,label:c}))}/></Form.Item>
          <Form.Item name="initial_balance" label="初始余额" rules={[{required:true}]}><InputNumber style={{width:"100%"}} precision={2}/></Form.Item>
          <Form.Item name="description" label="备注"><Input.TextArea rows={2}/></Form.Item>
        </Form>
      </Modal>
    </div>
  );
}

// Family Transactions Tab
function FamilyTransactions({ family, canWrite, familyAccounts }: { family: Family; canWrite: boolean; familyAccounts: Account[] }) {
  const [txs, setTxs] = useState<Transaction[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [sub, setSub] = useState(false);
  const [cats, setCats] = useState<Category[]>([]);
  const [filterType, setFilterType] = useState<string|undefined>();
  const [formType, setFormType] = useState("expense");
  const [form] = Form.useForm();
  const load = useCallback(async (p=1) => {
    setLoading(true);
    try {
      const params: Record<string,unknown> = { page:p, page_size:20 };
      if (filterType) params.type = filterType;
      const r = await transactionsApi.listFamily(family.id, params);
      setTxs(r.data.transactions); setTotal(r.data.pagination.total); setPage(p);
    } catch { message.error("加载失败"); } finally { setLoading(false); }
  }, [family.id, filterType]);
  useEffect(() => { void load(); categoriesApi.list().then(r=>setCats(r.data.categories)).catch(()=>{}); }, [load]);
  const flat = cats.flatMap(c=>[c,...c.children]);
  const formCats = flat.filter(c=>c.type===(formType==="income"?"income":"expense"));
  const tColor: Record<string,string> = {expense:"red",income:"green",transfer:"blue"};
  const tLabel: Record<string,string> = {expense:"支出",income:"收入",transfer:"转账"};
  const onOk = async (v: Record<string,unknown>) => {
    setSub(true);
    try {
      const d = v.transaction_date as dayjs.Dayjs;
      await transactionsApi.create({ type:v.type as "expense"|"income"|"transfer", amount:v.amount as number, currency:v.currency as string, account_id:v.account_id as string, category_id:v.category_id as string, description:v.description as string|undefined, transaction_date:d.format("YYYY-MM-DD"), tags:[], family_id:family.uuid });
      message.success("记录成功"); setOpen(false); void load();
    } catch { } finally { setSub(false); }
  };
  const cols: ColumnsType<Transaction> = [
    {title:"日期",dataIndex:"transaction_date",width:110},
    {title:"类型",dataIndex:"type",width:80,render:v=><Tag color={tColor[v]}>{tLabel[v]??v}</Tag>},
    {title:"金额",dataIndex:"amount",width:130,render:(v:number,r)=><span style={{color:r.type==="expense"?"#f5222d":"#52c41a",fontWeight:600}}>{r.type==="expense"?"-":"+"}{v.toFixed(2)} {r.currency}</span>},
    {title:"账户",dataIndex:"account_id",render:v=>familyAccounts.find(a=>a.id===v)?.name??v},
    {title:"分类",dataIndex:"category_id",render:v=>flat.find(c=>c.id===v)?.name??v},
    {title:"描述",dataIndex:"description",ellipsis:true},
  ];
  return (
    <div>
      <Card style={{marginBottom:12}}><Space wrap>
        <Select allowClear placeholder="交易类型" style={{width:120}} options={TX_TYPES} value={filterType} onChange={setFilterType}/>
        <Button type="primary" onClick={()=>load()}>查询</Button>
        <Button onClick={()=>setFilterType(undefined)}>重置</Button>
        {canWrite&&<Button type="primary" icon={<PlusOutlined/>} onClick={()=>{form.resetFields();form.setFieldsValue({type:"expense",currency:family.default_currency,transaction_date:dayjs()});setFormType("expense");setOpen(true);}}>记家庭账</Button>}
      </Space></Card>
      <Card><Table rowKey="id" loading={loading} dataSource={txs} columns={cols} size="small" pagination={{current:page,total,pageSize:20,onChange:p=>load(p)}}/></Card>
      {canWrite&&<Modal title="记家庭账" open={open} onCancel={()=>setOpen(false)} onOk={()=>form.submit()} confirmLoading={sub} destroyOnClose>
        <Form form={form} layout="vertical" onFinish={onOk}>
          <Form.Item name="type" label="交易类型" rules={[{required:true}]}><Select options={TX_TYPES} onChange={v=>{setFormType(v);form.setFieldValue("category_id",undefined);}}/></Form.Item>
          <Form.Item name="amount" label="金额" rules={[{required:true,type:"number",min:0.01}]}><InputNumber style={{width:"100%"}} precision={2} min={0.01}/></Form.Item>
          <Form.Item name="currency" label="币种" rules={[{required:true}]}><Select options={CURRENCIES.map(c=>({value:c,label:c}))}/></Form.Item>
          <Form.Item name="account_id" label="账户" rules={[{required:true}]}><Select options={familyAccounts.map(a=>({value:a.id,label:`${a.name} (${a.current_balance.toFixed(2)})`}))} placeholder="请选择家庭账户"/></Form.Item>
          <Form.Item name="category_id" label="分类" rules={[{required:true}]}><Select options={formCats.map(c=>({value:c.id,label:c.name}))} placeholder="请选择分类"/></Form.Item>
          <Form.Item name="transaction_date" label="日期" rules={[{required:true}]}><DatePicker style={{width:"100%"}}/></Form.Item>
          <Form.Item name="description" label="描述"><Input placeholder="可选备注"/></Form.Item>
        </Form>
      </Modal>}
    </div>
  );
}

function FamilyBudgets({ family, canWrite }: { family: Family; canWrite: boolean }) {
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [cats, setCats] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [sub, setSub] = useState(false);
  const [form] = Form.useForm();
  const load = useCallback(async () => {
    setLoading(true);
    try { const r = await budgetsApi.listFamily(family.id); setBudgets(r.data.budgets); }
    catch { message.error("加载家庭预算失败"); } finally { setLoading(false); }
  }, [family.id]);
  useEffect(() => { void load(); categoriesApi.list({type:"expense"}).then(r=>setCats(r.data.categories)).catch(()=>{}); }, [load]);
  const flat = cats.flatMap(c=>[c,...c.children]);
  const onOk = async (v: Record<string,unknown>) => {
    setSub(true);
    try {
      await budgetsApi.create({ name:v.name as string, type:v.type as string, start_date:(v.start_date as dayjs.Dayjs).format("YYYY-MM-DD"), end_date:(v.end_date as dayjs.Dayjs).format("YYYY-MM-DD"), amount:v.amount as number, currency:v.currency as string, category_ids:(v.category_ids as string[]|undefined)??[], family_id:family.uuid, scope:"family" });
      message.success("创建成功"); setOpen(false); form.resetFields(); void load();
    } catch { } finally { setSub(false); }
  };
  const onDel = async (id: string) => { try { await budgetsApi.remove(id); message.success("已删除"); void load(); } catch { } };
  const sColor: Record<string,string> = {active:"green",completed:"blue",paused:"orange"};
  const sLabel: Record<string,string> = {active:"进行中",completed:"已完成",paused:"已暂停"};
  const cols: ColumnsType<Budget> = [
    {title:"预算名称",dataIndex:"name"},
    {title:"周期",dataIndex:"type",width:80,render:v=>BUDGET_TYPES.find(t=>t.value===v)?.label??v},
    {title:"预算额",dataIndex:"amount",width:100,render:(v:number)=>v.toFixed(2)},
    {title:"已支出",dataIndex:"spent",width:100,render:(v:number)=><span style={{color:"#f5222d"}}>{v.toFixed(2)}</span>},
    {title:"剩余",dataIndex:"remaining",width:100,render:(v:number)=><span style={{color:"#52c41a"}}>{v.toFixed(2)}</span>},
    {title:"进度",dataIndex:"progress",width:160,render:(v:number)=><Progress percent={Math.min(Math.round(v),100)} size="small" strokeColor={v<50?"#52c41a":v<80?"#faad14":"#f5222d"}/>},
    {title:"状态",dataIndex:"status",width:90,render:v=><Tag color={sColor[v]}>{sLabel[v]??v}</Tag>},
    {title:"操作",width:80,render:(_,r)=>canWrite?<Button size="small" danger icon={<DeleteOutlined/>} onClick={()=>onDel(r.id)}>删除</Button>:null},
  ];
  return (
    <div>
      <Card title="家庭共享预算" extra={canWrite&&<Button type="primary" size="small" icon={<PlusOutlined/>} onClick={()=>{form.resetFields();const now=dayjs();form.setFieldsValue({type:"monthly",currency:"CNY",start_date:now.startOf("month"),end_date:now.endOf("month")});setOpen(true);}}>新建</Button>}>
        {loading?<Spin/>:<Table rowKey="id" dataSource={budgets} columns={cols} pagination={{pageSize:20}} size="small"/>}
      </Card>
      {canWrite&&<Modal title="新建家庭预算" open={open} onCancel={()=>setOpen(false)} onOk={()=>form.submit()} confirmLoading={sub} destroyOnClose>
        <Form form={form} layout="vertical" onFinish={onOk}>
          <Form.Item name="name" label="预算名称" rules={[{required:true}]}><Input/></Form.Item>
          <Form.Item name="type" label="周期" rules={[{required:true}]}><Select options={BUDGET_TYPES}/></Form.Item>
          <Row gutter={8}><Col span={12}><Form.Item name="start_date" label="开始" rules={[{required:true}]}><DatePicker style={{width:"100%"}}/></Form.Item></Col><Col span={12}><Form.Item name="end_date" label="结束" rules={[{required:true}]}><DatePicker style={{width:"100%"}}/></Form.Item></Col></Row>
          <Row gutter={8}><Col span={14}><Form.Item name="amount" label="金额" rules={[{required:true,type:"number",min:0.01}]}><InputNumber style={{width:"100%"}} precision={2} min={0.01}/></Form.Item></Col><Col span={10}><Form.Item name="currency" label="币种" rules={[{required:true}]}><Select options={CURRENCIES.map(c=>({value:c,label:c}))}/></Form.Item></Col></Row>
          <Form.Item name="category_ids" label="关联分类"><Select mode="multiple" allowClear placeholder="留空=全部" options={flat.map(c=>({value:c.id,label:c.name}))}/></Form.Item>
        </Form>
      </Modal>}
    </div>
  );
}
export default function FamilyPage() {
  const [loading, setLoading] = useState(true);
  const [family, setFamily] = useState<Family | null>(null);
  const [members, setMembers] = useState<FamilyMember[]>([]);
  const [familyAccounts, setFamilyAccounts] = useState<Account[]>([]);
  const [createOpen, setCreateOpen] = useState(false);
  const [joinOpen, setJoinOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [createForm] = Form.useForm();
  const [joinForm] = Form.useForm();
  const [editForm] = Form.useForm();
  const user = useAuthStore((s) => s.user);

  async function load() {
    setLoading(true);
    try {
      const r = await familiesApi.mine();
      const f = r.data.family;
      setFamily(f);
      if (f) {
        const [mr, ar] = await Promise.all([familiesApi.members(f.id), accountsApi.listFamily(f.id)]);
        setMembers(mr.data.members);
        setFamilyAccounts(ar.data.accounts);
      }
    } catch { message.error("加载失败"); } finally { setLoading(false); }
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => { void load(); }, []);

  const myRole = members.find((m) => m.username === user?.username)?.role ?? "";
  const isOwner = myRole === "owner";
  const isAdmin = isOwner || myRole === "admin";
  const canWrite = myRole !== "readonly" && myRole !== "";

  async function handleCreate(v: { name: string; default_currency: string; nickname?: string; description?: string }) {
    setSubmitting(true);
    try { await familiesApi.create(v); message.success("创建成功"); setCreateOpen(false); createForm.resetFields(); await load(); }
    catch { message.error("创建失败"); } finally { setSubmitting(false); }
  }
  async function handleJoin(v: { invite_code: string; nickname?: string }) {
    setSubmitting(true);
    try { await familiesApi.join(v); message.success("加入成功"); setJoinOpen(false); joinForm.resetFields(); await load(); }
    catch { message.error("加入失败"); } finally { setSubmitting(false); }
  }
  async function handleUpdate(v: { name?: string; default_currency?: string; description?: string }) {
    if (!family) return; setSubmitting(true);
    try { await familiesApi.update(family.id, v); message.success("已更新"); setEditOpen(false); await load(); }
    catch { message.error("更新失败"); } finally { setSubmitting(false); }
  }
  async function handleRefreshCode() {
    if (!family) return;
    try { const r = await familiesApi.refreshInvite(family.id); message.success("新邀请码：" + r.data.invite_code); await load(); }
    catch { message.error("刷新失败"); }
  }
  function handleLeave() {
    if (!family) return;
    Modal.confirm({ title:"确认退出家庭？", okText:"退出", okType:"danger", cancelText:"取消",
      onOk: async () => { try { await familiesApi.leave(family.id); message.success("已退出"); await load(); } catch { message.error("退出失败"); } } });
  }
  function handleDissolve() {
    if (!family) return;
    Modal.confirm({ title:"确认解散家庭？", content:"所有成员将被移出。", okText:"解散", okType:"danger", cancelText:"取消",
      onOk: async () => { try { await familiesApi.dissolve(family.id); message.success("已解散"); await load(); } catch { message.error("解散失败"); } } });
  }
  async function handleRoleChange(uid: number, role: string) {
    if (!family) return;
    try { await familiesApi.updateMember(family.id, uid, { role }); message.success("已更新"); await load(); }
    catch { message.error("更新失败"); }
  }
  function handleRemove(uid: number, uname: string) {
    if (!family) return;
    Modal.confirm({ title:`移除成员 ${uname}？`, okText:"移除", okType:"danger", cancelText:"取消",
      onOk: async () => { try { await familiesApi.removeMember(family.id, uid); message.success("已移除"); await load(); } catch { message.error("移除失败"); } } });
  }

  const memberCols: ColumnsType<FamilyMember> = [
    { title:"成员", key:"user", render:(_,m)=><Space><b>{m.nickname||m.username}</b>{m.nickname&&<span style={{color:"#888",fontSize:12}}>({m.username})</span>}</Space> },
    { title:"角色", dataIndex:"role", width:130, render:(role,m)=>isOwner&&role!=="owner"?<Select size="small" value={role} onChange={(v)=>handleRoleChange(m.user_id,v)} style={{width:100}} options={[{value:"admin",label:"管理员"},{value:"member",label:"成员"},{value:"readonly",label:"只读"}]}/>:<Tag style={{border:"1.5px solid #1a1a1a",color:ROLE_COLOR[role],fontWeight:700}}>{ROLE_LABEL[role]??role}</Tag> },
    { title:"邮箱", dataIndex:"email", width:190 },
    { title:"加入时间", dataIndex:"joined_at", width:110, render:(v:string)=>v?.slice(0,10) },
    { title:"操作", key:"action", width:70, render:(_,m)=>isAdmin&&m.role!=="owner"?<Button size="small" danger icon={<DeleteOutlined/>} onClick={()=>handleRemove(m.user_id,m.username)}/>:null },
  ];

  if (loading) return <div style={{display:"grid",placeItems:"center",height:300}}><Spin/></div>;
  if (!family) return (
    <div>
      <Typography.Title level={3}>家庭管理</Typography.Title>
      <Card style={{textAlign:"center",padding:40}}>
        <TeamOutlined style={{fontSize:48,color:"#ccc",marginBottom:16,display:"block"}}/>
        <Typography.Title level={4} style={{marginBottom:8}}>你还没有加入家庭</Typography.Title>
        <Typography.Text type="secondary">创建一个家庭，或使用邀请码加入已有家庭</Typography.Text>
        <Row gutter={16} justify="center" style={{marginTop:24}}>
          <Col><Button type="primary" icon={<PlusOutlined/>} size="large" onClick={()=>setCreateOpen(true)}>创建家庭</Button></Col>
          <Col><Button icon={<UserAddOutlined/>} size="large" onClick={()=>setJoinOpen(true)}>加入家庭</Button></Col>
        </Row>
      </Card>
      <Modal title="创建家庭" open={createOpen} onCancel={()=>setCreateOpen(false)} footer={null}>
        <Form form={createForm} layout="vertical" onFinish={handleCreate}>
          <Form.Item name="name" label="家庭名称" rules={[{required:true}]}><Input placeholder="如：李家账本"/></Form.Item>
          <Form.Item name="nickname" label="你的昵称"><Input placeholder="如：爸爸"/></Form.Item>
          <Form.Item name="default_currency" label="默认币种" initialValue="CNY"><Select options={[{value:"CNY",label:"CNY"},{value:"USD",label:"USD"},{value:"EUR",label:"EUR"}]}/></Form.Item>
          <Form.Item name="description" label="备注"><Input.TextArea rows={2}/></Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>创建</Button>
        </Form>
      </Modal>
      <Modal title="加入家庭" open={joinOpen} onCancel={()=>setJoinOpen(false)} footer={null}>
        <Form form={joinForm} layout="vertical" onFinish={handleJoin}>
          <Form.Item name="invite_code" label="邀请码" rules={[{required:true,len:6}]}><Input placeholder="6位邀请码" maxLength={6} style={{textTransform:"uppercase",fontSize:20,letterSpacing:6}}/></Form.Item>
          <Form.Item name="nickname" label="你的昵称"><Input placeholder="如：妈妈"/></Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>加入</Button>
        </Form>
      </Modal>
    </div>
  );

  return (
    <div>
      <Typography.Title level={3}>{family.name}</Typography.Title>
      <Tabs defaultActiveKey="info" items={[
        { key:"info", label:"家庭信息", children: (
          <div>
            <Row gutter={16} style={{marginBottom:16}}>
              <Col xs={24} md={12}>
                <Card title="家庭信息" extra={isAdmin&&<Button size="small" onClick={()=>{editForm.setFieldsValue({name:family.name,default_currency:family.default_currency,description:family.description});setEditOpen(true);}}>编辑</Button>}>
                  <Space direction="vertical" style={{width:"100%"}}>
                    <div><div style={{color:"#888",fontSize:12}}>家庭名称</div><div style={{fontSize:18,fontWeight:700}}>{family.name}</div></div>
                    {family.description&&<div style={{color:"#888"}}>{family.description}</div>}
                    <div><div style={{color:"#888",fontSize:12}}>默认币种</div><div style={{fontWeight:600}}>{family.default_currency}</div></div>
                    <div><div style={{color:"#888",fontSize:12}}>成员数</div><div style={{fontWeight:600}}>{family.member_count} 人</div></div>
                    <Space style={{marginTop:8}}>
                      {!isOwner&&<Button danger size="small" onClick={handleLeave}>退出家庭</Button>}
                      {isOwner&&<Button danger size="small" onClick={handleDissolve}>解散家庭</Button>}
                    </Space>
                  </Space>
                </Card>
              </Col>
              <Col xs={24} md={12}>
                <Card title="邀请码">
                  <Space direction="vertical" style={{width:"100%"}}>
                    <div style={{display:"flex",alignItems:"center",gap:12}}>
                      <span style={{fontSize:28,fontWeight:700,letterSpacing:6,border:"2px dashed #1a1a1a",padding:"8px 16px"}}>{family.invite_code}</span>
                      <Button icon={<CopyOutlined/>} onClick={()=>{navigator.clipboard.writeText(family.invite_code);message.success("已复制");}}>复制</Button>
                    </div>
                    <Typography.Text type="secondary" style={{fontSize:12}}>将邀请码分享给家人，通过加入家庭输入即可加入。</Typography.Text>
                    {isAdmin&&<Button size="small" icon={<ReloadOutlined/>} onClick={handleRefreshCode}>刷新邀请码</Button>}
                  </Space>
                </Card>
              </Col>
            </Row>
            <Card title={`成员列表（${members.length} 人）`}>
              <Table rowKey="id" size="small" columns={memberCols} dataSource={members} pagination={false}/>
            </Card>
          </div>
        )},
        { key:"accounts", label:"家庭账户", children: <FamilyAccounts family={family} canWrite={canWrite}/> },
        { key:"transactions", label:"家庭账单", children: <FamilyTransactions family={family} canWrite={canWrite} familyAccounts={familyAccounts}/> },
        { key:"budgets", label:"家庭预算", children: <FamilyBudgets family={family} canWrite={canWrite}/> },
      ]}/>
      <Modal title="编辑家庭信息" open={editOpen} onCancel={()=>setEditOpen(false)} footer={null}>
        <Form form={editForm} layout="vertical" onFinish={handleUpdate}>
          <Form.Item name="name" label="家庭名称" rules={[{required:true}]}><Input/></Form.Item>
          <Form.Item name="default_currency" label="默认币种"><Select options={[{value:"CNY",label:"CNY"},{value:"USD",label:"USD"},{value:"EUR",label:"EUR"}]}/></Form.Item>
          <Form.Item name="description" label="备注"><Input.TextArea rows={2}/></Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>保存</Button>
        </Form>
      </Modal>
    </div>
  );
}
