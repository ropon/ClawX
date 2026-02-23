# ClawX 技术架构与实现方案

> 本文档详细描述各 Phase 的技术实现方案，包括文件结构、数据模型、API 设计、组件设计等。
> 所有新功能开发前必须在此文档中完成技术设计并评审。

---

## 目录

- [1. 现有架构概览](#1-现有架构概览)
- [2. Phase 1: 多 Agent 角色管理](#2-phase-1-多-agent-角色管理)
- [3. Phase 2: 桌面助手核心能力](#3-phase-2-桌面助手核心能力)
- [4. Phase 3: 知识库 & RAG](#4-phase-3-知识库--rag)
- [5. Phase 4: Agent 协作 & 高级自动化](#5-phase-4-agent-协作--高级自动化)
- [6. 跨 Phase 共享设计](#6-跨-phase-共享设计)
- [7. 抽象层设计（Tauri 2 迁移准备）](#7-抽象层设计tauri-2-迁移准备)

---

## 1. 现有架构概览

> v0.2.0 — 纯 Tauri 原生架构（零 Node.js 进程）

```
┌──────────────────────────────────────────────────────┐
│                    ClawX Desktop v0.2.0                │
├──────────────────────┬───────────────────────────────┤
│  Tauri Rust Shell    │  React Renderer (Vite WebView) │
│  (src-tauri/)        │  (src/)                        │
│                      │                                │
│  ┌────────────────┐  │  ┌──────────┐ ┌─────────────┐ │
│  │ 123 #[tauri::   │  │  │ Zustand  │ │ React Router│ │
│  │   command] fns  │◄─┤  │ Stores   │ │ Pages       │ │
│  └───────┬────────┘  │  └──────────┘ └─────────────┘ │
│          │           │  ┌──────────┐ ┌─────────────┐ │
│  ┌───────┴────────┐  │  │ shadcn   │ │ i18next     │ │
│  │ GatewayClient  │  │  │ UI       │ │ i18n        │ │
│  │ (Actor WS)     │  │  └──────────┘ └─────────────┘ │
│  └───────┬────────┘  │                                │
│          │ WS        │  ┌──────────────────────────┐  │
│  ┌───────┴────────┐  │  │ bridge.ts                │  │
│  │ app.emit()     │──┤  │  invoke → Tauri command  │  │
│  │ (9 事件通道)    │  │  │  on → Tauri listen       │  │
│  └────────────────┘  │  └──────────────────────────┘  │
│                      │                                │
├──────────────────────┴───────────────────────────────┤
│              OpenClaw Gateway (port 18789)             │
│  ┌──────────┬──────────┬──────────┬────────────────┐  │
│  │ Chat     │ Channels │ Skills   │ Cron / Agents  │  │
│  └──────────┴──────────┴──────────┴────────────────┘  │
└──────────────────────────────────────────────────────┘
```

### 进程模型

| 进程 | 说明 |
|---|---|
| Tauri 主进程 | Rust 后端 + 系统 WebView，处理全部 123 个命令 + 9 个推送事件 |
| OpenClaw Gateway | Python/Node.js AI Agent 运行时，由 Tauri 通过 `tauri-plugin-shell` 启动 |

### Gateway 连接认证

ClawX 使用 Ed25519 设备身份连接 Gateway：

```
WS Upgrade → Gateway 发送 connect.challenge (nonce)
           → ClawX 构建签名 payload: v2|{deviceId}|{clientId}|...
           → ClawX 用 Ed25519 私钥签名 payload
           → ClawX 发送 connect frame (含 device 对象)
           → Gateway 验证签名 + 自动配对本地设备
           → Gateway 返回 hello-ok
```

| 字段 | 格式 | 存储位置 |
|---|---|---|
| 密钥对 | Ed25519 (32 byte private + 32 byte public) | `{data_dir}/clawx-device-identity.json` (0600) |
| deviceId | SHA-256(raw_public_key).hex (64 chars) | 同上 |
| publicKey | base64url(raw_32_bytes) (43 chars) | connect frame `device.publicKey` |
| signature | base64url(ed25519_sign(payload)) (86 chars) | connect frame `device.signature` |

实现文件: `src-tauri/src/device_identity.rs`, `src-tauri/src/gateway/protocol.rs`, `src-tauri/src/gateway/client.rs`

### 技术约定

| 约定 | 说明 |
|---|---|
| Store 模式 | `create<State>((set, get) => ({...}))`, 通过 `invoke()` from `@/lib/bridge` |
| IPC 命名 | `namespace:camelCaseAction`，如 `gateway:rpc`, `provider:save` |
| Rust 命令命名 | `namespace_camelCaseAction`，如 `gateway_rpc`, `provider_save` |
| Bridge 路由 | `NATIVE_COMMANDS`（单对象参数）vs 其余命令（`{ _args: [...] }` 包装）|
| 事件系统 | Rust `app.emit()` → 前端 `listen()` via `bridge.on()` + `EVENT_MAP` 映射 |
| 页面结构 | `src/pages/Name/index.tsx`，使用 `useTranslation('namespace')` |
| 组件库 | `src/components/ui/` (shadcn), `src/components/common/`, `src/components/layout/` |
| 路径别名 | `@/` → `src/` |
| 类型定义 | `src/types/` 目录下按模块分文件 |
| 持久化 | Rust JSON 文件存储 (`JsonStore<T>`) + `tauri-plugin-store` + rusqlite |
| 错误处理 | Tauri 命令返回 `Result<Value, String>` |

---

## 2. Phase 1: 多 Agent 角色管理

### 2.1 数据模型

```typescript
// src/types/agent.ts

export interface AgentConfig {
  id: string;                    // UUID
  name: string;                  // 显示名称
  avatar: string;                // 头像 URL 或 emoji
  description: string;           // 简短描述
  systemPrompt: string;          // System Prompt
  providerId: string;            // 关联的 AI Provider ID
  model: string;                 // 模型标识
  temperature: number;           // 0-2
  maxTokens: number;             // 最大输出 Token
  skillIds: string[];            // 启用的 Skill ID 列表
  knowledgeBaseIds: string[];    // 关联的知识库 ID（Phase 3）
  channelBindings: string[];     // 绑定的 Channel ID
  isDefault: boolean;            // 是否为默认 Agent
  createdAt: number;             // 创建时间戳
  updatedAt: number;             // 更新时间戳
}

export interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  systemPrompt: string;
  suggestedModel: string;
  category: 'general' | 'coding' | 'writing' | 'translation' | 'analysis' | 'custom';
}
```

### 2.2 新增文件清单

```
src-tauri/src/
  storage/agents.rs            # Agent 持久化 (JSON 文件)
  commands/agents_cmd.rs       # 9 个 Agent Tauri 命令

src/
  types/
    agent.ts                   # Agent 类型定义
  stores/
    agents.ts                  # Agent Zustand store
  pages/
    Agents/
      index.tsx                # Agent 列表页
      AgentCard.tsx            # Agent 卡片组件
      AgentEditor.tsx          # Agent 编辑器（抽屉/模态框）
      AgentTemplates.tsx       # 预设模板选择
  i18n/locales/
    en/agents.json
    zh/agents.json
    ja/agents.json
```

### 2.3 Store 设计

```typescript
// src/stores/agents.ts

interface AgentsState {
  agents: AgentConfig[];
  activeAgentId: string | null;
  loading: boolean;
  error: string | null;

  // CRUD
  fetchAgents: () => Promise<void>;
  createAgent: (agent: Omit<AgentConfig, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateAgent: (id: string, updates: Partial<AgentConfig>) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  cloneAgent: (id: string) => Promise<void>;

  // 切换
  setActiveAgent: (id: string) => void;
  getActiveAgent: () => AgentConfig | null;

  // 导入导出
  exportAgent: (id: string) => Promise<string>;    // JSON string
  importAgent: (json: string) => Promise<void>;
}
```

### 2.4 IPC 通道设计

| 通道 | 方向 | 参数 | 返回 |
|---|---|---|---|
| `agent:list` | invoke | - | `AgentConfig[]` |
| `agent:get` | invoke | `id: string` | `AgentConfig` |
| `agent:create` | invoke | `Partial<AgentConfig>` | `AgentConfig` |
| `agent:update` | invoke | `id: string, updates: Partial<AgentConfig>` | `AgentConfig` |
| `agent:delete` | invoke | `id: string` | `void` |
| `agent:export` | invoke | `id: string` | `string` (JSON) |
| `agent:import` | invoke | `json: string` | `AgentConfig` |

### 2.5 UI 设计要点

**Sidebar 新增入口**：在 Chat 和 Cron 之间添加 `Agents` 导航项，图标使用 `Bot` (lucide-react)。

**Agent 列表页**：
- 顶部搜索栏 + 「新建 Agent」按钮
- 网格布局展示 Agent 卡片
- 卡片显示：Avatar, Name, Description, Model, 技能数量
- 卡片操作：编辑、克隆、删除、设为默认

**Agent 编辑器**（右侧抽屉）：
- 基本信息：名称、头像（emoji 选择器）、描述
- AI 配置：Provider 选择、模型选择、Temperature 滑块、Max Tokens
- System Prompt：多行文本编辑器，支持变量模板
- 技能：从已安装 Skill 中勾选启用
- 通道绑定：勾选要绑定的 Channel

**Chat 页面修改**：
- 顶部工具栏增加 Agent 切换下拉
- 切换 Agent 时保留当前会话，但后续消息使用新 Agent 配置
- Agent 头像显示在 AI 消息气泡旁

### 2.6 与 OpenClaw Gateway 的集成

Agent 的 system prompt、model、temperature 等参数通过 `chat.send` RPC 调用时传入：

```typescript
// 扩展 chat.send 的 params
{
  message: string,
  sessionId: string,
  agentConfig: {
    systemPrompt: string,
    model: string,
    temperature: number,
    maxTokens: number,
    skills: string[],
  }
}
```

如果 Gateway 不支持 per-request 配置覆盖，则在发送前通过 `openclaw.json` 配置文件动态写入，然后触发 Gateway 热重载。

---

## 3. Phase 2: 桌面助手核心能力

### 3.1 Spotlight 窗口架构

```
┌─────────────────────────────────────────┐
│           Spotlight Window               │
│  ┌─────────────────────────────────────┐ │
│  │  搜索/输入框  [Agent: xxx]  [⌘]    │ │
│  ├─────────────────────────────────────┤ │
│  │  剪贴板上下文提示                    │ │
│  ├─────────────────────────────────────┤ │
│  │  快捷指令列表 / AI 回复区域          │ │
│  │  ┌───────────────────────────────┐  │ │
│  │  │  /translate  翻译选中文本     │  │ │
│  │  │  /summarize  总结内容         │  │ │
│  │  │  /explain    解释代码         │  │ │
│  │  │  /rewrite    重写文本         │  │ │
│  │  └───────────────────────────────┘  │ │
│  └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### 3.2 新增文件清单

```
src-tauri/src/
  lib.rs                       # Spotlight 窗口 toggle/hide 命令
  file_search.rs               # 本地文件检索 (mdfind + walkdir)
  file_staging.rs              # 屏幕截图文件处理

src/
  pages/
    Spotlight/
      index.tsx                # Spotlight 主页面
      SpotlightInput.tsx       # 输入框组件
      SpotlightResults.tsx     # 结果展示区
      QuickCommands.tsx        # 快捷指令列表
      ClipboardPreview.tsx     # 剪贴板内容预览
  stores/
    spotlight.ts               # Spotlight 状态管理
  types/
    spotlight.ts               # Spotlight 类型定义
  i18n/locales/
    en/spotlight.json
    zh/spotlight.json
    ja/spotlight.json
```

### 3.3 数据模型

```typescript
// src/types/spotlight.ts

export interface QuickCommand {
  id: string;
  name: string;                    // 显示名称
  command: string;                 // 触发命令 (如 /translate)
  description: string;
  icon: string;                    // lucide icon name
  agentId?: string;                // 可选绑定特定 Agent
  promptTemplate: string;          // Prompt 模板，{{input}} 为占位符
  inputSource: 'clipboard' | 'selection' | 'manual' | 'screenshot';
  outputAction: 'clipboard' | 'notification' | 'replace' | 'overlay';
  shortcut?: string;               // 可选的独立快捷键
}

export interface ClipboardContext {
  content: string;
  type: 'text' | 'image' | 'url' | 'code' | 'file-path';
  detectedLanguage?: string;       // 检测到的编程语言或自然语言
  timestamp: number;
}

export interface SpotlightState {
  visible: boolean;
  query: string;
  mode: 'search' | 'chat' | 'command';
  clipboardContext: ClipboardContext | null;
  activeAgentId: string | null;
  results: SpotlightResult[];
  loading: boolean;
}

export interface SpotlightResult {
  id: string;
  type: 'command' | 'ai-response' | 'file' | 'action';
  title: string;
  description: string;
  icon: string;
  action: () => void;
}
```

### 3.4 Spotlight 窗口实现

Spotlight 窗口通过 `tauri.conf.json` 配置为独立 WebView 窗口，使用 `/spotlight` 路由：

```json
{
  "label": "spotlight",
  "title": "Spotlight",
  "url": "/spotlight",
  "width": 680,
  "height": 480,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "visible": false,
  "skipTaskbar": true
}
```

Rust 命令 `spotlight_toggle` / `spotlight_hide` 控制窗口显示/隐藏。

### 3.5 全局快捷键设计

通过 `tauri-plugin-global-shortcut` 注册全局快捷键。默认快捷键 `CommandOrControl+Shift+Space` 切换 Spotlight 窗口。快捷键配置存储在 `tauri-plugin-store`（`clawx-settings.json`）。

### 3.6 剪贴板智能处理

通过 `tauri-plugin-clipboard-manager` 读写剪贴板。Spotlight 激活时读取当前内容，前端 `src/utils/clipboard-detect.ts` 检测内容类型（URL/代码/文本/图片）。

### 3.7 截图模块

使用系统原生截图工具：macOS `screencapture -i`、Linux `gnome-screenshot`/`scrot`。截图文件通过 Rust `file_staging.rs` 处理。

### 3.8 IPC 通道设计

| 通道 | 方向 | 参数 | 返回 |
|---|---|---|---|
| `spotlight:toggle` | invoke | - | `void` |
| `spotlight:show` | invoke | `{ query?: string }` | `void` |
| `spotlight:hide` | invoke | - | `void` |
| `spotlight:resize` | invoke | `{ height: number }` | `void` |
| `spotlight:shown` | on | - | - |
| `spotlight:hidden` | on | - | - |
| `clipboard:read` | invoke | - | `ClipboardContext` |
| `clipboard:write` | invoke | `{ text: string }` | `void` |
| `screenshot:capture` | invoke | - | `{ base64: string, path: string }` |
| `screenshot:region` | invoke | - | `{ base64: string, path: string }` |
| `shortcut:list` | invoke | - | `ShortcutConfig[]` |
| `shortcut:update` | invoke | `ShortcutConfig` | `void` |
| `shortcut:reset` | invoke | - | `void` |
| `command:list` | invoke | - | `QuickCommand[]` |
| `command:execute` | invoke | `{ commandId: string, input: string }` | `string` |
| `local-search:query` | invoke | `{ query: string, path?: string }` | `SearchResult[]` |

### 3.9 Spotlight UI 交互流程

```
用户按下 ⌥ Space
    ↓
Spotlight 窗口显示（居中置顶）
    ↓
自动读取剪贴板 → 检测内容类型 → 底部提示栏
    ↓
用户可以:
├── 直接输入文字 → 发送给当前 Agent → 流式显示回复
├── 输入 / 开头 → 显示匹配的快捷指令列表
├── 选择快捷指令 → 以剪贴板/手动输入为 input 执行
├── 按 ⌥ S → 进入截图模式 → 截图发送给 AI
└── 按 Esc → 隐藏窗口
    ↓
AI 回复完成 → 用户可:
├── ⌘ C 复制回复
├── Enter 继续对话
└── Esc 关闭
```

---

## 4. Phase 3: 知识库 & RAG

### 4.1 数据模型

```typescript
// src/types/knowledge.ts

export interface KnowledgeBase {
  id: string;
  name: string;
  description: string;
  documentCount: number;
  totalChunks: number;
  totalSize: number;              // bytes
  embeddingModel: string;
  createdAt: number;
  updatedAt: number;
}

export interface KnowledgeDocument {
  id: string;
  knowledgeBaseId: string;
  fileName: string;
  filePath: string;
  fileType: 'pdf' | 'txt' | 'md' | 'docx' | 'url';
  fileSize: number;
  chunkCount: number;
  status: 'pending' | 'processing' | 'ready' | 'error';
  error?: string;
  createdAt: number;
}

export interface KnowledgeChunk {
  id: string;
  documentId: string;
  content: string;
  embedding: number[];            // 向量
  metadata: {
    page?: number;
    section?: string;
    lineStart?: number;
    lineEnd?: number;
  };
}

export interface RAGConfig {
  topK: number;                   // 检索 Top-K，默认 5
  scoreThreshold: number;         // 相似度阈值，默认 0.7
  maxContextTokens: number;       // 注入上下文的最大 Token 数
  includeSource: boolean;         // 是否显示来源引用
}
```

### 4.2 RAG 流程

```
用户发送消息
    ↓
检查当前 Agent 是否关联知识库
    ↓ (是)
将用户消息向量化 (Embedding API)
    ↓
在关联知识库中进行相似度搜索 (Top-K)
    ↓
筛选 score > threshold 的结果
    ↓
构造增强 Prompt:
┌──────────────────────────────────┐
│ [System Prompt]                  │
│                                  │
│ 以下是相关参考资料：               │
│ ---                              │
│ [来源: doc1.pdf, 第3页]           │
│ {chunk_content_1}                │
│ ---                              │
│ [来源: notes.md, 第12-15行]      │
│ {chunk_content_2}                │
│ ---                              │
│                                  │
│ 请基于上述资料回答用户问题。        │
│ 如果资料中没有相关信息，请如实告知。 │
└──────────────────────────────────┘
    ↓
发送给 LLM → 流式回复
    ↓
回复中标注引用来源（可点击展开原文）
```

### 4.3 技术选型

| 需求 | 方案 | 备注 |
|---|---|---|
| 向量存储 | rusqlite (bundled SQLite) | f32 BLOB + 纯 Rust 余弦相似度搜索 |
| Embedding | OpenAI 兼容 /embeddings 端点 | reqwest，batch size 32 |
| 文本分块 | Rust 递归字符分割 | 512 tokens/chunk，128 overlap |
| PDF 解析 | `lopdf` (Rust) | |
| DOCX 解析 | `zip` + `quick-xml` (Rust) | |
| 网页抓取 | `reqwest` + `scraper` (Rust) | CSS 选择器提取正文 |
| 文件夹监听 | `notify` crate (Rust) | 500ms debounce |

### 4.4 新增文件清单

```
src-tauri/src/
  storage/knowledge.rs         # 知识库元数据管理 (JSON 文件)
  database/vector_db.rs        # SQLite 向量数据库
  database/mod.rs              # DbManager (knowledge.db + runs.db)
  document_parser.rs           # PDF/DOCX/TXT 文档解析
  text_chunker.rs              # 文本分块
  embedding.rs                 # Embedding API 客户端
  embedding_api.rs             # 模型发现 + 维度检测
  rag_engine.rs                # RAG 检索引擎
  web_scraper.rs               # 网页抓取
  knowledge_pipeline.rs        # 文档处理管线
  folder_watcher.rs            # 文件夹监听
  commands/knowledge_cmd.rs    # 15 个知识库命令

src/
  types/
    knowledge.ts
  stores/
    knowledge.ts
  pages/
    Knowledge/
      index.tsx                # 知识库列表
      KnowledgeDetail.tsx      # 知识库详情（文档列表）
      DocumentUpload.tsx       # 文档上传组件
      KnowledgeSearch.tsx      # 知识库搜索面板
  i18n/locales/
    en/knowledge.json
    zh/knowledge.json
    ja/knowledge.json
```

---

## 5. Phase 4: Agent 协作 & 高级自动化

### 5.1 任务链数据模型

```typescript
// src/types/workflow.ts

export interface Workflow {
  id: string;
  name: string;
  description: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  triggers: WorkflowTrigger[];
  createdAt: number;
  updatedAt: number;
}

export interface WorkflowNode {
  id: string;
  type: 'agent' | 'condition' | 'merge' | 'input' | 'output';
  position: { x: number; y: number };
  data: {
    agentId?: string;            // type=agent 时
    condition?: string;          // type=condition 时，JS 表达式
    inputMapping?: Record<string, string>;
    outputMapping?: Record<string, string>;
  };
}

export interface WorkflowEdge {
  id: string;
  source: string;               // 源节点 ID
  target: string;               // 目标节点 ID
  sourceHandle?: string;        // condition 节点的 true/false 出口
  label?: string;
}

export interface WorkflowTrigger {
  type: 'manual' | 'cron' | 'file-change' | 'clipboard' | 'shortcut' | 'channel-message';
  config: Record<string, unknown>;
}

export interface WorkflowExecution {
  id: string;
  workflowId: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  startedAt: number;
  completedAt?: number;
  nodeResults: Record<string, {
    status: 'pending' | 'running' | 'completed' | 'failed';
    input: unknown;
    output: unknown;
    startedAt: number;
    completedAt?: number;
    error?: string;
  }>;
}
```

### 5.2 可视化编排

使用 `@xyflow/react` (React Flow) 实现拖拽式流程编排：

```
新增文件：
src/
  pages/
    Workflows/
      index.tsx               # 工作流列表
      WorkflowEditor.tsx      # 可视化编辑器（React Flow）
      nodes/
        AgentNode.tsx          # Agent 执行节点
        ConditionNode.tsx      # 条件分支节点
        MergeNode.tsx          # 合并节点
        InputNode.tsx          # 输入节点
        OutputNode.tsx         # 输出节点
      WorkflowRunner.tsx      # 执行面板（实时状态）
  stores/
    workflows.ts
  types/
    workflow.ts
```

---

## 6. 跨 Phase 共享设计

### 6.1 通知系统

通过 `tauri-plugin-notification` 实现系统原生通知。AI 回复完成时（窗口未聚焦）自动推送桌面通知。

### 6.2 统一的 Agent 执行层

所有 Agent 调用通过 Rust `GatewayClient` WS Actor 发送 RPC 到 OpenClaw Gateway：

```rust
// src-tauri/src/gateway/client.rs
gateway_client.rpc("chat.send", Some(json!({
    "message": input,
    "sessionKey": session_key,
    "agentConfig": { ... },
})), 30000).await
```

### 6.3 Sidebar 导航扩展路线

```
Phase 1 新增: 🤖 Agents    → /agents
Phase 2 新增: (无新导航，Spotlight 是独立窗口)
Phase 3 新增: 📚 Knowledge → /knowledge
Phase 4 新增: ⚡ Workflows → /workflows
```

最终 Sidebar 顺序：
```
💬 Chat
🤖 Agents
📚 Knowledge
⚡ Workflows
⏰ Cron
🧩 Skills
📡 Channels
📊 Dashboard
⚙️ Settings
```

---

## 7. Tauri 原生架构（v0.2.0 最终架构）

> 迁移已完成。Electron 和 Node.js Sidecar 已完全移除。

```
┌──────────────────────────────────────────────────────────┐
│  Tauri Rust Shell (src-tauri/)                            │
│  ├── 123 native commands                                  │
│  │   ├── window: minimize, maximize, close, isMaximized   │
│  │   ├── clipboard: read, write                           │
│  │   ├── spotlight: toggle, hide                          │
│  │   ├── shortcut: get, update                            │
│  │   ├── app: version, platform                           │
│  │   ├── agent: list,get,create,update,delete,            │
│  │   │   getActive,setActive,export,import                │
│  │   ├── knowledge: list,get,create,update,delete,        │
│  │   │   listDocuments,getDocument,createDocument,         │
│  │   │   updateDocument,deleteDocument,refreshStats,       │
│  │   │   search,rag,addDocument,addUrl,reprocessDocument,  │
│  │   │   setWatchFolder,removeDocument,getWatchStatus,     │
│  │   │   getEmbeddingOptions,detectDimension               │
│  │   ├── workflow: list,get,create,update,delete,         │
│  │   │   duplicate,export,import,getRuns,getRun,deleteRun, │
│  │   │   clearRuns,execute,cancel,registerTriggers,        │
│  │   │   unregisterTriggers,getTemplates,importTemplate    │
│  │   ├── gateway: status,isConnected,rpc,                 │
│  │   │   getControlUiUrl,health,start,stop,restart         │
│  │   ├── cron: list,create,update,delete,toggle,trigger   │
│  │   ├── channel: saveConfig,getConfig,getFormValues,     │
│  │   │   deleteConfig,listConfigured,setEnabled,           │
│  │   │   validate,validateCredentials                      │
│  │   ├── chat: sendWithMedia                              │
│  │   ├── clawhub: search,install,uninstall,list,          │
│  │   │   openSkillReadme                                   │
│  │   ├── provider: save,list,get,delete,validate,         │
│  │   │   getApiKey,deleteApiKey,getAll                     │
│  │   ├── skill: list,install,uninstall                    │
│  │   ├── openclaw: getConfig,getDefaultModel,             │
│  │   │   getAuthProfiles                                   │
│  │   ├── settings: get,set                                │
│  │   ├── update: check,install                            │
│  │   ├── file: stage,stageMultiple,getInfo                │
│  │   ├── media: resizeImage                               │
│  │   ├── filesearch: search                               │
│  │   ├── uv: check,install                                │
│  │   └── log: write, getRecent                            │
│  │                                                        │
│  ├── Gateway WS layer:                                    │
│  │   ├── gateway/client.rs — Actor WS client              │
│  │   │   + 推送事件转发 (chat/channel.status/agent)        │
│  │   │   + 断连自动重连 (指数退避 3s-30s, 最多10次)         │
│  │   │   + 桌面通知 (tauri-plugin-notification, 窗口失焦时) │
│  │   ├── gateway/protocol.rs — OpenClaw Protocol v3 frames │
│  │   ├── settings_store.rs — Read/Write gatewayToken       │
│  │   └── channel_config.rs — ~/.openclaw/openclaw.json     │
│  │                                                        │
│  ├── Database layer:                                      │
│  │   ├── database/mod.rs — DbManager (knowledge.db+runs.db)│
│  │   ├── database/vector_db.rs — Cosine search (no ext)   │
│  │   ├── database/workflow_runs.rs — Run CRUD              │
│  │   ├── embedding.rs — OpenAI /embeddings API client     │
│  │   ├── rag_engine.rs — RAG retrieval engine             │
│  │   └── secure_storage.rs — Provider CRUD + API key store │
│  │                                                        │
│  ├── Knowledge Pipeline layer:                            │
│  │   ├── document_parser.rs — PDF(lopdf)/DOCX(zip+xml)/TXT│
│  │   ├── web_scraper.rs — URL fetch + HTML text extract   │
│  │   ├── embedding_api.rs — Provider embedding API client │
│  │   ├── knowledge_pipeline.rs — parse→chunk→embed→store  │
│  │   └── folder_watcher.rs — notify file change monitor   │
│  │                                                        │
│  ├── Workflow Engine layer:                               │
│  │   ├── workflow_engine.rs — DAG executor (Kahn topo)    │
│  │   └── workflow_triggers.rs — cron/file/clipboard/key   │
│  │                                                        │
│  ├── Gateway Lifecycle layer:                             │
│  │   ├── gateway_lifecycle.rs — Process spawn/stop/restart│
│  │   └── clawhub.rs — ClawHub CLI subprocess wrapper      │
│  │                                                        │
│  ├── Internal modules:                                    │
│  │   ├── providers.rs — Provider Registry (11 providers)  │
│  │   ├── text_chunker.rs — Text splitting algorithm       │
│  │   ├── openclaw_auth.rs — Auth profiles + default model │
│  │   ├── openclaw_paths.rs — Path resolution              │
│  │   ├── provider_validate.rs — API key HTTP validation   │
│  │   ├── file_staging.rs — File staging + image resize    │
│  │   ├── file_search.rs — mdfind + content read           │
│  │   └── channel_validate.rs — Config + credential checks │
│  │                                                        │
│  ├── 8 plugins: global-shortcut, clipboard, notification, │
│  │   dialog, store, process, shell, updater               │
│  └── System tray (show/hide/spotlight/quit)               │
├──────────────────────────────────────────────────────────┤
│  Renderer (React + Vite + WebView)                        │
│  └── src/lib/bridge.ts                                    │
│      ├── invoke() → Tauri invoke                          │
│      │   NATIVE_COMMANDS → 单对象参数                      │
│      │   其余命令 → { _args: [...] } 包装                  │
│      ├── on() → Tauri listen (EVENT_MAP 映射)             │
│      │   gateway_chatMessage, gateway_channelStatus,       │
│      │   gateway_notification, gateway_statusChanged,      │
│      │   gateway_error, knowledge_documentProgress,        │
│      │   workflow_stepProgress                              │
│      ├── once(), openExternal(), getPlatform(), getIsDev() │
│      └── 零 SSE，零 Sidecar，零 Electron                   │
├──────────────────────────────────────────────────────────┤
│              OpenClaw Gateway (port 18789)                 │
│  ┌──────────┬──────────┬──────────┬────────────────────┐  │
│  │ Chat     │ Channels │ Skills   │ Cron / Agents      │  │
│  └──────────┴──────────┴──────────┴────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### 7.1 Bridge 层（最终版）

```typescript
// src/lib/bridge.ts — 纯 Tauri 桥接层

export async function invoke<T>(channel: string, ...args: unknown[]): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  const command = channel.replace(':', '_').replace(/-/g, '_');
  if (NATIVE_COMMANDS.has(command)) {
    return tauriInvoke<T>(command, args[0] as Record<string, unknown>);
  }
  return tauriInvoke<T>(command, { _args: args });
}

export function on(channel: string, handler: IpcHandler): () => void {
  // 所有事件通过 Tauri listen + EVENT_MAP 映射
  const tauriEvent = EVENT_MAP[channel] ?? channel.replace(/:/g, '_');
  // ... Tauri listen implementation
}
```

### 7.2 事件系统

| Rust 事件名 | 前端通道名 | 来源 |
|---|---|---|
| `gateway_statusChanged` | `gateway:status-changed` | `gateway_lifecycle.rs` + `gateway/client.rs` |
| `gateway_chatMessage` | `gateway:chat-message` | `gateway/client.rs` (WS 推送) |
| `gateway_notification` | `gateway:notification` | `gateway/client.rs` (WS 推送) |
| `gateway_channelStatus` | `gateway:channel-status` | `gateway/client.rs` (WS 推送) |
| `gateway_error` | `gateway:error` | `gateway/client.rs` (断连) |
| `knowledge_documentProgress` | `knowledge:documentProgress` | `knowledge_pipeline.rs` |
| `workflow_stepProgress` | `workflow:stepProgress` | `workflow_engine.rs` |

---

## 附录 A: Rust 依赖清单

| Crate | 用途 |
|---|---|
| `tauri` + 8 plugins | 桌面框架 + 系统集成 |
| `tokio-tungstenite` | Gateway WebSocket 客户端 |
| `reqwest` | HTTP 请求（Embedding API、Provider 验证） |
| `rusqlite` (bundled) | SQLite 向量数据库 + 工作流执行日志 |
| `lopdf` | PDF 文本提取 |
| `zip` + `quick-xml` | DOCX 解析 |
| `scraper` | HTML DOM 解析 / 网页抓取 |
| `notify` | 文件系统监听 |
| `cron` | Cron 表达式解析 |
| `image` | 图片 resize |
| `uuid`, `chrono`, `regex` | 通用工具 |

### 前端依赖

| 包 | 用途 |
|---|---|
| `@tauri-apps/api` | Tauri IPC + 事件 |
| `@tauri-apps/plugin-opener` | 外部链接打开 |
| `@tauri-apps/plugin-updater` | 应用自动更新 |
| `@tauri-apps/plugin-dialog` | 文件选择对话框 |
| `@xyflow/react` | 可视化流程编排 |
| `react` + `react-dom` + `react-router-dom` | UI 框架 |
| `zustand` | 状态管理 |
| `i18next` + `react-i18next` | 国际化 |
| `framer-motion` | 动画 |
| shadcn/ui (`@radix-ui/*`) | UI 组件库 |

---

## 附录 B: 数据存储规划

| 数据 | 存储方式 | 位置 |
|---|---|---|
| Agent 配置 | Rust JsonStore | `~/.clawx/clawx-agents.json` |
| Provider 配置 + API Key | Rust JsonStore | `~/.clawx/clawx-providers.json` |
| 设置 | Rust JsonStore | `~/.clawx/clawx-settings.json` |
| 知识库元数据 | Rust JsonStore | `~/.clawx/clawx-knowledge.json` |
| 知识库向量数据 | rusqlite | `~/.clawx/knowledge.db` |
| 工作流配置 | Rust JsonStore | `~/.clawx/clawx-workflows.json` |
| 工作流执行日志 | rusqlite | `~/.clawx/runs.db` |
| 截图临时文件 | 文件系统 | `~/.clawx/screenshots/` |
| 应用日志 | 文件系统 | `~/.clawx/logs/` |
