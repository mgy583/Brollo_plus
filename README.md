# Brollo+

基于 Rust 微服务 + React 的家庭记账系统。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | React 18 + TypeScript + Vite + Ant Design + ECharts + Zustand |
| 后端 | Rust (Axum) 微服务架构 |
| 网关 | Nginx 反向代理 |
| 存储 | PostgreSQL (用户/认证) · MongoDB (业务数据) · TimescaleDB (时序) · Redis (缓存) |
| 消息队列 | RabbitMQ |
| 容器化 | Docker Compose |

## 系统架构

```
                  ┌─────────────┐
                  │  React SPA  │ :5173 (Vite dev)
                  └──────┬──────┘
                         │ /api/v1/*
                  ┌──────▼──────┐
                  │    Nginx    │ :8000
                  └──────┬──────┘
        ┌────────┬───────┼───────┬────────┬────────┐
        ▼        ▼       ▼       ▼        ▼        ▼
   user-svc  account  txn-svc  budget  report  quote
    :8001     :8002    :8003    :8004   :8005   :8006
        │        │       │       │        │        │
   Postgres   MongoDB  MongoDB  Mongo   MongoDB  Timescale
              Redis    Redis   Timescale Redis   Redis
                      RabbitMQ RabbitMQ
```

## 项目结构

```
.
├── docker-compose.dev.yml          # 开发环境编排（全部服务 + 基础设施）
├── infra/
│   └── nginx/nginx.conf            # API 网关路由配置
├── services/                       # Rust workspace
│   ├── Cargo.toml                  # workspace 根配置
│   ├── common/                     # 公共库（JWT、响应格式、中间件等）
│   ├── user-service/               # 用户认证 + 家庭管理 (PostgreSQL)
│   ├── account-service/            # 账户 + 分类管理 (MongoDB)
│   ├── transaction-service/        # 交易记录 (MongoDB + RabbitMQ)
│   ├── budget-service/             # 预算管理 (MongoDB + TimescaleDB)
│   ├── report-service/             # 统计报表 (MongoDB)
│   └── quote-service/              # 汇率服务 (TimescaleDB)
└── frontend/                       # React SPA
    ├── src/
    │   ├── api/                    # Axios API 层
    │   ├── store/                  # Zustand 状态管理
    │   ├── ui/                     # 布局、路由守卫
    │   └── views/                  # 页面组件
    ├── package.json
    └── vite.config.ts
```

## 功能模块

- **仪表盘** — 资产概览、收支趋势、分类占比、预算进度
- **账户管理** — 多账户（储蓄/信用卡等）增删改查
- **交易记录** — 收入/支出/转账，支持分类筛选
- **预算管理** — 按分类/时间周期设定预算，实时追踪
- **统计报表** — 多维度收支分析
- **家庭管理** — 邀请码加入家庭，角色权限管理
- **系统设置** — 个人资料、偏好设置（货币/时区/语言/主题）、修改密码

## 快速开始

### 环境要求

- Docker Desktop（支持 `docker compose`）
- Node.js 18+

### 1. 配置环境变量

```bash
cp .env.example .env
```

### 2. 一键启动全部后端

```bash
docker compose -f docker-compose.dev.yml up -d
```

首次启动需要编译 Rust 项目，耗时较长（约 5-10 分钟），后续启动会利用缓存。

### 3. 启动前端开发服务器

```bash
cd frontend
npm install
npm run dev
```

### 4. 访问应用

- 前端：http://localhost:5173
- API 网关：http://localhost:8000
- RabbitMQ 管理台：http://localhost:15672（账号 `abook` / `abook_password`）

### 默认端口映射

| 服务 | 端口 |
|------|------|
| Vite 前端 | 5173 |
| Nginx 网关 | 8000 |
| user-service | 8001 |
| account-service | 8002 |
| transaction-service | 8003 |
| budget-service | 8004 |
| report-service | 8005 |
| quote-service | 8006 |
| Redis | 6380 (映射到宿主机) |
| RabbitMQ 管理台 | 15672 |

## 常用命令

```bash
# 查看所有服务状态
docker compose -f docker-compose.dev.yml ps

# 查看某服务日志
docker compose -f docker-compose.dev.yml logs -f user-service

# 停止所有服务
docker compose -f docker-compose.dev.yml down

# 停止并清除数据卷（慎用）
docker compose -f docker-compose.dev.yml down -v
```
