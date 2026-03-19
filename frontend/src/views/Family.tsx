import React, { useEffect, useState } from "react";
import { Button, Card, Col, Form, Input, Modal, Row, Select, Space, Spin, Table, Tag, Typography, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { CopyOutlined, TeamOutlined, UserAddOutlined, ReloadOutlined, DeleteOutlined, PlusOutlined } from "@ant-design/icons";
import { familiesApi, type Family, type FamilyMember } from "../api/families";
import { useAuthStore } from "../store/authStore";

const ROLE_LABEL: Record<string,string> = { owner:"家长", admin:"管理员", member:"成员", readonly:"只读" };
const ROLE_COLOR: Record<string,string> = { owner:"#1a1a1a", admin:"#2980b9", member:"#27ae60", readonly:"#888" };

export default function FamilyPage() {
  const [loading, setLoading] = useState(true);
  const [family, setFamily] = useState<Family | null>(null);
  const [members, setMembers] = useState<FamilyMember[]>([]);
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
      if (f) { const mr = await familiesApi.members(f.id); setMembers(mr.data.members); }
    } catch { /**/ } finally { setLoading(false); }
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => { void load(); }, []);

  const myRole = members.find((m) => m.username === user?.username)?.role ?? "";
  const isOwner = myRole === "owner";
  const isAdmin = isOwner || myRole === "admin";

  async function handleCreate(v: { name: string; default_currency: string; nickname?: string; description?: string }) {
    setSubmitting(true);
    try { await familiesApi.create(v); message.success("创建成功"); setCreateOpen(false); createForm.resetFields(); await load(); }
    catch { message.error("创建失败"); } finally { setSubmitting(false); }
  }
  async function handleJoin(v: { invite_code: string; nickname?: string }) {
    setSubmitting(true);
    try { await familiesApi.join(v); message.success("加入成功"); setJoinOpen(false); joinForm.resetFields(); await load(); }
    catch { message.error("加入失败，请检查邀请码"); } finally { setSubmitting(false); }
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

  const columns: ColumnsType<FamilyMember> = [
    { title:"成员", key:"user", render:(_,m)=><Space><b>{m.nickname||m.username}</b>{m.nickname&&<span style={{color:"#888",fontSize:12}}>({m.username})</span>}</Space> },
    { title:"角色", dataIndex:"role", width:130, render:(role,m)=>isOwner&&role!=="owner" ? <Select size="small" value={role} onChange={(v)=>handleRoleChange(m.user_id,v)} style={{width:95}} options={[{value:"admin",label:"管理员"},{value:"member",label:"成员"},{value:"readonly",label:"只读"}]}/> : <Tag style={{border:"1.5px solid #1a1a1a",color:ROLE_COLOR[role],fontWeight:700}}>{ROLE_LABEL[role]??role}</Tag> },
    { title:"邮箱", dataIndex:"email", width:190 },
    { title:"加入时间", dataIndex:"joined_at", width:110, render:(v:string)=>v?.slice(0,10) },
    { title:"操作", key:"action", width:70, render:(_,m)=>isAdmin&&m.role!=="owner" ? <Button size="small" danger icon={<DeleteOutlined/>} onClick={()=>handleRemove(m.user_id,m.username)}/> : null },
  ];

  if (loading) return <div style={{display:"grid",placeItems:"center",height:300}}><Spin/></div>;

  if (!family) return (
    <div>
      <Typography.Title level={3}>🏠 家庭管理</Typography.Title>
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
      <Typography.Title level={3}>🏠 {family.name}</Typography.Title>
      <Row gutter={16} style={{marginBottom:16}}>
        <Col xs={24} md={12}>
          <Card title="家庭信息" extra={isAdmin&&<Button size="small" onClick={()=>{editForm.setFieldsValue({name:family.name,default_currency:family.default_currency,description:family.description});setEditOpen(true);}}>编辑</Button>}>
            <Space direction="vertical" style={{width:"100%"}}>
              <div><div style={{color:"#888",fontSize:12}}>家庭名称</div><div style={{fontSize:18,fontWeight:700}}>{family.name}</div></div>
              {family.description&&<div style={{color:"#888"}}>{family.description}</div>}
              <div><div style={{color:"#888",fontSize:12}}>默认币种</div><div style={{fontWeight:600}}>{family.default_currency}</div></div>
              <div><div style={{color:"#888",fontSize:12}}>成员</div><div style={{fontWeight:600}}>{family.member_count} 人</div></div>
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
              <Typography.Text type="secondary" style={{fontSize:12}}>将邀请码分享给家人，通过「加入家庭」输入即可加入。</Typography.Text>
              {isAdmin&&<Button size="small" icon={<ReloadOutlined/>} onClick={handleRefreshCode}>刷新邀请码</Button>}
            </Space>
          </Card>
        </Col>
      </Row>
      <Card title={`成员列表（${members.length} 人）`}>
        <Table rowKey="id" size="small" columns={columns} dataSource={members} pagination={false}/>
      </Card>
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
