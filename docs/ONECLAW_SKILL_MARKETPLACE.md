# OneClaw 技能市场架构与扩展方案

## 1. 现有架构

```
┌─ Skills 页面 (渲染进程) ─────────────────────────────────────┐
│  src/pages/Skills/index.tsx                                   │
│  src/stores/skills.ts      聚合本地 + 市场技能                  │
└──────────────────────────────────────────────────────────────┘
    │ IPC / Host API
    ▼
┌─ Extension Registry (主进程) ────────────────────────────────┐
│  electron/extensions/registry.ts                              │
│    getMarketplaceProvider() → MarketplaceProviderExtension     │
└──────────────────────────────────────────────────────────────┘
    │
    ▼
┌─ ClawHub Marketplace (内置) ──────────────────────────────────┐
│  electron/extensions/builtin/clawhub-marketplace.ts            │
│    mode: 'clawhub'                                            │
│    search() → ClawHubService.search()                         │
│    install() → ClawHubService.install()                       │
└──────────────────────────────────────────────────────────────┘
    │ HTTP
    ▼
┌─ ClawHub Registry (外部) ─────────────────────────────────────┐
│  https://clawhub.ai                                            │
│    GET  /api/v1/search?q=&limit=                               │
│    GET  /api/v1/skills/:slug                                   │
│    GET  /api/v1/download?slug=&version=                        │
└──────────────────────────────────────────────────────────────┘
```

### 核心接口（`electron/extensions/types.ts`）

```typescript
export interface MarketplaceProviderExtension extends Extension {
  getCapability(): Promise<MarketplaceCapability>;
  search(params: ClawHubSearchParams): Promise<ClawHubSkillResult[]>;
  install(params: ClawHubInstallParams): Promise<void>;
}
```

### 前端聚合（`src/stores/skills.ts`）

技能列表来源于三个方向，前端自动合并：

| 来源 | 获取方式 | 示例 |
|------|---------|------|
| **本地技能** (bundled/managed/workspace/extra) | Host API: `/api/skills/status` | OpenClaw 内置 pdf、xlsx 等 |
| **ClawHub 市场** | Host API: `/api/clawhub/list` | clawhub.ai 上发布的技能 |
| **搜索市场** | Host API: `/api/clawhub/search` | 按关键词搜索 clawhub |

---

## 2. 企业/私有技能市场方案

### 方案 A：依赖注入覆盖 Registry URL（零代码）

如果你的私有 registry 符合 ClawHub API 规范，通过环境变量覆盖地址：

```bash
# .env 或启动脚本
OPENCLAW_CLAWHUB_URL=https://skills.mycompany.com
CLAWHUB_TOKEN=sk-xxxxxxxxxxxxxxxxxxxxxxxx
```

**适用场景**：企业内部搭建了符合 ClawHub API 的私有 registry。

**优点**：零代码，即开即用。
**缺点**：只能指向一个 registry，替换了公开市场。

---

### 方案 B：注册 Extension，支持多市场并行（推荐）

创建一个新的 `MarketplaceProviderExtension`，注册到 Extension Registry，与官方市场并存。

#### 步骤 1：创建企业市场 Extension

`electron/extensions/builtin/enterprise-marketplace.ts`：

```typescript
import type {
  Extension,
  ExtensionContext,
  MarketplaceProviderExtension,
  MarketplaceCapability,
} from '../types';
import type {
  ClawHubSearchParams,
  ClawHubInstallParams,
  ClawHubSkillResult,
} from '../../gateway/clawhub';

const ENTERPRISE_REGISTRY_URL = process.env.ENTERPRISE_SKILLS_URL || 'https://skills.mycompany.com';
const ENTERPRISE_TOKEN = process.env.ENTERPRISE_SKILLS_TOKEN || '';

class EnterpriseMarketplaceExtension implements MarketplaceProviderExtension {
  readonly id = 'builtin/enterprise-marketplace';

  setup(_ctx: ExtensionContext): void {}

  async getCapability(): Promise<MarketplaceCapability> {
    return {
      mode: 'enterprise',      // 标识来源
      canSearch: true,
      canInstall: true,
    };
  }

  async search(params: ClawHubSearchParams): Promise<ClawHubSkillResult[]> {
    const url = new URL('/api/v1/search', ENTERPRISE_REGISTRY_URL);
    url.searchParams.set('q', params.query);
    url.searchParams.set('limit', String(params.limit || 20));

    const headers: Record<string, string> = {};
    if (ENTERPRISE_TOKEN) {
      headers['Authorization'] = `Bearer ${ENTERPRISE_TOKEN}`;
    }

    const res = await fetch(url, { headers });
    if (!res.ok) {
      console.warn(`[enterprise-marketplace] search failed (${res.status})`);
      return [];
    }

    const data = await res.json();
    return (data.results || []).map((item: any) => ({
      slug: item.slug,
      name: item.name,
      description: item.description,
      version: item.version,
      author: item.author,
      source: 'enterprise',
      // ... 映射为标准 ClawHubSkillResult 格式
    }));
  }

  async install(params: ClawHubInstallParams): Promise<void> {
    // 下载 ZIP，校验安全（SHA-256、路径注入防护），解压到 managed 目录
    // 参考 electron/gateway/clawhub.ts 的 installSkillFromClawHub 逻辑
    const downloadUrl = new URL('/api/v1/download', ENTERPRISE_REGISTRY_URL);
    downloadUrl.searchParams.set('slug', params.slug);
    downloadUrl.searchParams.set('version', params.version || 'latest');

    const headers: Record<string, string> = {};
    if (ENTERPRISE_TOKEN) {
      headers['Authorization'] = `Bearer ${ENTERPRISE_TOKEN}`;
    }

    const res = await fetch(downloadUrl, { headers });
    if (!res.ok) throw new Error(`Enterprise registry download failed (${res.status})`);

    // 复用现有安装逻辑
    const { installSkillFromArchive } = await import('../../gateway/clawhub');
    await installSkillFromArchive(params.slug, Buffer.from(await res.arrayBuffer()));
  }
}

export function createEnterpriseMarketplaceExtension(): Extension {
  return new EnterpriseMarketplaceExtension();
}
```

#### 步骤 2：注册到系统

`electron/extensions/builtin/index.ts`：

```typescript
import { createEnterpriseMarketplaceExtension } from './enterprise-marketplace';

export function registerBuiltinExtensions(registry: ExtensionRegistry): void {
  // 现有
  registerBuiltinExtension('builtin/clawhub-marketplace', createClawHubMarketplaceExtension);

  // 新增企业市场
  if (process.env.ENTERPRISE_SKILLS_URL) {
    registerBuiltinExtension('builtin/enterprise-marketplace', createEnterpriseMarketplaceExtension);
  }
}
```

#### 步骤 3：允许注册多个 Marketplace Provider

`electron/extensions/registry.ts` 当前只有一个 provider，需要改为支持多个：

```typescript
// 之前：只返回一个
getMarketplaceProvider(): MarketplaceProviderExtension | undefined

// 改为：返回所有
getMarketplaceProviders(): MarketplaceProviderExtension[] {
  return this.getAll().filter(isMarketplaceProviderExtension);
}
```

#### 步骤 4：前端支持多市场

`src/stores/skills.ts` 中改为遍历所有 provider 并聚合结果，带上 `source` 标记。

`src/pages/Skills/index.tsx` 中根据 `source` 展示不同的标签：`ClawHub` / `企业`。

---

### 方案 C：文件共享目录（零后端，技能放文件服务器）

适合企业内部无 registry 服务的场景，直接把技能文件放在共享目录。

#### 技能目录结构

```
/mnt/company-skills/
├── code-review/
│   ├── SKILL.md          ← AI 读取的能力描述
│   ├── references/
│   └── scripts/
├── compliance-check/
│   ├── SKILL.md
│   └── scripts/
└── deploy-helper/
    ├── SKILL.md
    └── scripts/
```

#### SKILL.md 格式

```markdown
---
name: Code Review
description: 企业内部代码审查助手，支持 Java / Go 项目规范
command-dispatch: tool
command-tool: company_code_review
user-invocable: true
---

# Code Review

## 使用方法
...
```

#### 配置方式

`~/.openclaw/openclaw.json` 或 `~/.openclaw/easyclaw.json`：

```json
{
  "skills": {
    "extraDirs": [
      "/mnt/company-skills",
      "//fileserver/skills",
      "/Users/shared/project-skills"
    ]
  }
}
```

或通过环境变量：
```bash
OPENCLAW_SKILLS_EXTRA_DIRS="/mnt/company-skills://fileserver/skills"
```

**优点**：零代码，部署即用。  
**缺点**：无版本管理，无搜索 UI（需要通过 Finder/命令行操作）。

---

## 3. 私有 Registry API 规范

如果你自建 registry（方案 A 或 B），需要实现以下接口：

### `GET /api/v1/search`

```
查询参数：q=关键词&limit=20

响应：
{
  "results": [
    {
      "slug": "code-review",
      "name": "Code Review",
      "description": "企业内部代码审查助手",
      "version": "1.2.0",
      "author": { "name": "DevOps Team" },
      "tags": ["code", "review"],
      "downloads": 1234,
      "createdAt": "2026-01-01T00:00:00Z"
    }
  ]
}
```

### `GET /api/v1/skills/:slug`

```
响应：
{
  "slug": "code-review",
  "name": "Code Review",
  "version": "1.2.0",
  "description": "...",
  "author": { "name": "DevOps Team", "email": "..." },
  "versions": {
    "latest": "1.2.0",
    "list": ["1.0.0", "1.1.0", "1.2.0"]
  },
  "sha256": "abc123...",
  "minGatewayVersion": "2026.1.0"
}
```

### `GET /api/v1/download`

```
查询参数：slug=code-review&version=1.2.0

响应：
  Content-Type: application/zip
  Body: 技能 ZIP 包
```

### 认证（Bearer Token）

```
Authorization: Bearer <token>
```

Token 来源优先级：
1. 请求参数直接传入 `params.token`
2. 环境变量 `ENTERPRISE_SKILLS_TOKEN`
3. 本地配置文件 `~/.config/clawhub/config.json`

---

## 4. 安全考虑

| 风险 | 防护 |
|------|------|
| ZIP 炸弹 | 限制解压后总大小 ≤ 256MB，文件数 ≤ 50000 |
| 路径注入 | 拒绝包含 `../`、`/` 开头、反斜杠的路径 |
| 恶意脚本 | 安装前 SHA-256 校验；运行时在沙箱中执行 |
| Token 泄露 | Token 仅通过环境变量注入，不写入代码仓库 |
| HTTPS | 私有 registry 必须启用 HTTPS |

---

## 5. EasyClaw 对比参考

| 功能 | EasyClaw 实现 | OneClaw 对应实现 |
|------|-------------|----------------|
| 公开市场 | clawhub.ai | clawhub.ai（通过 ClawHub Marketplace Extension）|
| 私有市场 | 环境变量 `OPENCLAW_CLAWHUB_URL` | 同（方案 A）/ Enterprise Extension（方案 B）|
| 多市场 | 单一 clawhub 实例 | 可注册多个 `MarketplaceProviderExtension` |
| 技能目录 | `~/.easyclaw/skills/` | `~/.openclaw/skills/` |
| 额外目录 | `skills.extraDirs` 配置 | 同（方案 C）|
| Token 存储 | 环境变量 / `~/.config/clawhub/config.json` | 环境变量 / Electron secureStorage |
| 安装安全 | SHA-256 + 路径验证 + 大小限制 | 复用同逻辑 |

---

## 6. 推荐实施路线

| 阶段 | 方案 | 说明 |
|------|------|------|
| **P0** | 方案 C：extraDirs | 零代码，立即可用，适合内部共享技能 |
| **P1** | 方案 B：Enterprise Extension | 自建 registry，多市场并行，权限控制 |
| **P2** | 前端多市场 UI | Tab 切换、来源标签、搜索聚合 |
| **P3** | 方案 A：环境变量覆盖 | 白标部署，替换公开市场 |
