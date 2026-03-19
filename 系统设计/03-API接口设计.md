# API 接口设计

## 1. API 设计原则

### 1.1 RESTful 规范
- 使用标准HTTP方法: GET, POST, PUT, PATCH, DELETE
- 资源命名使用复数名词
- 使用HTTP状态码表示结果
- 统一的错误响应格式

### 1.2 版本控制
- API版本通过URL路径指定: `/api/v1/...`
- 保持向后兼容，弃用API使用`Deprecated`头

### 1.3 认证授权
- 使用JWT Bearer Token认证
- 刷新令牌机制延长会话
- RBAC权限控制

## 2. 通用规范

### 2.1 请求头

```http
Authorization: Bearer <jwt_token>
Content-Type: application/json
Accept: application/json
X-Request-ID: <uuid>           # 请求追踪ID
X-Client-Version: 1.0.0         # 客户端版本
```

### 2.2 统一响应格式

#### 成功响应
```json
{
  "success": true,
  "data": {
    // 实际数据
  },
  "message": "操作成功",
  "timestamp": "2024-12-02T10:30:00Z",
  "request_id": "uuid-xxx"
}
```

#### 错误响应
```json
{
  "success": false,
  "error": {
    "code": "INVALID_INPUT",
    "message": "输入参数无效",
    "details": {
      "field": "amount",
      "reason": "金额必须大于0"
    }
  },
  "timestamp": "2024-12-02T10:30:00Z",
  "request_id": "uuid-xxx"
}
```

### 2.3 分页响应

```json
{
  "success": true,
  "data": {
    "items": [...],
    "pagination": {
      "total": 1523,
      "page": 1,
      "page_size": 20,
      "total_pages": 77,
      "has_next": true,
      "has_prev": false,
      "next_cursor": "cursor_xxx"
    }
  }
}
```

### 2.4 错误码定义

| 错误码 | HTTP状态码 | 说明 |
|-------|-----------|------|
| `SUCCESS` | 200 | 成功 |
| `CREATED` | 201 | 资源创建成功 |
| `NO_CONTENT` | 204 | 成功但无返回内容 |
| `INVALID_INPUT` | 400 | 输入参数无效 |
| `UNAUTHORIZED` | 401 | 未授权 |
| `FORBIDDEN` | 403 | 无权限 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `CONFLICT` | 409 | 资源冲突 |
| `RATE_LIMIT_EXCEEDED` | 429 | 请求频率超限 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |
| `SERVICE_UNAVAILABLE` | 503 | 服务暂时不可用 |

## 3. 认证授权 API

### 3.1 用户注册

```http
POST /api/v1/auth/register
```

**请求体**:
```json
{
  "username": "zhangsan",
  "email": "zhangsan@example.com",
  "password": "StrongP@ssw0rd",
  "full_name": "张三"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "username": "zhangsan",
      "email": "zhangsan@example.com",
      "full_name": "张三",
      "created_at": "2024-12-02T10:30:00Z"
    },
    "tokens": {
      "access_token": "eyJhbGc...",
      "refresh_token": "eyJhbGc...",
      "expires_in": 7200
    }
  }
}
```

### 3.2 用户登录

```http
POST /api/v1/auth/login
```

**请求体**:
```json
{
  "username": "zhangsan",
  "password": "StrongP@ssw0rd",
  "device_info": {
    "device_type": "web",
    "device_name": "Chrome on Windows",
    "ip_address": "192.168.1.100"
  }
}
```

**响应**: 同注册响应

### 3.3 刷新令牌

```http
POST /api/v1/auth/refresh
```

**请求体**:
```json
{
  "refresh_token": "eyJhbGc..."
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGc...",
    "expires_in": 7200
  }
}
```

### 3.4 登出

```http
POST /api/v1/auth/logout
```

**响应**:
```json
{
  "success": true,
  "message": "登出成功"
}
```

## 4. 用户管理 API

### 4.1 获取当前用户信息

```http
GET /api/v1/users/me
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439011",
    "username": "zhangsan",
    "email": "zhangsan@example.com",
    "full_name": "张三",
    "avatar_url": "https://...",
    "phone": "+8613800138000",
    "settings": {
      "default_currency": "CNY",
      "timezone": "Asia/Shanghai",
      "language": "zh-CN",
      "theme": "light"
    },
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### 4.2 更新用户信息

```http
PATCH /api/v1/users/me
```

**请求体**:
```json
{
  "full_name": "张三三",
  "avatar_url": "https://new-avatar.com/avatar.jpg",
  "phone": "+8613800138001"
}
```

### 4.3 更新用户设置

```http
PUT /api/v1/users/me/settings
```

**请求体**:
```json
{
  "default_currency": "USD",
  "timezone": "America/New_York",
  "language": "en-US",
  "theme": "dark",
  "notifications": {
    "email": true,
    "push": true,
    "budget_alert": true
  }
}
```

### 4.4 修改密码

```http
POST /api/v1/users/me/password
```

**请求体**:
```json
{
  "old_password": "OldP@ssw0rd",
  "new_password": "NewP@ssw0rd"
}
```

## 5. 账户管理 API

### 5.1 创建账户

```http
POST /api/v1/accounts
```

**请求体**:
```json
{
  "name": "招商银行储蓄卡",
  "type": "debit_card",
  "currency": "CNY",
  "initial_balance": 10000.00,
  "icon": "bank",
  "color": "#1890ff",
  "description": "工资卡",
  "meta": {
    "bank_name": "招商银行",
    "card_number": "****1234"
  }
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439012",
    "user_id": "507f1f77bcf86cd799439011",
    "name": "招商银行储蓄卡",
    "type": "debit_card",
    "currency": "CNY",
    "initial_balance": 10000.00,
    "current_balance": 10000.00,
    "icon": "bank",
    "color": "#1890ff",
    "status": "active",
    "created_at": "2024-12-02T10:30:00Z"
  }
}
```

### 5.2 获取账户列表

```http
GET /api/v1/accounts?status=active&type=debit_card
```

**查询参数**:
- `status`: active/archived (可选)
- `type`: 账户类型 (可选)
- `currency`: 币种 (可选)

**响应**:
```json
{
  "success": true,
  "data": {
    "accounts": [
      {
        "id": "507f1f77bcf86cd799439012",
        "name": "招商银行储蓄卡",
        "type": "debit_card",
        "currency": "CNY",
        "current_balance": 15234.56,
        "icon": "bank",
        "color": "#1890ff"
      }
    ],
    "summary": {
      "total_assets": 45234.56,
      "total_liabilities": 5000.00,
      "net_worth": 40234.56
    }
  }
}
```

### 5.3 获取账户详情

```http
GET /api/v1/accounts/{account_id}
```

**响应**: 单个账户完整信息

### 5.4 更新账户

```http
PATCH /api/v1/accounts/{account_id}
```

**请求体**:
```json
{
  "name": "招商银行工资卡",
  "description": "主要工资卡"
}
```

### 5.5 删除账户

```http
DELETE /api/v1/accounts/{account_id}
```

**响应**:
```json
{
  "success": true,
  "message": "账户已删除"
}
```

### 5.6 获取账户余额历史

```http
GET /api/v1/accounts/{account_id}/balance-history?start_date=2024-11-01&end_date=2024-12-01&interval=day
```

**查询参数**:
- `start_date`: 开始日期
- `end_date`: 结束日期
- `interval`: day/week/month

**响应**:
```json
{
  "success": true,
  "data": {
    "history": [
      {
        "date": "2024-11-01",
        "balance": 10000.00
      },
      {
        "date": "2024-11-02",
        "balance": 9850.50
      }
    ]
  }
}
```

## 6. 交易管理 API

### 6.1 创建交易

```http
POST /api/v1/transactions
```

**请求体**:
```json
{
  "type": "expense",
  "amount": 238.50,
  "currency": "CNY",
  "account_id": "507f1f77bcf86cd799439012",
  "category_id": "507f1f77bcf86cd799439020",
  "tags": ["餐饮", "聚餐"],
  "description": "与朋友聚餐",
  "payee": "海底捞火锅",
  "transaction_date": "2024-12-01T18:30:00Z",
  "location": {
    "latitude": 39.9042,
    "longitude": 116.4074,
    "address": "北京市朝阳区..."
  }
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439030",
    "type": "expense",
    "amount": 238.50,
    "currency": "CNY",
    "account_id": "507f1f77bcf86cd799439012",
    "category_id": "507f1f77bcf86cd799439020",
    "description": "与朋友聚餐",
    "transaction_date": "2024-12-01T18:30:00Z",
    "status": "confirmed",
    "created_at": "2024-12-01T18:35:00Z"
  }
}
```

### 6.2 获取交易列表

```http
GET /api/v1/transactions?page=1&page_size=20&start_date=2024-11-01&end_date=2024-12-01&type=expense&category_id=xxx&account_id=xxx
```

**查询参数**:
- `page`: 页码 (默认1)
- `page_size`: 每页数量 (默认20)
- `start_date`: 开始日期
- `end_date`: 结束日期
- `type`: expense/income/transfer
- `category_id`: 分类ID
- `account_id`: 账户ID
- `tags`: 标签 (逗号分隔)
- `search`: 搜索关键词
- `cursor`: 游标分页 (可选)

**响应**:
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "id": "507f1f77bcf86cd799439030",
        "type": "expense",
        "amount": 238.50,
        "currency": "CNY",
        "account": {
          "id": "507f1f77bcf86cd799439012",
          "name": "招商银行储蓄卡"
        },
        "category": {
          "id": "507f1f77bcf86cd799439020",
          "name": "餐饮美食",
          "icon": "food"
        },
        "description": "与朋友聚餐",
        "transaction_date": "2024-12-01T18:30:00Z",
        "tags": ["餐饮", "聚餐"]
      }
    ],
    "pagination": {
      "total": 1523,
      "page": 1,
      "page_size": 20,
      "total_pages": 77
    }
  }
}
```

### 6.3 获取交易详情

```http
GET /api/v1/transactions/{transaction_id}
```

### 6.4 更新交易

```http
PATCH /api/v1/transactions/{transaction_id}
```

### 6.5 删除交易

```http
DELETE /api/v1/transactions/{transaction_id}
```

### 6.6 批量导入交易

```http
POST /api/v1/transactions/import
Content-Type: multipart/form-data
```

**请求体**:
```
file: <csv/excel文件>
account_id: "507f1f77bcf86cd799439012"
auto_categorize: true
skip_duplicates: true
```

**响应**:
```json
{
  "success": true,
  "data": {
    "total": 150,
    "imported": 145,
    "skipped": 5,
    "errors": [],
    "import_id": "import_xxx"
  }
}
```

### 6.7 获取导入状态

```http
GET /api/v1/transactions/import/{import_id}
```

### 6.8 上传附件

```http
POST /api/v1/transactions/{transaction_id}/attachments
Content-Type: multipart/form-data
```

**请求体**:
```
file: <图片文件>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "file_id": "file_xxx",
    "file_name": "receipt.jpg",
    "file_url": "https://...",
    "file_size": 102400
  }
}
```

## 7. 分类管理 API

### 7.1 获取分类列表

```http
GET /api/v1/categories?type=expense
```

**响应**:
```json
{
  "success": true,
  "data": {
    "categories": [
      {
        "id": "507f1f77bcf86cd799439020",
        "name": "餐饮美食",
        "type": "expense",
        "icon": "food",
        "color": "#ff6b6b",
        "is_system": true,
        "children": [
          {
            "id": "507f1f77bcf86cd799439021",
            "name": "快餐",
            "icon": "fast-food"
          }
        ]
      }
    ]
  }
}
```

### 7.2 创建分类

```http
POST /api/v1/categories
```

**请求体**:
```json
{
  "name": "交通出行",
  "type": "expense",
  "icon": "car",
  "color": "#52c41a",
  "parent_id": null
}
```

### 7.3 更新分类

```http
PATCH /api/v1/categories/{category_id}
```

### 7.4 删除分类

```http
DELETE /api/v1/categories/{category_id}
```

## 8. 预算管理 API

### 8.1 创建预算

```http
POST /api/v1/budgets
```

**请求体**:
```json
{
  "name": "2024年12月餐饮预算",
  "type": "monthly",
  "start_date": "2024-12-01",
  "end_date": "2024-12-31",
  "amount": 3000.00,
  "currency": "CNY",
  "category_ids": ["507f1f77bcf86cd799439020"],
  "account_ids": [],
  "alert_thresholds": [50, 80, 100]
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439040",
    "name": "2024年12月餐饮预算",
    "type": "monthly",
    "amount": 3000.00,
    "spent": 0.00,
    "remaining": 3000.00,
    "progress": 0.00,
    "status": "active",
    "created_at": "2024-11-25T00:00:00Z"
  }
}
```

### 8.2 获取预算列表

```http
GET /api/v1/budgets?status=active&type=monthly
```

**响应**:
```json
{
  "success": true,
  "data": {
    "budgets": [
      {
        "id": "507f1f77bcf86cd799439040",
        "name": "2024年12月餐饮预算",
        "type": "monthly",
        "amount": 3000.00,
        "spent": 1456.80,
        "remaining": 1543.20,
        "progress": 48.56,
        "status": "active",
        "categories": [
          {
            "id": "507f1f77bcf86cd799439020",
            "name": "餐饮美食"
          }
        ]
      }
    ]
  }
}
```

### 8.3 获取预算详情

```http
GET /api/v1/budgets/{budget_id}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439040",
    "name": "2024年12月餐饮预算",
    "type": "monthly",
    "start_date": "2024-12-01",
    "end_date": "2024-12-31",
    "amount": 3000.00,
    "spent": 1456.80,
    "remaining": 1543.20,
    "progress": 48.56,
    "categories": [...],
    "alert_thresholds": [
      {
        "percentage": 50,
        "notified": true,
        "notified_at": "2024-12-15T10:00:00Z"
      },
      {
        "percentage": 80,
        "notified": false
      }
    ],
    "prediction": {
      "predicted_total": 3200.00,
      "predicted_exceed": 200.00,
      "confidence": 0.85,
      "predicted_at": "2024-12-02T00:00:00Z"
    },
    "execution_history": [
      {
        "date": "2024-12-01",
        "spent": 250.00,
        "remaining": 2750.00
      }
    ]
  }
}
```

### 8.4 更新预算

```http
PATCH /api/v1/budgets/{budget_id}
```

### 8.5 删除预算

```http
DELETE /api/v1/budgets/{budget_id}
```

## 9. 报表统计 API

### 9.1 获取概览统计

```http
GET /api/v1/reports/overview?start_date=2024-11-01&end_date=2024-12-01
```

**响应**:
```json
{
  "success": true,
  "data": {
    "total_income": 15000.00,
    "total_expense": 8500.00,
    "net": 6500.00,
    "transaction_count": 156,
    "income_change": 5.2,
    "expense_change": -3.5,
    "top_expense_categories": [
      {
        "category_id": "507f1f77bcf86cd799439020",
        "category_name": "餐饮美食",
        "amount": 2345.60,
        "percentage": 27.6
      }
    ],
    "top_income_categories": [...]
  }
}
```

### 9.2 获取分类统计

```http
GET /api/v1/reports/categories?type=expense&start_date=2024-11-01&end_date=2024-12-01
```

**响应**:
```json
{
  "success": true,
  "data": {
    "categories": [
      {
        "category_id": "507f1f77bcf86cd799439020",
        "category_name": "餐饮美食",
        "amount": 2345.60,
        "count": 45,
        "percentage": 27.6,
        "trend": [
          {
            "date": "2024-11-01",
            "amount": 150.00
          }
        ]
      }
    ],
    "total": 8500.00
  }
}
```

### 9.3 获取趋势分析

```http
GET /api/v1/reports/trend?start_date=2024-01-01&end_date=2024-12-01&interval=month&type=expense
```

**响应**:
```json
{
  "success": true,
  "data": {
    "series": [
      {
        "date": "2024-01",
        "income": 15000.00,
        "expense": 8500.00,
        "net": 6500.00
      },
      {
        "date": "2024-02",
        "income": 15200.00,
        "expense": 8700.00,
        "net": 6500.00
      }
    ]
  }
}
```

### 9.4 获取账户分析

```http
GET /api/v1/reports/accounts?start_date=2024-11-01&end_date=2024-12-01
```

### 9.5 导出报表

```http
POST /api/v1/reports/export
```

**请求体**:
```json
{
  "report_type": "transactions",
  "format": "excel",
  "start_date": "2024-11-01",
  "end_date": "2024-12-01",
  "filters": {
    "account_ids": [],
    "category_ids": []
  }
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "download_url": "https://.../report_20241202.xlsx",
    "expires_at": "2024-12-02T12:00:00Z"
  }
}
```

## 10. 行情服务 API

### 10.1 获取最新汇率

```http
GET /api/v1/quotes/exchange-rates?base=CNY&targets=USD,EUR,JPY
```

**响应**:
```json
{
  "success": true,
  "data": {
    "base": "CNY",
    "rates": {
      "USD": 0.1389,
      "EUR": 0.1278,
      "JPY": 20.56
    },
    "updated_at": "2024-12-02T10:00:00Z"
  }
}
```

### 10.2 获取历史汇率

```http
GET /api/v1/quotes/exchange-rates/history?base=CNY&target=USD&start_date=2024-11-01&end_date=2024-12-01
```

### 10.3 获取资产净值

```http
GET /api/v1/quotes/net-worth?currency=CNY
```

**响应**:
```json
{
  "success": true,
  "data": {
    "net_worth": 45234.56,
    "currency": "CNY",
    "accounts": [
      {
        "account_id": "507f1f77bcf86cd799439012",
        "name": "招商银行储蓄卡",
        "balance": 15234.56,
        "currency": "CNY",
        "converted_balance": 15234.56
      }
    ],
    "calculated_at": "2024-12-02T10:30:00Z"
  }
}
```

## 11. 仪表盘 API

### 11.1 获取仪表盘配置

```http
GET /api/v1/dashboards?is_default=true
```

### 11.2 创建仪表盘

```http
POST /api/v1/dashboards
```

**请求体**:
```json
{
  "name": "我的首页",
  "is_default": true,
  "layout": [
    {
      "widget_id": "net_worth",
      "position": { "x": 0, "y": 0, "w": 6, "h": 4 },
      "config": {
        "show_trend": true,
        "time_range": "1M"
      }
    }
  ]
}
```

### 11.3 更新仪表盘

```http
PATCH /api/v1/dashboards/{dashboard_id}
```

### 11.4 删除仪表盘

```http
DELETE /api/v1/dashboards/{dashboard_id}
```

### 11.5 获取小部件数据

```http
GET /api/v1/dashboards/widgets/{widget_id}/data?config=...
```

## 12. 系统管理 API

### 12.1 健康检查

```http
GET /api/v1/health
```

**响应**:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2024-12-02T10:30:00Z",
  "services": {
    "database": "healthy",
    "cache": "healthy",
    "queue": "healthy"
  }
}
```

### 12.2 系统信息

```http
GET /api/v1/system/info
```

**响应**:
```json
{
  "success": true,
  "data": {
    "version": "1.0.0",
    "build_time": "2024-12-01T00:00:00Z",
    "uptime": 86400,
    "environment": "production"
  }
}
```

## 13. WebSocket 实时通信

### 13.1 连接

```
ws://api.example.com/ws?token=<jwt_token>
```

### 13.2 消息格式

#### 客户端订阅
```json
{
  "action": "subscribe",
  "channel": "budget_alerts",
  "data": {
    "budget_ids": ["507f1f77bcf86cd799439040"]
  }
}
```

#### 服务器推送
```json
{
  "type": "budget_alert",
  "data": {
    "budget_id": "507f1f77bcf86cd799439040",
    "budget_name": "2024年12月餐饮预算",
    "threshold": 80,
    "current_progress": 82.5,
    "message": "预算执行已超过80%"
  },
  "timestamp": "2024-12-02T10:30:00Z"
}
```

### 13.3 支持的频道

- `budget_alerts`: 预算预警
- `transaction_sync`: 交易同步
- `quote_updates`: 行情更新
- `account_balance`: 账户余额变动

## 14. 限流策略

### 14.1 全局限流

- 单IP: 100 req/s
- 单用户: 50 req/s

### 14.2 特定接口限流

| 接口 | 限制 |
|-----|------|
| 登录 | 5 req/min |
| 注册 | 3 req/min |
| 导出报表 | 10 req/hour |
| 批量导入 | 5 req/hour |

### 14.3 限流响应

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "请求频率超限，请稍后再试",
    "retry_after": 60
  }
}
```

响应头:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1701504060
Retry-After: 60
```

## 15. API 文档

### 15.1 Swagger/OpenAPI

访问地址: `http://api.example.com/docs`

### 15.2 Postman Collection

提供完整的Postman集合供测试使用。

## 16. 版本演进

### 16.1 v1.0 (当前版本)
- 核心功能: 账户、交易、预算、报表
- 认证: JWT

### 16.2 v2.0 (规划中)
- GraphQL支持
- 批量操作优化
- 更细粒度的权限控制
