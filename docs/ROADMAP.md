# ClawX 功能演进路线图

> 本文档是 ClawX 项目所有功能规划、变更和追踪的**唯一基准文档**。
> 任何新功能的添加、现有功能的修改都必须在此文档中记录并更新状态。

## 文档索引

| 文档 | 说明 |
|---|---|
| [ROADMAP.md](./ROADMAP.md) | 功能演进路线图（本文档） |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 详细技术架构与实现方案 |
| [TAURI_MIGRATION.md](./TAURI_MIGRATION.md) | Electron → Tauri 2 迁移指南 |
| [CHANGELOG.md](./CHANGELOG.md) | 功能变更日志 |

---

## 项目愿景

将 ClawX 从「OpenClaw 的桌面 GUI」演进为**「多 Agent 桌面智能助手平台」**，融合：

- **多 Agent 协作 & 知识库管理**：支持创建多个 Agent 角色，各自拥有独立人设、技能集和知识库
- **桌面 AI 助手 & 本地自动化**：全局快捷键呼出、屏幕感知、剪贴板智能、本地文件检索

最终形态：一个随时随地可呼出的、具备多 Agent 能力的桌面 AI 助手。

---

## 里程碑总览

```
v0.1.x  ─── Phase 1: 多 Agent 角色管理              ✅
v0.1.x  ─── Phase 2: 桌面助手核心能力               ✅
v0.1.x  ─── Phase 3: 知识库 & RAG                   ✅
v0.1.x  ─── Phase 4: Agent 协作 & 高级自动化         ✅
v0.2.0  ─── Phase 5: Tauri 2 迁移 (Electron 移除)    ✅
v1.0.0  ─── Phase 6: 稳定性 + 功能补全 + 正式发布     ✅
```

---

## Phase 1: 多 Agent 角色管理 `v0.2.x`

> 目标：从单一 Chat 升级为多 Agent 切换，每个 Agent 有独立人设和配置

### 功能清单

| ID | 功能 | 优先级 | 状态 | 说明 |
|---|---|---|---|---|
| F-1.1 | Agent CRUD 管理 | P0 | `已完成` | 创建/编辑/删除/克隆 Agent |
| F-1.2 | Agent 配置面板 | P0 | `已完成` | Name, Avatar, System Prompt, Model, Temperature, Skills |
| F-1.3 | Chat 页面 Agent 切换 | P0 | `已完成` | 顶部下拉选择当前 Agent |
| F-1.4 | Agent 预设模板 | P1 | `已完成` | 内置常用 Agent 模板（翻译、编程、写作等） |
| F-1.5 | Channel 绑定 Agent | P1 | `已完成` | 每个消息通道可绑定不同 Agent |
| F-1.6 | Agent 独立会话历史 | P1 | `已完成` | 每个 Agent 维护自己的对话记录 |
| F-1.7 | Agent 导入/导出 | P2 | `已完成` | JSON 格式导入导出 Agent 配置 |

### 技术要点
- 新增 `src/stores/agents.ts` (Zustand store)
- 新增 `src/pages/Agents/` 页面
- Rust: `src-tauri/src/storage/agents.rs` (JSON 文件持久化) + `commands/agents_cmd.rs` (9 个命令)
- 新增 i18n 命名空间 `agents`

---

## Phase 2: 桌面助手核心能力 `v0.3.x`

> 目标：从「打开应用才能用」变成「随时随地呼出 AI」

### 功能清单

| ID | 功能 | 优先级 | 状态 | 说明 |
|---|---|---|---|---|
| F-2.1 | 全局快捷键呼出 | P0 | `已完成` | 系统级快捷键（默认 `⌃⇧Space`）弹出悬浮窗 |
| F-2.2 | 悬浮快捷窗口 | P0 | `已完成` | Spotlight 风格的轻量输入窗口 |
| F-2.3 | 剪贴板智能处理 | P0 | `已完成` | 自动检测剪贴板内容，提供上下文相关的 AI 操作 |
| F-2.4 | 屏幕截图分析 | P1 | `已完成` | 截取屏幕区域发送给 AI 分析 |
| F-2.5 | 快捷指令系统 | P1 | `已完成` | 预定义命令（翻译选中文本、总结剪贴板、解释代码等） |
| F-2.6 | 本地文件快速检索 | P2 | `已完成` | 通过 AI 对话搜索和分析本地文件 |
| F-2.7 | 系统通知集成 | P2 | `已完成` | Agent 执行结果通过系统通知推送 |

### 技术要点
- Tauri: `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`
- Spotlight 窗口: `tauri.conf.json` 独立 WebView 窗口 + `spotlight_toggle`/`spotlight_hide` 命令
- 截图: macOS `screencapture` / Linux `gnome-screenshot` CLI
- 新增 `src/pages/Spotlight/` (悬浮窗 UI)

---

## Phase 3: 知识库 & RAG `v0.4.x`

> 目标：让 Agent 拥有专属知识库，实现检索增强生成

### 功能清单

| ID | 功能 | 优先级 | 状态 | 说明 |
|---|---|---|---|---|
| F-3.1 | 知识库 CRUD | P0 | `已完成` | 创建/编辑/删除知识库 |
| F-3.2 | 文档上传与解析 | P0 | `已完成` | 支持 PDF, TXT, MD, DOCX |
| F-3.3 | 文本分块与向量化 | P0 | `已完成` | 混合 Embedding（Provider API + 本地 MiniLM），SQLite + sqlite-vec |
| F-3.4 | Agent 关联知识库 | P0 | `已完成` | 一个 Agent 可关联多个知识库（多选 Checkbox） |
| F-3.5 | RAG 检索增强 | P0 | `已完成` | 对话时自动检索知识库相关内容注入上下文（Chat + Spotlight） |
| F-3.6 | 知识库搜索预览 | P1 | `已完成` | 可在 UI 中搜索和预览知识库内容，片段高亮 |
| F-3.7 | 网页抓取入库 | P2 | `已完成` | 粘贴 URL 自动抓取内容加入知识库（Readability + Turndown） |
| F-3.8 | 本地文件夹监听 | P2 | `已完成` | 监听指定文件夹，自动同步到知识库（chokidar + 防抖） |

### 技术要点
- 向量数据库：Rust `rusqlite` (bundled)，f32 BLOB + 纯 Rust 余弦相似度搜索
- Embedding：OpenAI 兼容 `/embeddings` 端点（reqwest，batch size 32）
- 文档解析：Rust `lopdf` (PDF), `zip`+`quick-xml` (DOCX), 纯文本直接读取
- 文本分块：Rust 递归字符分割，支持自定义 chunkSize / chunkOverlap
- 网页抓取：Rust `reqwest` + `scraper` CSS 选择器正文提取
- 文件夹监听：Rust `notify` crate，500ms 防抖
- 新增 `src/stores/knowledge.ts` (Zustand store) + `src/pages/Knowledge/`
- Rust 模块：`database/vector_db.rs`, `document_parser.rs`, `text_chunker.rs`, `embedding.rs`, `knowledge_pipeline.rs`, `rag_engine.rs`, `web_scraper.rs`, `folder_watcher.rs`
- 15 个 `knowledge:*` Tauri 命令

---

## Phase 4: Agent 协作 & 高级自动化 `v0.5.x`

> 目标：多 Agent 之间可以协作，实现复杂自动化任务链

### 功能清单

| ID | 功能 | 优先级 | 状态 | 说明 |
|---|---|---|---|---|
| F-4.1 | Agent 间消息传递 | P0 | `已完成` | Agent A 的输出可以作为 Agent B 的输入 |
| F-4.2 | 任务链编排 | P0 | `已完成` | 可视化节点编排，支持线性/分支/并行 |
| F-4.3 | 条件路由 | P1 | `已完成` | Keyword / Regex / AI 分类三种条件类型 |
| F-4.4 | 并行执行 | P1 | `已完成` | 同层级节点 Promise.all 并发，Merge 节点合并 |
| F-4.5 | 自动化触发器 | P1 | `已完成` | Cron / 文件变化 / 剪贴板 / 快捷键四种触发器 |
| F-4.6 | 执行日志与回放 | P2 | `已完成` | SQLite 存储执行记录，步骤级 input/output 回看 |
| F-4.7 | 任务链模板市场 | P2 | `已完成` | 6 个内置模板 + JSON 导入导出 |

### 技术要点
- 可视化编辑器：`@xyflow/react` 节点画布，5 种自定义节点（Input/Output/Agent/Condition/Merge）
- 执行引擎：Rust `workflow_engine.rs` (DAG 验证 + Kahn 拓扑排序 + 逐层并行执行)
- 通过 `GatewayClient.rpc("chat.send", ...)` 调用 Gateway 执行 Agent 对话
- 触发器：Rust `cron` crate (定时) + `notify` (文件) + 剪贴板轮询 + 快捷键映射
- 执行日志：Rust `rusqlite` → `~/.clawx/runs.db`
- 工作流配置：Rust `JsonStore` → `clawx-workflows.json`
- 新增 `src/stores/workflow.ts` + `src/pages/Workflows/` (11 + 6 组件)
- 18 个 `workflow:*` Tauri 命令 + `workflow_stepProgress` 事件
- 新增 i18n 命名空间 `workflows`，`src/data/workflow-templates.ts` (6 个内置模板)

---

## Phase 5: Tauri 2 迁移 `v0.2.0` ✅ 已完成

> 目标：从 Electron 迁移到 Tauri 2，解决内存泄漏和包体积问题
> **状态：已完成。** Electron + Node.js Sidecar 已完全移除，123 个 Rust 命令 + 9 个原生事件。

详见 [TAURI_MIGRATION.md](./TAURI_MIGRATION.md)

---

## Phase 6: 稳定性 + 功能补全 + 正式发布 `v1.0.0` ✅ 已完成

> 目标：修复 P0/P1 稳定性问题，补全缺失功能，打磨 UX，迁移 CI 至 Tauri 2

### 功能清单

| ID | 功能 | 优先级 | 状态 | 说明 |
|---|---|---|---|---|
| F-6.1 | Ed25519 设备认证 | P0 | `已完成` | 替换 controlUi bypass 临时方案 |
| F-6.2 | Gateway 事件监听器泄漏修复 | P0 | `已完成` | init() 5 个 on() 保存 unsubscribe + destroy() |
| F-6.3 | 快捷键持久化 | P0 | `已完成` | settings_store 读写 + 启动时注册 |
| F-6.4 | Chat switchSession 竞态修复 | P1 | `已完成` | loadHistory 守卫 sessionKeyAtStart |
| F-6.5 | RPC pending 上限 | P1 | `已完成` | HashMap 1000 上限 |
| F-6.6 | Gateway stdout/stderr 捕获 | P1 | `已完成` | Stdio::piped + tokio::spawn |
| F-6.7 | 日语 i18n 补全 | P1 | `已完成` | updates.status/action 15 键 |
| F-6.8 | Chat 错误重试按钮 | P1 | `已完成` | lastSentText + retryLastMessage() |
| F-6.9 | 滚动到底部按钮 | P2 | `已完成` | onScroll 距离检测 + ArrowDown |
| F-6.10 | 工作流监听器热重载修复 | P2 | `已完成` | unsub 先清理再注册 |
| F-6.11 | Sidecar 注释清理 | P2 | `已完成` | 4 个 Rust 文件 |
| F-6.12 | CI 迁移至 Tauri 2 | P0 | `已完成` | release.yml → tauri-action + tauri-update.json |
| F-6.13 | Updater 配置 | P0 | `已完成` | endpoints + pubkey 配置 |
| F-6.14 | 图标补全 | P1 | `已完成` | 256x256 + 512x512 |
| F-6.15 | 开机自启动 | P1 | `已完成` | tauri-plugin-autostart 集成，三语 UI 开关 |
| F-6.16 | 截图命令 Rust 实现 | P1 | `已完成` | macOS/Linux/Windows 三平台 screenshot:capture |
| F-6.17 | 代码质量清理 | P2 | `已完成` | debug 日志清理、unwrap 安全化、空 catch 修复 |
| F-6.18 | README 全面更新 | P1 | `已完成` | Electron → Tauri 2 架构更新 |

---

## 功能状态说明

| 状态 | 含义 |
|---|---|
| `待开发` | 已规划，未开始 |
| `开发中` | 正在实现 |
| `测试中` | 开发完成，正在测试 |
| `已完成` | 已合并到主分支 |
| `已废弃` | 不再计划实现 |

---

## 变更记录

| 日期 | 变更内容 | 操作人 |
|---|---|---|
| 2026-02-18 | 初始版本，创建完整路线图 | - |
| 2026-02-19 | Phase 1 全部功能标记为已完成 | - |
| 2026-02-19 | Phase 2 P0 (F-2.1/F-2.2/F-2.3) 实现完成 | - |
| 2026-02-19 | F-2.4 屏幕截图分析实现完成 | - |
| 2026-02-19 | F-2.5 快捷指令系统实现完成 | - |
| 2026-02-19 | F-2.6 本地文件快速检索实现完成 | - |
| 2026-02-19 | F-2.7 系统通知集成实现完成，Phase 2 全部完成 | - |
| 2026-02-19 | Phase 3 全部功能 (F-3.1 ~ F-3.8) 实现完成 | - |
| 2026-02-19 | Phase 4 全部功能 (F-4.1 ~ F-4.7) 实现完成 | - |
| 2026-02-20 | Phase 5 Tauri 迁移完成，Electron + Sidecar 完全移除，v0.2.0 | - |
| 2026-02-23 | Phase 6 全部功能 (F-6.1 ~ F-6.18) 实现完成，v1.0.0 | - |
