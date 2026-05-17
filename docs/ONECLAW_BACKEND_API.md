# OneClaw 后端 API 对接文档

## 1. 概述

| 项目 | 值 |
|------|-----|
| 后端地址 | `https://api.oneclaw.net` |
| 客户端 | OneClaw 桌面应用 (Electron) |
| 认证方式 | 登录获取 uid + token，所有模型请求经后端代理 |
| 设备安全 | Ed25519 密钥对绑定 |

## 2. 整体架构

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────┐
│  OneClaw 桌面端  │────→│  api.oneclaw.net │────→│  OpenAI 等    │
│  (Electron)     │←────│  (后端代理)       │←────│  (模型提供商)  │
└─────────────────┘     └──────────────────┘     └──────────────┘

客户端不持有真实 API Key，所有模型请求通过后端代理。
客户端 apiKey 固定为 "oneclaw-placeholder"。
后端替换为真实 Key 后转发到模型提供商。
```

## 3. 接口列表

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/auth/login` | 用户登录 | 无 |
| POST | `/api/auth/pair-device` | 设备绑定 | uid + token |
| POST | `/api/auth/providers` | 获取可用模型列表 | uid + token |
| POST | `/api/chat/completions` | 代理聊天请求 | uid + token + device |

---

## 4. 接口详情

### 4.1 用户登录

```
POST /api/auth/login
```

**请求体：**
```json
{
  "email": "user@example.com",
  "password": "plain_password"
}
```

**成功响应 (200)：**
```json
{
  "success": true,
  "uid": "u_abc123def456",
  "token": "ot_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "user": {
    "id": "u_abc123def456",
    "email": "user@example.com",
    "name": "张三",
    "avatar": "https://cdn.oneclaw.net/avatars/u_abc123.png"
  }
}
```

**失败响应：**
```json
{
  "success": false,
  "error": "Invalid email or password"
}
```

**实现要点：**
- uid 格式建议：`u_` + 12 位随机 hex
- token 格式建议：`ot_` + 32 位随机 hex (一次性生成，长期有效)
- 密码使用 bcrypt/argon2 哈希存储
- 可支持邮箱验证码登录(预留 `type: "email_code"` 字段)

---

### 4.2 设备绑定

```
POST /api/auth/pair-device
```

**请求头：**
```
x-device-id: device-1712345678901-abc12345
x-device-pubkey: MCowBQYDK2VwAyEA...（Ed25519 公钥 base64）
```

**请求体：**
```json
{
  "uid": "u_abc123def456",
  "token": "ot_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "devicePublicKey": "MCowBQYDK2VwAyEA...",
  "deviceName": "MacBook Pro 16\"",       // 可选，设备名称
  "platform": "darwin",                    // 可选，darwin/win32/linux
  "appVersion": "1.0.0"                   // 可选，OneClaw 版本
}
```

**成功响应 (200)：**
```json
{
  "success": true,
  "deviceId": "dev_1712345678901",
  "pairedAt": "2026-05-18T10:30:00Z",
  "deviceCount": 2
}
```

**失败响应：**
```json
{
  "success": false,
  "error": "Device limit reached. Max 5 devices per account."
}
```

**实现要点：**
- 存储 `{ uid, deviceId, publicKey, deviceName, platform, pairedAt }`
- 限制每个账号最多 **5 个设备**
- 支持重复配对（同一 deviceId + uid 更新 publicKey）
- 支持解绑：`DELETE /api/auth/devices/:deviceId`
- `x-device-pubkey` 是从 `x-device-id` 对应的公钥来的，供后端校验签名时使用
- 后续可扩展为请求签名：客户端用私钥签名请求体，后端用公钥验证

---

### 4.3 获取模型列表

```
POST /api/auth/providers
```

**请求头：**
```
x-device-id: device-1712345678901-abc12345
x-device-pubkey: MCowBQYDK2VwAyEA...
```

**请求体：**
```json
{
  "uid": "u_abc123def456",
  "token": "ot_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

**成功响应 (200)：**
```json
{
  "success": true,
  "providers": [
    {
      "vendorId": "openai",
      "label": "OpenAI (GPT-4o)",
      "model": "gpt-4o",
      "baseUrl": "https://api.oneclaw.net/proxy/openai",
      "apiKey": "oneclaw-placeholder",
      "enabled": true,
      "isDefault": true
    },
    {
      "vendorId": "deepseek",
      "label": "DeepSeek V3",
      "model": "deepseek-chat",
      "baseUrl": "https://api.oneclaw.net/proxy/deepseek",
      "apiKey": "oneclaw-placeholder",
      "enabled": true,
      "isDefault": false
    },
    {
      "vendorId": "anthropic",
      "label": "Claude Sonnet 4.5",
      "model": "claude-sonnet-4-5",
      "baseUrl": "https://api.oneclaw.net/proxy/anthropic",
      "apiKey": "oneclaw-placeholder",
      "enabled": true,
      "isDefault": false
    }
  ]
}
```

**失败响应 (401)：**
```json
{
  "success": false,
  "error": "Invalid credentials"
}
```

**实现要点：**
- apiKey **永远返回 `"oneclaw-placeholder"`**，不能返回真实 Key
- baseUrl 指向 OneClaw 后端的代理端点，格式：`https://api.oneclaw.net/proxy/{vendorId}`
- 根据用户套餐/余额返回不同的模型列表
- 已登录用户默认有 1 个默认模型，付费后解锁更多

---

### 4.4 代理聊天请求

```
POST /proxy/{vendorId}/chat/completions
```

**请求头：**
```
x-auth-uid: u_abc123def456
x-auth-token: ot_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
x-device-id: device-1712345678901-abc12345
Content-Type: application/json
```

**请求体：**
```
透传 OpenAI-compatible 格式
```

**实现要点：**
1. 校验 `x-auth-uid` + `x-auth-token` 是不是有效会话
2. 校验 `x-device-id` 是否是已绑定设备
3. 如果是 → 查询后端存储的该 vendor 真实 API Key → 替换请求头 → 转发到目标模型提供商
4. 如果否 → 返回 401

**校验流程：**
```
收到请求
  │
  ├─ 1. 验证 uid + token → 无效 → 401
  │
  ├─ 2. 验证 deviceId 已绑定 → 未绑定 → 403 "Device not paired"
  │
  ├─ 3. 查询该用户的 provider 配置 → 无此 vendor → 403 "Provider not available"
  │
  ├─ 4. 用真实 API Key 替换 authorization 头
  │
  ├─ 5. 转发到目标模型提供商
  │
  └─ 6. 流式返回响应（SSE）
```

**计费相关（可选）：**
- 每个请求记录 token 用量
- 扣减用户配额
- 返回 `x-usage-tokens: 1500` 头告知用量

---

## 5. 数据存储设计

### 5.1 用户表 `users`
```
id          VARCHAR(32)   PK    u_abc123def456
email       VARCHAR(255)  UNIQUE
password    VARCHAR(255)        bcrypt hash
name        VARCHAR(100)
avatar      VARCHAR(500)
plan        VARCHAR(20)         free | pro | enterprise
quotaDaily  INT                每日 token 上限, 0=无限
quotaUsed   INT                已用 token
createdAt   TIMESTAMP
updatedAt   TIMESTAMP
```

### 5.2 会话表 `sessions`
```
id          VARCHAR(32)   PK    ot_xxxxxxxx...
uid         VARCHAR(32)   FK    → users.id
token       VARCHAR(64)   UNIQUE
expiresAt   TIMESTAMP          NULL=永不过期
createdAt   TIMESTAMP
lastUsedAt  TIMESTAMP
```

### 5.3 设备表 `devices`
```
id          VARCHAR(64)   PK    客户端生成的 deviceId
uid         VARCHAR(32)   FK    → users.id
publicKey   TEXT                Ed25519 公钥 base64
name        VARCHAR(200)        设备名称
platform    VARCHAR(20)         darwin/win32/linux
pairedAt    TIMESTAMP
lastSeenAt  TIMESTAMP
revoked     BOOLEAN   DEFAULT false
```

### 5.4 模型配置表 `provider_keys`
```
id          SERIAL        PK
uid         VARCHAR(32)   FK    → users.id
vendorId    VARCHAR(50)         如 openai/deepseek/anthropic
apiKey      TEXT                真实 API Key（加密存储）
label       VARCHAR(200)        显示名称
model       VARCHAR(100)        默认模型
enabled     BOOLEAN   DEFAULT true
createdAt   TIMESTAMP
updatedAt   TIMESTAMP
UNIQUE(uid, vendorId)
```

---

## 6. 错误码

| HTTP 状态 | error 字段 | 含义 |
|-----------|-----------|------|
| 400 | `Missing required fields` | 缺少必要字段 |
| 401 | `Invalid credentials` | uid/token 无效或过期 |
| 403 | `Device not paired` | 设备未绑定 |
| 403 | `Provider not available` | 该用户不可用此模型 |
| 403 | `Device limit reached` | 设备数达到上限 |
| 429 | `Rate limit exceeded` | 请求频率过高 |
| 500 | `Internal server error` | 服务端错误 |

---

## 7. 安全建议

1. **API Key 加密存储**：`provider_keys.apiKey` 使用 AES-256-GCM 加密，密钥由环境变量注入
2. **HTTPS 强制**：所有通信必须 HTTPS
3. **请求频率限制**：登录接口每分钟最多 10 次，模型代理接口按 token 配额限制
4. **Token 轮换**：支持 `POST /api/auth/refresh-token` 刷新 token（可选，当前 token 永不过期）
5. **日志脱敏**：不记录 API Key 明文到日志
6. **请求签名（可选增强）**：后续可让客户端用 Ed25519 私钥签名请求体，后端用公钥验证，防止 token 泄露后重放攻击

---

## 8. 技术栈建议

| 层 | 推荐 | 备选 |
|----|------|------|
| 语言 | TypeScript (Node.js) | Go / Python |
| 框架 | Hono / Express | Fastify / Gin |
| 数据库 | PostgreSQL | MySQL |
| 缓存 | Redis | Memory |
| 部署 | Cloudflare Workers / Vercel / 自有服务器 | AWS Lambda |
| 密钥管理 | 环境变量 + 加密存储 | Vault / AWS KMS |

## 9. 开发阶段 Mock 实现

后端开发期间，客户端可直接用本地 Host API 的 mock 路由验证：

```typescript
// electron/api/routes/auth.ts 中的 proxyToBackend
// 后端未就绪时，可在请求头中携带 mock 标识，返回模拟数据
```

最小可用的后端只需要三个接口：
1. `POST /api/auth/login` — 查 users 表，验证密码，生成 token
2. `POST /api/auth/providers` — 查 sessions + provider_keys 表，返回模型列表
3. `POST /proxy/:vendorId/chat/completions` — 校验认证，转发到真实模型
