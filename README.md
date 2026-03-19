# Brollo_plus

按 `系统设计/` 文档落地的记账系统（Rust 微服务 + React 前端）。

## 本地启动（开发）

### 依赖
- Docker Desktop（支持 `docker compose`）
- Node.js 18+
- Rust stable（建议 1.75+）

### 1) 复制环境变量

```bash
cp .env.example .env
```

### 2) 启动依赖与网关

```bash
docker compose -f docker-compose.dev.yml up -d
```

默认端口：
- Nginx 网关：`http://localhost:8000`
- 前端（本地跑 Vite 时）：`http://localhost:5173`
- Postgres：`localhost:5432`
- MongoDB：默认不映射到宿主机端口（避免与本机 MongoDB 冲突）
- Redis：`localhost:6380`（避免与本机已有 Redis 冲突）
- RabbitMQ 管理台：`http://localhost:15672`
- TimescaleDB：`localhost:5433`

### 3) 启动后端（先跑 user-service）

```bash
cd services
cargo run -p user-service
```

健康检查：
- `GET http://localhost:8001/health`
- 经网关：`GET http://localhost:8000/api/v1/health`（路由到 user-service）

### 4) 启动前端

```bash
cd frontend
npm install
npm run dev
```

## 项目结构

```
.
├── docker-compose.dev.yml
├── infra/
│   └── nginx/
│       └── nginx.conf
├── services/                 # Rust workspace（微服务）
│   ├── Cargo.toml
│   ├── common/
│   └── user-service/
└── frontend/                 # React + Vite + TS
```

