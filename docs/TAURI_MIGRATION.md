# Electron → Tauri 2 迁移指南

> **迁移状态：✅ 已完成 (v0.2.0)**
>
> Electron 和 Node.js Sidecar 已完全移除。应用现在是纯 Tauri 原生架构。

---

## 目录

- [1. 迁移动机](#1-迁移动机)
- [2. Electron vs Tauri 2 能力对照](#2-electron-vs-tauri-2-能力对照)
- [3. 当前 Electron 依赖清单](#3-当前-electron-依赖清单)
- [4. 迁移策略](#4-迁移策略)
- [5. 风险评估与应对](#5-风险评估与应对)
- [6. 迁移实施步骤](#6-迁移实施步骤)

---

## 1. 迁移动机

| 问题 | Electron 现状 | Tauri 2 预期 |
|---|---|---|
| 内存占用 | 300-800MB（Chromium 进程） | 30-80MB（系统 WebView） |
| 安装包大小 | ~150MB (含 Chromium) | ~10-15MB (无捆绑浏览器) |
| 启动速度 | 2-5s | <1s |
| 内存泄漏 | Chromium 已知问题，长时间运行后内存持续增长 | Rust 内存安全，泄漏风险极低 |
| 进程数量 | 5+ 进程（main, renderer, GPU, utility, gateway） | 1-2 进程（main + gateway） |
| 安全性 | Node.js 完全访问 | Rust 命令白名单，默认拒绝 |

---

## 2. Electron vs Tauri 2 能力对照

### 2.1 核心 API 对照

| 功能 | Electron API | Tauri 2 等效 |
|---|---|---|
| 窗口创建 | `new BrowserWindow()` | `WebviewWindow::new()` / `tauri::window` |
| IPC 通信 | `ipcMain.handle()` / `ipcRenderer.invoke()` | `#[tauri::command]` + `invoke()` |
| 事件推送 | `webContents.send()` | `app.emit()` / `window.emit()` |
| 系统托盘 | `new Tray()` | `tauri-plugin-tray` (内置) |
| 全局快捷键 | `globalShortcut.register()` | `tauri-plugin-global-shortcut` |
| 剪贴板 | `clipboard.readText()` | `tauri-plugin-clipboard-manager` |
| 文件对话框 | `dialog.showOpenDialog()` | `tauri-plugin-dialog` |
| Shell 调用 | `shell.openExternal()` | `tauri-plugin-shell` |
| 自动更新 | `electron-updater` | `tauri-plugin-updater` |
| 持久化存储 | `electron-store` | `tauri-plugin-store` |
| 通知 | `new Notification()` | `tauri-plugin-notification` |
| 子进程 | `child_process.spawn()` | `tauri-plugin-shell` (sidecar) |
| 菜单 | `Menu.buildFromTemplate()` | `tauri::menu` |
| 屏幕信息 | `screen.getPrimaryDisplay()` | `tauri::window::Monitor` |
| 日志 | 自定义 logger | `tauri-plugin-log` |
| 文件系统 | `fs` (Node.js) | `tauri-plugin-fs` |
| HTTP 请求 | `net` / `fetch` | `tauri-plugin-http` |

### 2.2 Tauri 2 Plugin 生态

| 需求 | 推荐 Plugin | 状态 |
|---|---|---|
| 全局快捷键 | `tauri-plugin-global-shortcut` | 稳定 |
| 剪贴板 | `tauri-plugin-clipboard-manager` | 稳定 |
| 文件系统 | `tauri-plugin-fs` | 稳定 |
| 对话框 | `tauri-plugin-dialog` | 稳定 |
| Shell/子进程 | `tauri-plugin-shell` | 稳定 |
| 自动更新 | `tauri-plugin-updater` | 稳定 |
| 持久化存储 | `tauri-plugin-store` | 稳定 |
| 系统通知 | `tauri-plugin-notification` | 稳定 |
| 系统托盘 | `tauri-plugin-tray` (内置) | 稳定 |
| WebSocket | Rust `tokio-tungstenite` | 成熟 |
| SQLite | `tauri-plugin-sql` | 稳定 |
| 日志 | `tauri-plugin-log` | 稳定 |

### 2.3 不兼容项 & 替代方案

| Electron 特性 | 问题 | Tauri 2 替代方案 |
|---|---|---|
| `desktopCapturer` | Tauri 无内置截图 | macOS: `screencapture` CLI; Windows: Win32 API; 封装为 Rust 命令 |
| `ELECTRON_RUN_AS_NODE` | Gateway 启动方式 | 使用 `tauri-plugin-shell` 的 sidecar 模式管理 Gateway 进程 |
| `webview` 嵌入 | DevConsole 使用 webview | Tauri 原生支持多 webview |
| `vibrancy` 毛玻璃 | macOS 窗口特效 | `window.set_effects()` (Tauri 原生支持 macOS 毛玻璃) |
| `contextBridge` | 安全隔离 | Tauri 默认安全模型更严格，通过 `capabilities` 配置权限 |
| `electron-store` | 持久化存储 | `tauri-plugin-store`（API 类似，JSON 文件存储） |

---

## 3. Electron 依赖清单（已移除）

> 以下所有 Electron 依赖和模块已在 v0.2.0 中完全移除。

### 3.1 已移除的直接依赖

```
electron: ^40.2.1                   ❌ 已移除
electron-builder: ^26.0.12          ❌ 已移除
electron-updater: ^6.6.2            ❌ 已移除
electron-store: ^11.0.0             ❌ 已移除
vite-plugin-electron: ^0.32.2       ❌ 已移除
vite-plugin-electron-renderer: ^0.14.6  ❌ 已移除
better-sqlite3, sqlite-vec          ❌ 已移除 (改用 rusqlite)
@huggingface/transformers            ❌ 已移除
ws                                   ❌ 已移除 (改用 tokio-tungstenite)
pdf-parse, mammoth                   ❌ 已移除 (改用 lopdf, zip+quick-xml)
turndown, @mozilla/readability       ❌ 已移除 (改用 reqwest+scraper)
chokidar                             ❌ 已移除 (改用 notify crate)
```

### 3.2 已移除的 Electron 模块

所有 `electron/` 目录文件已删除：
- `electron/main/` — 8 个文件 → Rust `src-tauri/src/lib.rs` + commands
- `electron/preload/` — 1 个文件 → 移除（Tauri 不需要）
- `electron/gateway/` — 4 个文件 → Rust `gateway/client.rs` + `gateway_lifecycle.rs`
- `electron/utils/` — 28 个文件 → Rust 对应模块
- `electron/sidecar/` — 4 个文件 → 移除（事件改为 Tauri emit）

### 3.3 渲染进程迁移结果

| 文件 | 原耦合方式 | 迁移结果 |
|---|---|---|
| `src/types/electron.d.ts` | `window.electron` 类型 | ✅ 仅保留 `window.__TAURI_INTERNALS__` |
| 所有 stores | `window.electron.ipcRenderer.invoke()` | ✅ 使用 `invoke()` from `@/lib/bridge` |
| `src/App.tsx` | `window.electron.ipcRenderer.on()` | ✅ 使用 `on()` from `@/lib/bridge` |
| `src/lib/bridge.ts` | Electron + SSE + Sidecar | ✅ 纯 Tauri invoke + listen |

---

## 4. 迁移策略

### 4.1 渐进式迁移（已完成）

```
阶段 A: 引入抽象层 ✅ (Week 1)
  ├── ✅ 创建 src/lib/bridge.ts (IPC 抽象)
  ├── ✅ 所有 stores + pages 通过 bridge.invoke() / bridge.on()
  └── ✅ 26 个文件从 window.electron 迁移到 bridge

阶段 B: 搭建 Tauri 骨架 ✅ (Week 1)
  ├── ✅ 初始化 src-tauri/ (Cargo.toml + tauri.conf.json)
  ├── ✅ 12 个原生命令 + 7 个 plugins + 系统托盘 + Spotlight 窗口
  └── ✅ Node.js Sidecar HTTP 代理 108 个业务通道

阶段 C: 业务逻辑 Rust 重写 ✅ (Week 3-7)
  ├── ✅ Week 3: 28 个 CRUD 命令 (Agent/Knowledge/Workflow)
  ├── ✅ Week 4: 6 个数据库命令 (SQLite + 向量搜索 + RAG)
  ├── ✅ Week 5: 18 个网络命令 (Gateway WS + Cron + Channel)
  ├── ✅ Week 6: 37 个工具命令 (Provider + File + OpenClaw)
  └── ✅ Week 7: 22 个最终命令 (文档管线 + 工作流引擎 + Gateway 生命周期)

阶段 D: 清理 ✅ (Week 8)
  ├── ✅ 移除全部 Electron 代码 (~39 文件)
  ├── ✅ 移除全部 Sidecar 代码 (~10 文件)
  ├── ✅ SSE 事件 → Tauri 原生 emit/listen
  ├── ✅ 移除 WhatsApp channel
  └── ✅ 文档和版本号更新
```

### 4.2 项目结构（迁移后最终状态）

```
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs          # AppState + 123 个命令注册 + 系统托盘
│   │   ├── commands/       # 10 个命令模块
│   │   │   ├── agents_cmd.rs, knowledge_cmd.rs, workflows_cmd.rs
│   │   │   ├── gateway_cmd.rs, cron_cmd.rs, channel_cmd.rs
│   │   │   ├── provider_cmd.rs, skill_cmd.rs, openclaw_cmd.rs
│   │   │   ├── chat_cmd.rs, clawhub_cmd.rs, log_cmd.rs
│   │   │   ├── file_cmd.rs, filesearch_cmd.rs, uv_cmd.rs
│   │   │   └── mod.rs
│   │   ├── gateway/        # Gateway WS 客户端
│   │   │   ├── client.rs   # Actor 模式 WS + 推送事件转发
│   │   │   └── protocol.rs # OpenClaw v3 帧定义
│   │   ├── storage/        # JSON 文件存储
│   │   │   ├── mod.rs, agents.rs, knowledge.rs, workflows.rs
│   │   ├── database/       # SQLite 数据库
│   │   │   ├── mod.rs, vector_db.rs, workflow_runs.rs
│   │   └── ...             # 20+ 业务模块
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── icons/
├── src/                    # React 前端
│   ├── lib/bridge.ts       # 纯 Tauri IPC 桥接层
│   ├── stores/             # Zustand stores
│   ├── pages/              # React pages
│   ├── components/         # UI 组件
│   └── i18n/               # 国际化
└── package.json
```

### 4.3 前端代码迁移结果

| 层级 | 结果 | 说明 |
|---|---|---|
| `src/components/ui/` | 0 文件修改 | 纯 UI，无需修改 |
| `src/components/layout/` | TitleBar + Sidebar | 通过 bridge 层调用 |
| `src/pages/` | Week 1 完成 | 全部通过 bridge.invoke() |
| `src/stores/` | Week 1 完成 | 全部通过 bridge.invoke() + bridge.on() |
| `src/lib/bridge.ts` | 最终简化至 ~100 行 | 纯 Tauri invoke + listen |
| `src/i18n/` | 0 文件修改 | 纯数据 |

**实际结果：前端迁移量 < 3%，抽象层策略成功。**

---

## 5. 风险评估与应对

| 风险 | 影响 | 概率 | 应对措施 |
|---|---|---|---|
| WebView 兼容性差异 | CSS/JS 在不同平台 WebView 上表现不一 | 中 | 使用标准 API，避免 Chromium 特有 CSS；充分测试 |
| Node.js 生态丢失 | 部分 npm 包依赖 Node.js API | 高 | 需要 Node.js 功能的模块通过 sidecar 或 Rust 重写 |
| Gateway 管理复杂化 | Tauri sidecar 管理不如 child_process 灵活 | 中 | 使用 `tauri-plugin-shell` + 自定义 Rust 进程管理 |
| sqlite-vss 兼容性 | 知识库向量数据库需要 native 模块 | 中 | 改用 Rust `hnsw` 库或 `sqlite-vec` (Rust 原生) |
| 开发体验下降 | Rust 编译慢，学习曲线陡 | 高 | 团队 Rust 培训；利用 `cargo watch` 加速迭代 |
| macOS 签名差异 | Tauri 签名流程不同 | 低 | Tauri 内置签名支持，调整 CI 即可 |

---

## 6. 迁移实施进度

> 以下为迁移的详细实施步骤和完成状态。

### Week 1: Bridge 抽象层 + Tauri 骨架 ✅

**目标**：解耦前端与 Electron，搭建 Tauri 2 项目骨架。

- [x] 安装 Rust 工具链 (`rustup`) + Tauri CLI
- [x] 创建 `src/lib/bridge.ts` — IPC 抽象层（Electron / Tauri 双模式）
- [x] 重构所有 stores (`src/stores/*.ts`) 通过 `bridge.invoke()` 调用
- [x] 重构所有 pages/components 通过 bridge 层调用
- [x] 初始化 `src-tauri/` 项目骨架（Cargo.toml, tauri.conf.json, capabilities, icons）
- [x] 实现 12 个 Rust 原生命令（window, clipboard, spotlight, shortcut, app）
- [x] 集成 7 个 Tauri Plugin（global-shortcut, clipboard, notification, dialog, store, process, shell）
- [x] 创建系统托盘（show/hide/spotlight/quit）
- [x] Spotlight 窗口配置（tauri.conf.json + toggle 命令）

### Week 2: Node.js Sidecar HTTP 服务器 ✅

**目标**：构建独立的 Node.js HTTP 服务器（端口 18790），使 Tauri 模式下所有 ~108 个非原生 IPC 通道正常工作。

#### Step 1: 运行时抽象层
- [x] 新建 `electron/utils/runtime.ts` — 统一 Electron/Sidecar 环境检测与路径解析
- [x] 修改 `electron/utils/paths.ts` — `app.getPath()` → `runtime.getUserDataPath()`
- [x] 修改 `electron/utils/logger.ts` — `app.getPath()/getVersion()` → `runtime.*`
- [x] 修改 `electron/utils/openclaw-cli.ts` — `app.isPackaged` → `runtime.isPackaged()`
- [x] 修改 `electron/utils/uv-env.ts` — `app.getLocale()/whenReady()` → `runtime.*`
- [x] 修改 `electron/utils/uv-setup.ts` — `app.isPackaged` → `runtime.isPackaged()`

#### Step 2: GatewayManager 解耦
- [x] 修改 `electron/gateway/manager.ts` — 移除 `import { app } from 'electron'`，Sidecar 模式下简化进程启动
- [x] 修改 `electron/gateway/clawhub.ts` — `shell.openPath()` → `runtime.openPath()`，条件导入 Electron API

#### Step 3: Sidecar HTTP 服务器
- [x] 新建 `electron/sidecar/server.ts` — Node.js 原生 HTTP 服务器（端口 18790）
- [x] 新建 `electron/sidecar/router.ts` — 全部 ~108 个 IPC 通道映射为 HTTP 路由
- [x] 新建 `electron/sidecar/events.ts` — SSE 事件推送管理器（替代 `webContents.send()`）
- [x] Gateway 事件自动转发到 SSE（status, message, notification, chat:message, exit, error）

#### Step 4: Bridge 增强 + SSE 客户端
- [x] 新建 `src/lib/sse-client.ts` — Renderer 侧 SSE 客户端（自动重连）
- [x] 修改 `src/lib/bridge.ts` — Tauri bridge `on()` 方法支持 SSE 事件订阅

#### Step 5: Tauri Sidecar 生命周期
- [x] 修改 `src-tauri/src/lib.rs` — `tauri-plugin-shell` 启动 Node.js sidecar 进程
- [x] 新建 `electron/sidecar/build.ts` — esbuild 打包为单文件 `dist-sidecar/server.js`
- [x] 修改 `package.json` — 新增 `sidecar:dev` / `sidecar:build` 脚本

#### Step 6: Electron-only 模块适配
- [x] 修改 `electron/utils/notification.ts` — Sidecar 模式通过 SSE 推送通知事件
- [x] 修改 `electron/utils/screenshot.ts` — 无 nativeImage 时直接读取原始 base64
- [x] 修改 `electron/utils/workflow-triggers.ts` — Sidecar 模式下 cron/file-watch 正常运行，shortcut 由 Tauri Rust 处理

#### Step 7: 文档更新
- [x] 更新 `docs/CHANGELOG.md` — 新增 `[0.6.0-alpha.2]`
- [x] 更新 `docs/TAURI_MIGRATION.md` — Week 2 完成状态
- [x] 更新 `docs/ARCHITECTURE.md` — Sidecar 架构图

### Week 3: 纯逻辑模块 Rust 重写 ✅

**目标**：将 Agent/Knowledge/Workflow 存储 CRUD + Provider Registry + Text Chunker 重写为 Rust，28 个通道直接由 Rust 处理。

#### Phase A: 纯逻辑模块
- [x] `config.ts` → `src-tauri/src/config.rs` (Rust const)
- [x] `provider-registry.ts` → `src-tauri/src/providers.rs` (Rust struct，内部模块)
- [x] `text-chunker.ts` → `src-tauri/src/text_chunker.rs` (Rust 算法，含单元测试)
- [x] `agent-storage.ts` → `src-tauri/src/storage/agents.rs` + `commands/agents_cmd.rs` (9 个命令)
- [x] `knowledge-storage.ts` → `src-tauri/src/storage/knowledge.rs` + `commands/knowledge_cmd.rs` (11 个命令)
- [x] `workflow-storage.ts` → `src-tauri/src/storage/workflows.rs` + `commands/workflows_cmd.rs` (8 个命令)
- [x] 通用 JSON 文件存储抽象 → `src-tauri/src/storage/mod.rs` (`JsonStore<T>`)
- [x] Bridge 存储通道路由 → `src/lib/bridge.ts` (STORAGE_PREFIXES + _args 包装)
- [x] lib.rs 扩展 — 12 → 40 个命令，AppState 新增 data_dir
- [x] 26 个单元测试全部通过 (`cargo test`)

### Week 4: 数据库 + 搜索模块 Rust 重写 ✅

**目标**：SQLite 向量搜索全链路 Rust 化 + Workflow Run DB + Bridge 路由 Bug 修复，6 个新命令（40 → 46）。

#### Bug Fix: Bridge 路由
- [x] `src/lib/bridge.ts` — 新增 SIDECAR_OVERRIDES 白名单，修复 storage-prefixed 但未迁移的通道（如 knowledge:addDocument）在 Tauri 模式失败的关键 Bug

#### Phase B: 数据库模块
- [x] `database/mod.rs` — DbManager 懒初始化双 SQLite 连接（knowledge.db + runs.db），WAL 模式，embedding BLOB 列迁移
- [x] `vector-db.ts` → `database/vector_db.rs` — 纯 Rust 余弦相似度搜索（替代 sqlite-vec 虚拟表），f32 little-endian BLOB 存储
- [x] `workflow-run-db.ts` → `database/workflow_runs.rs` — 4 个读/删命令（getRuns/getRun/deleteRun/clearRuns）
- [x] `embedding.ts` (API 部分) → `embedding.rs` — reqwest 调用 OpenAI 兼容 /embeddings 端点，batch size 32（本地 ONNX 延迟到 Week 6+）
- [x] `rag-engine.ts` → `rag_engine.rs` — 按 embedding model 分组 → 嵌入查询 → 向量搜索 → 合并 top_k → token 截断
- [x] 新建 `secure_storage.rs` — 只读访问 clawx-providers.json（provider 配置 + API key）
- [x] `commands/knowledge_cmd.rs` — create/delete 增加 SQLite KB 记录管理，新增 search/rag 命令
- [x] `commands/workflows_cmd.rs` — 新增 4 个 workflow run 命令
- [x] `lib.rs` — AppState 新增 db_manager, 注册 6 个新命令（40 → 46）
- [x] Cargo.toml — 新增 rusqlite (bundled), reqwest, dirs, byteorder
- [x] 48 个单元测试全部通过 (`cargo test`)

### Week 5: 网络模块 Rust 重写 ✅

**目标**：Gateway WS 客户端 + Cron + Channel Config + Chat 命令，18 个新命令（46 → 64）。

#### 混合方案
- Sidecar 继续管理 Gateway 进程生命周期（start/stop/restart + Python 环境检测）
- Rust 建立独立 WS 连接用于 RPC 调用（Actor 模式，tokio-tungstenite）
- 两个 WS 连接（Sidecar 和 Rust）共存，Gateway 支持多客户端连接

#### Phase C: 网络模块
- [x] `settings_store.rs` — 只读访问 clawx-settings.json 获取 gatewayToken
- [x] `gateway/protocol.rs` — OpenClaw Protocol v3 帧类型（Request/Response/Event），connect 握手帧
- [x] `gateway/client.rs` — Actor 模式 WS 客户端（mpsc + oneshot），惰性连接，断线标记 + 重连
- [x] `channel-config.ts` → `channel_config.rs` — Channel Config CRUD，Discord/Telegram/WhatsApp 特殊处理
- [x] `commands/gateway_cmd.rs` — 5 个 gateway 命令（status/isConnected/rpc/getControlUiUrl/health）
- [x] `commands/cron_cmd.rs` — 6 个 cron 命令（list/create/update/delete/toggle/trigger），含 GatewayCronJob 转换
- [x] `commands/channel_cmd.rs` — 6 个 channel 命令（saveConfig/getConfig/getFormValues/deleteConfig/listConfigured/setEnabled）
- [x] `commands/chat_cmd.rs` — 1 个 chat 命令（sendWithMedia），base64 图片附件 + 文件引用
- [x] `lib.rs` — AppState 新增 gateway_client, 注册 18 个新命令（46 → 64）
- [x] `bridge.ts` — STORAGE_PREFIXES + SIDECAR_OVERRIDES 扩展，on() 事件路由修复
- [x] Cargo.toml — 新增 tokio-tungstenite, futures-util, base64
- [x] 55 个单元测试全部通过 (`cargo test`)

#### 保留在 Sidecar 的通道
| 通道 | 原因 |
|---|---|
| `gateway:start/stop/restart` | 需要进程 spawn/kill + Python 环境检测 |
| `channel:validate/validateCredentials` | 需要 execSync/fetch 调用外部 API |
| `channel:requestWhatsAppQr/cancelWhatsAppQr` | 需要 WhatsApp 登录管理器 |

### Week 6: Provider + Utility 模块 Rust 重写 ✅

**目标**：Provider 管理、Skill 配置、OpenClaw CLI、Log、File Staging、FileSearch、UV 环境检测，37 个新命令（64 → 101）。

#### Phase D: Provider + Utility 模块
- [x] `secure_storage.rs` 扩展为完整 CRUD — 原子写入，save/delete/store_api_key/set_default
- [x] `openclaw_auth.rs` — auth-profiles.json + openclaw.json 写入，同步 API key 和 default model
- [x] `openclaw_paths.rs` — OpenClaw 路径解析 + 状态检查
- [x] `provider_validate.rs` — 4 种 API key HTTP 验证 profile (reqwest)
- [x] `file_staging.rs` — 文件暂存 + image crate 图片 resize (512px)
- [x] `file_search.rs` — mdfind (macOS) + walkdir 回退 + 安全文件读取
- [x] `commands/provider_cmd.rs` — 12 个 provider 命令
- [x] `commands/skill_cmd.rs` — 3 个 skill 命令
- [x] `commands/openclaw_cmd.rs` — 7 个 openclaw 命令
- [x] `commands/log_cmd.rs` — 4 个 log 命令
- [x] `commands/file_cmd.rs` — 3 个 file/media 命令
- [x] `commands/filesearch_cmd.rs` — 2 个 filesearch 命令
- [x] `commands/uv_cmd.rs` — 2 个 uv 命令
- [x] `lib.rs` — 注册 37 个新命令（64 → 101）
- [x] `bridge.ts` — STORAGE_PREFIXES + SIDECAR_OVERRIDES 更新
- [x] Cargo.toml — 新增 image v0.25, walkdir v2
- [x] 67 个单元测试全部通过 (`cargo test`)

#### 保留在 Sidecar 的通道
| 通道 | 原因 |
|---|---|
| `clawhub:search/install/uninstall/list/openSkillReadme` (5) | CLI subprocess (`openclaw hub` 命令) |
| `log:getRecent` (1) | Sidecar 内存 ring buffer |
| `gateway:start/stop/restart` (3) | 进程 spawn/kill + Python 环境 |
| `channel:requestWhatsAppQr/cancelWhatsAppQr` (2) | Baileys npm 依赖 |
| `knowledge:addDocument/addUrl/removeDocument/reprocessDocument` (4) | 文档解析管线 |
| `knowledge:setWatchFolder/getWatchStatus/getEmbeddingOptions/detectDimension` (4) | 文件监听/Embedding 选项 |
| `workflow:execute/cancel/registerTriggers/unregisterTriggers/getTemplates/importTemplate` (6) | DAG 执行引擎 |
| **总计** | **~25 个** |

### Week 7: 全面迁移 — 消除 Sidecar 依赖 ✅

**目标**：将剩余 22 个 Sidecar 通道迁移到 Rust，仅保留 3 个永久 Sidecar 通道（WhatsApp QR + log ring buffer）。

#### Phase E: 简单通道 (12 命令)
- [x] `clawhub.ts` → `clawhub.rs` — CLI subprocess 调用 (tokio::process::Command)
- [x] `channel-config.ts` (validate 部分) → `channel_validate.rs` — openclaw doctor + API 验证
- [x] `commands/clawhub_cmd.rs` — 5 个 clawhub 命令
- [x] `commands/channel_cmd.rs` 扩展 — 2 个 channel validate 命令
- [x] `commands/knowledge_cmd.rs` 扩展 — 4 个简单命令 (removeDocument/getWatchStatus/getEmbeddingOptions/detectDimension)
- [x] `commands/workflows_cmd.rs` 扩展 — 2 个模板命令 (getTemplates/importTemplate)

#### Phase F: 知识管线重写 (4 命令)
- [x] `document-parser.ts` → `document_parser.rs` — PDF (lopdf) + DOCX (zip+quick-xml) + TXT
- [x] `web-scraper.ts` → `web_scraper.rs` — reqwest + scraper HTML DOM 解析
- [x] `embedding.ts` (工具) → `embedding_api.rs` — 模型发现 + 维度检测
- [x] `knowledge-pipeline.ts` → `knowledge_pipeline.rs` — parse→chunk→embed→store 全管线
- [x] `folder-watcher.ts` → `folder_watcher.rs` — notify crate + 500ms debounce
- [x] `commands/knowledge_cmd.rs` 扩展 — 4 个管线命令 (addDocument/addUrl/reprocessDocument/setWatchFolder)

#### Phase G: Workflow Engine + Gateway 生命周期 (7 命令)
- [x] `workflow-engine.ts` (835 行) → `workflow_engine.rs` — DAG 执行引擎 (Kahn 拓扑排序 + 取消支持)
- [x] `workflow-triggers.ts` → `workflow_triggers.rs` — Cron/文件/剪贴板/快捷键触发器
- [x] `gateway/manager.ts` (生命周期) → `gateway_lifecycle.rs` — Gateway 进程 spawn/stop/restart
- [x] `commands/workflows_cmd.rs` 扩展 — 4 个运行时命令 (execute/cancel/registerTriggers/unregisterTriggers)
- [x] `commands/gateway_cmd.rs` 扩展 — 3 个生命周期命令 (start/stop/restart)

#### Phase H: 集成
- [x] `lib.rs` — 注册 22 个新命令（101 → 123），AppState 新增 4 个字段
- [x] `bridge.ts` — STORAGE_PREFIXES 新增 `clawhub:`，SIDECAR_OVERRIDES 从 25 缩减为 3
- [x] `commands/mod.rs` — 新增 clawhub_cmd 模块
- [x] Cargo.toml — 新增 lopdf, zip, quick-xml, scraper, notify, cron, regex (7 个依赖)
- [x] 83 个单元测试全部通过 (`cargo test`)

#### 仅剩 Sidecar 通道 (3 个永久保留)
| 通道 | 原因 | 去除条件 |
|---|---|---|
| `channel:requestWhatsAppQr` | Baileys npm WebSocket 协议栈 | 放弃 WhatsApp channel |
| `channel:cancelWhatsAppQr` | 同上 | 同上 |
| `log:getRecent` | Sidecar 进程内存 ring buffer | 改为 Rust 日志文件读取 |

### Week 8: 完全移除 Electron + Sidecar ✅

**目标**：完全移除所有 Electron 和 Node.js Sidecar 代码，实现零 Node.js 依赖。

#### Phase I: 移除 Electron 代码
- [x] 删除 `electron/main/`（8 个文件）、`electron/preload/`（1 个文件）
- [x] 删除 28 个 `electron/utils/` 文件
- [x] 删除 `electron/gateway/clawhub.ts`
- [x] 清理 package.json — 移除 ~20 个 Electron 依赖和脚本
- [x] 清理 vite.config.ts — 移除 Electron 插件
- [x] 清理 bridge.ts — 移除 Electron bridge
- [x] 清理 tsconfig — 移除 @electron alias
- [x] 移除 WhatsApp 前端引用

#### Phase II: 移除 Sidecar
- [x] `log:getRecent` 改为 Rust 日志文件读取
- [x] 扩展 GatewayClient 处理推送事件（chat/channel.status/agent/断连）
- [x] 迁移 bridge.ts 事件路由 — SSE → Tauri listen
- [x] 移除 Sidecar 启动逻辑 (`lib.rs` 中 spawn node)
- [x] 删除 `electron/sidecar/`（4 个文件）、`electron/gateway/`（3 个文件）、`electron/utils/`（3 个文件）
- [x] 删除 `src/lib/sse-client.ts`
- [x] 清理 package.json — 移除 `ws`, sidecar 脚本
- [x] 简化 bridge.ts — 统一使用 NATIVE_COMMANDS + EVENT_MAP

#### Phase III: 事件系统优化
- [x] bridge.ts 最终简化 — 移除 IBridge 接口，合并路由
- [x] 验证 payload 一致性 — emit_status 使用 `state` 字段
- [x] Gateway 断连时 emit `gateway_statusChanged` + `gateway_error`
- [x] 惰性重连 — 下次 RPC 调用自动 lazy_connect
- [x] WS 自动重连 — 指数退避（3s 基础，30s 上限，10 次上限），Actor 命令通道模式避免 Send 约束
- [x] 桌面通知 — `tauri-plugin-notification`，AI 回复 `state == "final"` 且窗口失焦时推送
- [x] Gateway Token 持久化修复 — `save_gateway_token()` 确保认证令牌写入磁盘
- [x] 残余 Electron invoke 修复 — `shell:showItemInFolder`、`dialog:open`、`update:*` 迁移到 Tauri Plugin API

### 最终验证与发布
- [x] 版本号更新至 0.2.0（package.json + Cargo.toml + tauri.conf.json）
- [x] 文档更新（CHANGELOG.md, ARCHITECTURE.md, TAURI_MIGRATION.md）
- [ ] 功能完整性测试（逐页面验证）
- [ ] 性能基准测试（内存、CPU、启动时间）
- [ ] 长时间运行稳定性测试
- [ ] 多平台兼容性测试
- [ ] 正式发布 v0.2.0

---

## 附录: Tauri 2 命令示例

```rust
// src-tauri/src/commands/agent.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub description: String,
    pub system_prompt: String,
    pub provider_id: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub skill_ids: Vec<String>,
    pub knowledge_base_ids: Vec<String>,
    pub channel_bindings: Vec<String>,
    pub is_default: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[tauri::command]
pub async fn agent_list(state: State<'_, AppState>) -> Result<Vec<AgentConfig>, String> {
    state.agent_store.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_create(
    config: AgentConfig,
    state: State<'_, AppState>,
) -> Result<AgentConfig, String> {
    state.agent_store.create(config).map_err(|e| e.to_string())
}

// 在 main.rs 中注册:
// .invoke_handler(tauri::generate_handler![
//     agent_list,
//     agent_create,
//     agent_update,
//     agent_delete,
// ])
```
