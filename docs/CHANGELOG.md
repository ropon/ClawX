# ClawX 功能变更日志

> 本文档记录所有功能的添加、修改和删除。
> 每次变更必须在此记录，格式遵循下方规范。

---

## 变更记录格式

```markdown
### [版本号] - YYYY-MM-DD

#### 新增 (Added)
- [F-x.x] 功能描述 — 涉及文件: `file1.ts`, `file2.tsx`

#### 变更 (Changed)
- [F-x.x] 变更描述 — 涉及文件: `file1.ts`

#### 修复 (Fixed)
- 修复描述 — 涉及文件: `file1.ts`

#### 移除 (Removed)
- 移除描述 — 涉及文件: `file1.ts`

#### 架构 (Architecture)
- 架构变更描述 — 涉及文件: `file1.ts`
```

**规则：**
1. `[F-x.x]` 对应 [ROADMAP.md](./ROADMAP.md) 中的功能 ID
2. 必须列出涉及的关键文件
3. 版本号遵循 SemVer，与 `package.json` 保持一致
4. 同一版本的变更合并在一个标题下

---

## 变更记录

### [1.0.0] - 2026-02-23

> v1.0.0 正式版：稳定性修复 + 功能补全 + UX 打磨 + CI 迁移至 Tauri 2

#### 修复 (Fixed)
- **Gateway 事件监听器泄漏** [P0] — `init()` 中 5 个 `on()` 监听未保存 unsubscribe 函数，新增 `_cleanups` 数组和 `destroy()` 方法 — 涉及文件: `src/stores/gateway.ts`
- **Chat switchSession 竞态** [P1] — 快速切换会话时历史加载可能覆盖当前会话，新增 `sessionKeyAtStart` 守卫 — 涉及文件: `src/stores/chat.ts`
- **RPC pending HashMap 无上限** [P1] — 新增 1000 上限防止内存泄漏 — 涉及文件: `src-tauri/src/gateway/client.rs`
- **工作流监听器热重载泄漏** [P2] — 重复调用 `initWorkflowListener()` 不再叠加监听器 — 涉及文件: `src/stores/workflow.ts`

#### 新增 (Added)
- **快捷键持久化** [P0] — 快捷键配置保存到 `clawx-settings.json`，重启后自动恢复注册 — 涉及文件: `src-tauri/src/settings_store.rs`, `src-tauri/src/lib.rs`, `src/pages/Settings/index.tsx`
- **Gateway stdout/stderr 捕获** [P1] — Gateway 进程输出改为 piped + tokio::spawn 逐行读取打印，便于调试 — 涉及文件: `src-tauri/src/gateway_lifecycle.rs`
- **Chat 错误重试按钮** [P1] — 发送失败时显示重试按钮，点击重新发送上次消息 — 涉及文件: `src/stores/chat.ts`, `src/pages/Chat/index.tsx`
- **滚动到底部按钮** [P2] — 长对话向上滚动后显示浮动 ArrowDown 按钮 — 涉及文件: `src/pages/Chat/index.tsx`
- **日语 i18n 补全** [P1] — updates.status/action 等 15 个翻译键 — 涉及文件: `src/i18n/locales/ja/settings.json`
- **三语 retry 翻译** — common.json 新增 retry 键 — 涉及文件: `src/i18n/locales/{en,zh,ja}/common.json`

#### 变更 (Changed)
- **版本号 → 1.0.0** — package.json / Cargo.toml / tauri.conf.json 三处同步
- **图标补全** — 新增 256x256.png / 512x512.png — 涉及文件: `src-tauri/icons/`, `src-tauri/tauri.conf.json`
- **Updater 配置** — tauri.conf.json 配置 endpoints 指向 OSS tauri-update.json — 涉及文件: `src-tauri/tauri.conf.json`
- **CI 迁移至 Tauri 2** — release.yml 从 electron-builder 迁移到 tauri-action，新增 tauri-update.json manifest 生成 — 涉及文件: `.github/workflows/release.yml`

#### 移除 (Removed)
- **过时 Sidecar 注释** — 清理 4 个 Rust 文件中的 Sidecar 引用 — 涉及文件: `src-tauri/src/text_chunker.rs`, `providers.rs`, `commands/log_cmd.rs`, `database/mod.rs`
- **electron-builder.yml** — 删除 Electron 时代残留的打包配置文件
- **未使用常量 BUILTIN_PROVIDER_TYPES** — 涉及文件: `src-tauri/src/providers.rs`

#### 代码质量 (Quality)
- **清理 debug console.log** — 移除 ChatInput.tsx (7处)、chat.ts (3处)、cron.ts (1处) 的调试日志 — 涉及文件: `src/pages/Chat/ChatInput.tsx`, `src/stores/chat.ts`, `src/stores/cron.ts`
- **修复静默吞异常** — update.ts 3 处 `.catch(() => {})` 改为带 warn 日志的错误处理 — 涉及文件: `src/stores/update.ts`
- **bridge.ts 事件监听错误处理** — `on()` promise 链添加 `.catch()` 防止未处理的 rejection — 涉及文件: `src/lib/bridge.ts`
- **Rust unwrap() 安全化** — `clawhub.rs` / `web_scraper.rs` 改用 `OnceLock` 延迟初始化正则；`database/mod.rs` / `workflow_engine.rs` 改用 `expect()` 附带语义说明 — 涉及文件: `src-tauri/src/clawhub.rs`, `web_scraper.rs`, `database/mod.rs`, `workflow_engine.rs`
- **空 catch 块补充日志** — Channels 和 Settings 页面 2 处空 catch 块改为 warn 日志 — 涉及文件: `src/pages/Channels/index.tsx`, `src/pages/Settings/index.tsx`

#### 新增功能补全 (Feature Completion)
- **[F-2.4] 截图命令实现** — 新增 `screenshot:capture` Tauri 命令，支持 macOS (screencapture -i)、Linux (gnome-screenshot/slurp+grim/scrot)、Windows (PowerShell) 三平台 — 涉及文件: `src-tauri/src/commands/screenshot_cmd.rs`, `commands/mod.rs`, `lib.rs`, `src/lib/bridge.ts`
- **开机自启动** — 集成 `tauri-plugin-autostart`，Settings UI 新增三语开关 — 涉及文件: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/main-window.json`, `src/stores/settings.ts`, `src/pages/Settings/index.tsx`, `src/i18n/locales/{en,zh,ja}/settings.json`
- **Updater 签名密钥** — 生成 Ed25519 密钥对，公钥配置到 tauri.conf.json — 涉及文件: `src-tauri/tauri.conf.json`

#### 文档 (Documentation)
- **README.md 全面更新** — 从 Electron 架构更新为 Tauri 2：badge、架构图、项目结构、开发命令、技术栈、功能列表 — 涉及文件: `README.md`
- **图标生成脚本修复** — 输出目录从 `resources/icons/` 改为 `src-tauri/icons/`，新增 128x128@2x 生成 — 涉及文件: `scripts/generate-icons.mjs`

---

### [0.2.0] - 2026-02-21

> F-6.1: Ed25519 设备认证 — 替换 controlUi bypass 临时方案

#### 新增 (Added)
- [F-6.1] Ed25519 设备认证 — 每个 ClawX 实例生成唯一 Ed25519 密钥对，连接 Gateway 时签名认证，本地连接自动配对 — 涉及文件: `src-tauri/src/device_identity.rs`
- connect.challenge 协议支持 — WS 连接后接收 Gateway 的 nonce 挑战，构建 v2 签名 payload — 涉及文件: `src-tauri/src/gateway/client.rs`

#### 变更 (Changed)
- Connect frame 设备认证 — `build_connect_frame` 新增 `DeviceAuthInfo` 参数，connect frame 包含 `device` 对象 (id, publicKey, signature, signedAt, nonce) — 涉及文件: `src-tauri/src/gateway/protocol.rs`
- Client ID 更新 — 从 `openclaw-control-ui` 改为 `clawx`，不再依赖 controlUi bypass — 涉及文件: `src-tauri/src/gateway/protocol.rs`
- WS 客户端 challenge-response — `do_connect` 新增挑战-响应流程：WS 连接 → 等待 nonce → 签名 → 发送带设备认证的 connect frame — 涉及文件: `src-tauri/src/gateway/client.rs`

#### 移除 (Removed)
- controlUi bypass — 移除 `ensure_control_ui_bypass()` 和 `dangerouslyDisableDeviceAuth` 配置写入 — 涉及文件: `src-tauri/src/gateway_lifecycle.rs`
- `read_config_raw`/`write_config_raw` — 移除仅供 bypass 使用的公开函数 — 涉及文件: `src-tauri/src/channel_config.rs`

#### 架构 (Architecture)
- 设备身份: Ed25519 密钥对持久化在 `clawx-device-identity.json`（0600 权限），deviceId = SHA-256(raw_public_key).hex
- 签名协议: v2 pipe-delimited payload `v2|{deviceId}|{clientId}|{clientMode}|{role}|{scopes}|{signedAtMs}|{token}|{nonce}`
- 本地自动配对: Gateway 检测到 loopback 连接后 silent auto-pair，无需手动审批
- 新增 Rust 依赖: ed25519-dalek 2, sha2 0.10, rand 0.8

---

### [0.2.0] - 2026-02-20

> Electron + Sidecar 完全移除，全面迁移至 Tauri 原生架构

#### 移除 (Removed)
- **Electron 运行时** — 删除 `electron/main/`（8 个文件）、`electron/preload/`（1 个文件）、`dist-electron/` 构建产物
- **Electron 依赖** — 移除 `electron`, `electron-builder`, `electron-store`, `electron-updater`, `vite-plugin-electron`, `vite-plugin-electron-renderer`, `sharp`, `jsdom`, `node-cron` 等 ~20 个依赖
- **Node.js Sidecar** — 删除 `electron/sidecar/`（4 个文件）、`electron/gateway/`（3 个文件）、`electron/utils/`（3 个文件）、`dist-sidecar/` 构建产物
- **Sidecar 依赖** — 移除 `ws`, `better-sqlite3`, `sqlite-vec`, `@huggingface/transformers`, `chokidar`, `mammoth`, `pdf-parse`, `turndown`, `@mozilla/readability`, `openclaw`, `clawhub` 等
- **SSE 事件流** — 删除 `src/lib/sse-client.ts`，所有事件改为 Tauri 原生 emit/listen
- **Electron Bridge** — 删除 `createElectronBridge()`、`isElectron()` 运行时检测
- **WhatsApp 集成** — 完全移除 WhatsApp channel（Baileys npm 无法 Rust 替代）：`electron/utils/whatsapp-login.ts`、前端 QR 登录流程、channel type 定义

#### 新增 (Added)
- **GatewayClient 推送事件** — Rust WS 客户端现在处理所有 Gateway 推送帧（chat、channel.status、agent 等），通过 `app.emit()` 转发给前端 — 涉及文件: `src-tauri/src/gateway/client.rs`
- **log:getRecent Rust 命令** — 替代 Sidecar 内存 ring buffer，读取日志文件末尾 N 行 — 涉及文件: `src-tauri/src/commands/log_cmd.rs`
- **事件名称映射** — `bridge.ts` 使用 `EVENT_MAP` 将前端通道名（如 `gateway:chat-message`）映射到 Rust 事件名（如 `gateway_chatMessage`）
- **Gateway WS 自动重连** — 断连后指数退避自动重连（基础 3s，最大 30s，最多 10 次），通过 Actor 命令通道触发避免 Send 约束问题 — 涉及文件: `src-tauri/src/gateway/client.rs`
- **桌面通知** — AI 回复完成时（`state == "final"`），若主窗口未聚焦，通过 `tauri-plugin-notification` 推送系统原生通知 — 涉及文件: `src-tauri/src/gateway/client.rs`

#### 变更 (Changed)
- **bridge.ts 全面简化** — 移除 SSE/Sidecar/Electron 路由，统一使用 Tauri invoke + listen — 涉及文件: `src/lib/bridge.ts`
- **vite.config.ts** — 移除 Electron 插件，纯 React + Vite 配置 — 涉及文件: `vite.config.ts`
- **tsconfig** — 移除 `@electron/*` 路径别名，`tsconfig.node.json` 不再包含 `electron/` 目录
- **gateway_lifecycle.rs** — `emit_status` 使用 `state` 字段（非 `status`）匹配前端 `GatewayStatus` 类型
- **settings_store.rs** — 新增 `save_gateway_token()` 持久化 Gateway 认证令牌到 `clawx-settings.json`，修复 `lazy_connect()` 因找不到 token 导致的 "Settings file not found" 错误
- **package.json** — 移除 `"main"` 入口、`build`/`package`/`release`/`sidecar:*` 脚本

#### 修复 (Fixed)
- **Gateway Token 持久化** — 修复 Gateway 启动时生成的认证令牌未写入 `clawx-settings.json` 导致 WS RPC 调用失败的问题 — 涉及文件: `src-tauri/src/settings_store.rs`, `src-tauri/src/gateway_lifecycle.rs`
- **shell:showItemInFolder** — 修复 Setup 和 Settings 页面中 `invoke('shell:showItemInFolder')` 未迁移到 `@tauri-apps/plugin-opener` 的问题 — 涉及文件: `src/pages/Setup/index.tsx`, `src/pages/Settings/index.tsx`
- **dialog:open** — 修复 Spotlight 文件选择对话框未迁移到 `@tauri-apps/plugin-dialog` 的问题 — 涉及文件: `src/stores/spotlight.ts`
- **update store** — 完整重写更新模块，从 `invoke('update:*')` 迁移到 `@tauri-apps/plugin-updater` 原生 API — 涉及文件: `src/stores/update.ts`

#### 架构 (Architecture)
- **零 Node.js 进程** — 应用启动后不再有任何 Node.js 子进程（原 Sidecar + Gateway 的 Node 进程均已移除）
- **123 个 Tauri Rust 命令** + **9 个 Tauri 原生事件** 处理全部业务逻辑
- **进程模型**: 2 个进程 — Tauri 主进程（Rust + WebView）+ Gateway（Python/OpenClaw）
- **WS 自动重连**: Actor 命令通道模式，断连时 read task 通过 `cmd_tx` 发送 `Reconnect` 命令到 actor 主循环，避免 `do_connect` future 的 Send 约束问题

---

### [0.1.13] - 2026-02-18

> 当前版本（基线）。以下为项目现有功能的基线记录。

#### 已有功能基线
- AI 多模型对话（Chat 页面）— 支持流式响应、工具调用状态、图片附件
- 多通道消息集成（Channels 页面）— Telegram, Discord, WhatsApp, 飞书等
- 技能市场（Skills 页面）— ClawHub CLI 集成，搜索/安装/卸载
- Cron 定时任务（Cron 页面）— 创建/编辑/切换/触发/删除
- AI Provider 管理（Settings 页面）— 多 Provider 配置，API Key 验证
- 自动更新 — GitHub Releases + 阿里云 OSS
- 系统托盘 — 关闭窗口最小化到托盘
- 多语言支持 — 英文、中文、日文
- 主题切换 — 亮色/暗色/跟随系统
- 开发者模式 — 内嵌 OpenClaw Control UI

---

### [0.2.0] - 2026-02-19

> Phase 2 P0: 桌面助手核心能力 — 全局快捷键 + Spotlight 窗口 + 剪贴板智能

#### 新增 (Added)
- [F-2.1] 全局快捷键呼出 — 默认 `CommandOrControl+Shift+Space` 弹出 Spotlight 窗口，支持 Settings 页面自定义录制 — 涉及文件: `electron/main/global-shortcut.ts`, `electron/utils/store.ts`
- [F-2.2] Spotlight 悬浮快捷窗口 — 680×480 frameless/transparent/alwaysOnTop 窗口，macOS 毛玻璃效果，多屏支持，blur 自动隐藏，framer-motion 入场动画 — 涉及文件: `electron/main/spotlight.ts`, `src/pages/Spotlight/index.tsx`, `src/pages/Spotlight/SpotlightInput.tsx`, `src/pages/Spotlight/SpotlightResponse.tsx`
- [F-2.3] 剪贴板智能处理 — 自动检测 Text/URL/Code/Image 类型，提供翻译/摘要/解释/代码审查等快捷操作 — 涉及文件: `src/utils/clipboard-detect.ts`, `src/pages/Spotlight/ClipboardBar.tsx`, `src/types/spotlight.ts`
- Spotlight Zustand Store — 管理消息、流式响应、剪贴板状态 — 涉及文件: `src/stores/spotlight.ts`
- Spotlight i18n — 英/中/日三语言 — 涉及文件: `src/i18n/locales/{en,zh,ja}/spotlight.json`
- Settings 快捷键配置 — Desktop Assistant Card 支持键盘录制模式更新快捷键 — 涉及文件: `src/pages/Settings/index.tsx`, `src/i18n/locales/{en,zh,ja}/settings.json`
- Tray Quick Chat 菜单项 — 系统托盘新增 Quick Chat 入口 — 涉及文件: `electron/main/tray.ts`

#### 变更 (Changed)
- Gateway 事件双窗口转发 — 所有 Gateway 事件同时转发给主窗口和 Spotlight 窗口 — 涉及文件: `electron/main/ipc-handlers.ts`
- Preload 白名单扩展 — 新增 7 个 invoke 通道 (`spotlight:*`, `clipboard:*`, `shortcut:*`) 和 2 个 on 通道 (`spotlight:shown/hidden`) — 涉及文件: `electron/preload/index.ts`
- 路由增加 Spotlight — `/spotlight` 路由在 MainLayout 外部，跳过 Setup 重定向 — 涉及文件: `src/App.tsx`

#### 架构 (Architecture)
- 共享 Renderer + Hash Route — Spotlight 窗口复用 `dist/index.html`，通过 `#/spotlight` 路由区分，无需新 preload 脚本
- 每个 BrowserWindow 有独立 JS 上下文，Zustand store 实例隔离

---

### [0.2.1] - 2026-02-19

> Phase 2 P1: F-2.4 屏幕截图分析

#### 新增 (Added)
- [F-2.4] 屏幕截图分析 — Spotlight 窗口中截取屏幕区域发送给 AI 分析（macOS `screencapture -i`，Linux gnome-screenshot/scrot） — 涉及文件: `electron/utils/screenshot.ts`, `electron/main/ipc-handlers.ts`, `electron/preload/index.ts`, `src/stores/spotlight.ts`, `src/pages/Spotlight/SpotlightInput.tsx`, `src/i18n/locales/{en,zh,ja}/spotlight.json`

#### 变更 (Changed)
- Spotlight Store 扩展 — 新增 `screenshotPreview`/`isCapturing` 状态和 `takeScreenshot`/`clearScreenshot` 动作；`sendMessage` 支持截图附件通过 `chat:sendWithMedia` IPC 发送 — 涉及文件: `src/stores/spotlight.ts`
- SpotlightInput UI 增强 — 新增 Camera 截图按钮、截图进行中旋转动画、截图预览缩略图及删除按钮 — 涉及文件: `src/pages/Spotlight/SpotlightInput.tsx`
- Preload 白名单扩展 — 新增 `screenshot:capture`, `screenshot:checkPermission` 通道 — 涉及文件: `electron/preload/index.ts`

---

### [0.2.2] - 2026-02-19

> Phase 2 P1: F-2.5 快捷指令系统

#### 新增 (Added)
- [F-2.5] 快捷指令系统 — 输入 "/" 弹出命令面板，支持 10 个内置指令（翻译/摘要/解释/代码审查/优化/改写/修正语法/提问等），分 4 类显示，支持模糊搜索和键盘导航 — 涉及文件: `src/types/commands.ts`, `src/data/builtin-commands.ts`, `src/utils/command-engine.ts`, `src/pages/Spotlight/CommandPalette.tsx`

#### 变更 (Changed)
- Spotlight Store 扩展 — 新增 `commandPalette`/`pendingCommand` 状态和 7 个命令面板 actions；`clearConversation` 同步清除命令状态 — 涉及文件: `src/stores/spotlight.ts`
- SpotlightInput 增强 — 集成 "/" 检测触发命令面板、ArrowDown/Up/Enter/Tab/Escape 键盘导航、pendingCommand 模式显示指令标签和动态占位符 — 涉及文件: `src/pages/Spotlight/SpotlightInput.tsx`
- Spotlight i18n 扩展 — 新增 `commands.*` 命名空间，包含 4 个分类标题和 10 组指令的名称/描述/输入占位符 — 涉及文件: `src/i18n/locales/{en,zh,ja}/spotlight.json`

---

### [0.2.3] - 2026-02-19

> Phase 2 P2: F-2.6 本地文件快速检索

#### 新增 (Added)
- [F-2.6] 本地文件快速检索 — 两种触发方式：📎 按钮（系统文件对话框）和 `@` 前缀（inline 文件搜索面板）；选中文件显示为 attachment chips；发送时读取文件内容内联到 prompt — 涉及文件: `src/types/file-attachment.ts`, `electron/utils/file-search.ts`, `src/utils/file-search-engine.ts`, `src/pages/Spotlight/FileSearchPanel.tsx`, `src/pages/Spotlight/FileAttachmentBar.tsx`

#### 变更 (Changed)
- IPC Handlers 扩展 — 新增 `registerFileSearchHandlers()` 注册 `filesearch:search` 和 `filesearch:readContent` 两个 handler — 涉及文件: `electron/main/ipc-handlers.ts`
- Preload 白名单扩展 — 新增 `filesearch:search`, `filesearch:readContent` 通道 — 涉及文件: `electron/preload/index.ts`
- Spotlight Store 扩展 — 新增 `fileAttachments`/`fileSearch` 状态和 10 个文件附件 actions；`sendMessage` 发送前读取所有附件文件内容内联到消息；`clearConversation` 同步清除文件状态 — 涉及文件: `src/stores/spotlight.ts`
- SpotlightInput 增强 — 新增 📎 附加文件按钮、`@` 前缀检测触发文件搜索面板、文件搜索键盘导航(ArrowDown/Up/Enter/Escape)、FileAttachmentBar 和 FileSearchPanel 集成；`canSend` 增加文件附件条件 — 涉及文件: `src/pages/Spotlight/SpotlightInput.tsx`
- Spotlight i18n 扩展 — 新增 `fileSearch.*` 命名空间（attachFile/searching/noResults/hint） — 涉及文件: `src/i18n/locales/{en,zh,ja}/spotlight.json`

#### 架构 (Architecture)
- 文件搜索利用系统原生工具：macOS `mdfind` (Spotlight 索引) / Linux `find`，通过 `execFile` 防止命令注入
- 安全约束：路径限制在 homedir 下（`isPathSafe`）、单文件 ≤ 50KB、总计 ≤ 200KB、二进制文件拒绝读取、搜索超时 5 秒

---

### [0.2.4] - 2026-02-19

> Phase 2 P2: F-2.7 系统通知集成（Phase 2 最后一个功能）

#### 新增 (Added)
- [F-2.7] 系统通知集成 — AI 回复完成时通过系统原生通知推送给用户（仅在所有窗口均未聚焦时触发）；通知标题 "ClawX"，内容为回复前 100 字符；点击通知聚焦主窗口；Settings 中可开关 — 涉及文件: `electron/utils/notification.ts`, `electron/main/ipc-handlers.ts`, `electron/utils/store.ts`, `src/stores/settings.ts`, `src/pages/Settings/index.tsx`, `src/i18n/locales/{en,zh,ja}/settings.json`

#### 变更 (Changed)
- Gateway chat:message 事件处理增强 — 在转发事件给渲染进程后调用 `notifyChatComplete` 触发系统通知 — 涉及文件: `electron/main/ipc-handlers.ts`
- Settings Store 扩展 — 新增 `enableNotifications` 状态和 `setEnableNotifications` setter — 涉及文件: `src/stores/settings.ts`
- Settings UI 增强 — Desktop Assistant Card 新增系统通知开关（Bell icon + Switch） — 涉及文件: `src/pages/Settings/index.tsx`
- Settings i18n 扩展 — `spotlight` 命名空间新增 `notificationsLabel`/`notificationsDesc` — 涉及文件: `src/i18n/locales/{en,zh,ja}/settings.json`

---

### [0.4.0] - 2026-02-19

> Phase 3: 知识库 & RAG — 让 Agent 拥有专属知识库，实现检索增强生成

#### 新增 (Added)
- [F-3.1] 知识库 CRUD — 创建/编辑/删除知识库，支持配置 embedding 模型和分块参数 — 涉及文件: `electron/utils/knowledge-storage.ts`, `src/stores/knowledge.ts`, `src/pages/Knowledge/index.tsx`, `src/pages/Knowledge/KnowledgeCard.tsx`, `src/pages/Knowledge/KnowledgeEditor.tsx`
- [F-3.2] 文档上传与解析 — 支持 PDF, DOCX, TXT, MD, CSV 上传，自动解析文本 — 涉及文件: `electron/utils/document-parser.ts`, `src/pages/Knowledge/DocumentUpload.tsx`, `src/pages/Knowledge/KnowledgeDetail.tsx`
- [F-3.3] 文本分块与向量化 — 递归文本分割 + 混合 Embedding（Provider API 优先 + 本地 all-MiniLM-L6-v2 回退），SQLite + sqlite-vec 向量存储 — 涉及文件: `electron/utils/text-chunker.ts`, `electron/utils/embedding.ts`, `electron/utils/vector-db.ts`, `electron/utils/knowledge-pipeline.ts`
- [F-3.4] Agent 关联知识库 — Agent 编辑器支持多选 Checkbox 关联知识库 — 涉及文件: `src/pages/Agents/AgentEditor.tsx`, `src/i18n/locales/{en,zh,ja}/agents.json`
- [F-3.5] RAG 检索增强 — Chat 和 Spotlight 发送消息前自动检索关联 KB，以 `<knowledge>` 块注入上下文 — 涉及文件: `electron/utils/rag-engine.ts`, `src/stores/chat.ts`, `src/stores/spotlight.ts`
- [F-3.6] 知识库搜索预览 — KB 详情页内搜索，显示相关片段和相似度分数 — 涉及文件: `src/pages/Knowledge/KnowledgeSearch.tsx`
- [F-3.7] 网页抓取入库 — 粘贴 URL 自动抓取正文（Readability + Turndown HTML→MD）加入知识库 — 涉及文件: `electron/utils/web-scraper.ts`
- [F-3.8] 本地文件夹监听 — 监听指定文件夹，自动同步 .pdf/.docx/.txt/.md/.csv 到知识库（chokidar + 500ms 防抖），启动时自动恢复 — 涉及文件: `electron/utils/folder-watcher.ts`, `electron/main/index.ts`
- 知识库 Zustand Store — 管理 KB/文档/搜索状态 — 涉及文件: `src/stores/knowledge.ts`
- 知识库 i18n — 英/中/日三语言 — 涉及文件: `src/i18n/locales/{en,zh,ja}/knowledge.json`
- 知识库类型定义 — KnowledgeBase, KnowledgeDocument, KnowledgeChunk, RAGResult, RAGConfig — 涉及文件: `src/types/knowledge.ts`

#### 变更 (Changed)
- IPC Handlers 扩展 — 新增 `registerKnowledgeHandlers()` 注册 16 个 `knowledge:*` IPC 通道 — 涉及文件: `electron/main/ipc-handlers.ts`
- Preload 白名单扩展 — 新增 16 个 knowledge invoke 通道 + 1 个 on 通道 `knowledge:documentProgress` — 涉及文件: `electron/preload/index.ts`
- 路由增加 Knowledge — `/knowledge` 和 `/knowledge/:id` 路由 — 涉及文件: `src/App.tsx`
- 侧边栏增加 Knowledge 入口 — BookOpen 图标 — 涉及文件: `src/components/layout/Sidebar.tsx`
- i18n 注册 knowledge 命名空间 — 涉及文件: `src/i18n/index.ts`, `src/i18n/locales/{en,zh,ja}/common.json`
- 应用生命周期增强 — 启动时恢复文件夹 watchers，退出时 stopAll — 涉及文件: `electron/main/index.ts`

#### 架构 (Architecture)
- 向量存储：SQLite + sqlite-vec 扩展，每个 KB 拥有独立 `vec_chunks_{kbId}` 虚拟表，支持可变维度
- 混合 Embedding：Provider API 路径（OpenAI 兼容 `/embeddings` 端点）+ 本地 ONNX 路径（@huggingface/transformers + Xenova/all-MiniLM-L6-v2, 384 维）
- 元数据与向量分离：元数据在 electron-store (`clawx-knowledge`)，重数据（chunks + embeddings）在 SQLite
- 文档处理管线：parse → chunk → embed (batch 32) → insert → 状态跟踪
- 新增 npm 依赖：better-sqlite3, sqlite-vec, pdf-parse, mammoth, @huggingface/transformers, turndown, @mozilla/readability, chokidar

---

### [0.5.0] - 2026-02-19

> Phase 4: Agent 协作 & 工作流编排 — 多 Agent 可视化协作与自动化任务链

#### 新增 (Added)
- [F-4.1] Agent 间消息传递 — Agent A 的输出可以作为 Agent B 的输入，通过 DAG 执行引擎逐层传递 — 涉及文件: `electron/utils/workflow-engine.ts`
- [F-4.2] 任务链编排 — @xyflow/react 可视化节点画布，5 种自定义节点（Input/Output/Agent/Condition/Merge），拖拽创建 + 连线 + 右侧配置面板 — 涉及文件: `src/pages/Workflows/WorkflowEditor.tsx`, `src/pages/Workflows/nodes/*.tsx`, `src/pages/Workflows/nodes/NodeConfigPanel.tsx`
- [F-4.3] 条件路由 — Keyword / Regex / AI 分类三种条件类型，匹配后沿对应 handle 的 edge 传递 — 涉及文件: `src/pages/Workflows/nodes/ConditionNode.tsx`, `electron/utils/workflow-engine.ts`
- [F-4.4] 并行执行 — Kahn 拓扑排序 + 同层级节点 Promise.all 并发，Merge 节点等待所有入边完成后合并（concat/first/custom） — 涉及文件: `electron/utils/workflow-engine.ts`, `src/pages/Workflows/nodes/MergeNode.tsx`
- [F-4.5] 自动化触发器 — Cron（node-cron）/ 文件变化（chokidar）/ 剪贴板（2s 轮询 + regex）/ 快捷键（globalShortcut）四种触发器 — 涉及文件: `electron/utils/workflow-triggers.ts`, `src/pages/Workflows/WorkflowTriggerEditor.tsx`
- [F-4.6] 执行日志与回放 — SQLite 存储执行记录 (`~/.openclaw/workflows/runs.db`)，步骤级 input/output 回看，运行历史 Dialog — 涉及文件: `electron/utils/workflow-run-db.ts`, `src/pages/Workflows/WorkflowRunPanel.tsx`, `src/pages/Workflows/WorkflowRunHistory.tsx`
- [F-4.7] 任务链模板市场 — 6 个内置模板（翻译/代码审查/内容创作/研究助手/邮件分类/每日摘要）+ JSON 导入导出 — 涉及文件: `src/data/workflow-templates.ts`, `src/pages/Workflows/WorkflowTemplateGallery.tsx`
- 工作流配置持久化 — electron-store (`clawx-workflows`) — 涉及文件: `electron/utils/workflow-storage.ts`
- 工作流 Zustand Store — 管理工作流/运行/进度状态 — 涉及文件: `src/stores/workflow.ts`
- 工作流类型定义 — WorkflowConfig, WorkflowNode, WorkflowEdge, WorkflowTrigger, WorkflowRun, StepResult, WorkflowTemplate — 涉及文件: `src/types/workflow.ts`
- 工作流 i18n — 英/中/日三语言 — 涉及文件: `src/i18n/locales/{en,zh,ja}/workflows.json`

#### 变更 (Changed)
- IPC Handlers 扩展 — 新增 `registerWorkflowHandlers()` 注册 18 个 `workflow:*` IPC 通道 — 涉及文件: `electron/main/ipc-handlers.ts`
- Preload 白名单扩展 — 新增 18 个 workflow invoke 通道 + 1 个 on 通道 `workflow:stepProgress` — 涉及文件: `electron/preload/index.ts`
- 路由增加 Workflows — `/workflows` 和 `/workflows/:id` 路由 — 涉及文件: `src/App.tsx`
- 侧边栏增加 Workflows 入口 — GitBranch 图标 — 涉及文件: `src/components/layout/Sidebar.tsx`
- i18n 注册 workflows 命名空间 — 涉及文件: `src/i18n/index.ts`, `src/i18n/locales/{en,zh,ja}/common.json`

#### 架构 (Architecture)
- 执行引擎：WorkflowEngine extends EventEmitter，DAG 验证（1 input, ≥1 output, 无环检测）+ Kahn 拓扑排序 + 逐层并行执行
- 通过 `gatewayManager.rpc('chat.send', { sessionKey, message, deliver: false })` 调用 Gateway 执行 Agent 对话
- 触发器管理：WorkflowTriggerManager 类统一管理四种触发器生命周期，启动时自动恢复
- 新增 npm 依赖：@xyflow/react, node-cron, @types/node-cron

---

### [0.6.0-alpha.1] - 2026-02-19

> Phase 5 Week 1: Tauri 2 迁移准备 — Bridge 抽象层 + Tauri 2 骨架 + IPC 解耦

#### 新增 (Added)
- Bridge 抽象层 — 统一 Electron/Tauri IPC 接口，运行时自动检测桥接模式 — 涉及文件: `src/lib/bridge.ts`
- Tauri 2 项目骨架 — Cargo.toml + tauri.conf.json + 双窗口（main + spotlight）+ capabilities 权限系统 — 涉及文件: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/*.json`
- Tauri Rust Commands — 12 个原生命令（窗口操作/剪贴板/快捷键/Spotlight/应用信息）+ 系统托盘 + Spotlight 窗口管理 — 涉及文件: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`
- Vite Renderer-only 模式 — `pnpm dev:renderer` 和 `pnpm build:renderer` 脚本供 Tauri 使用，跳过 Electron 插件 — 涉及文件: `vite.config.ts`, `package.json`

#### 变更 (Changed)
- 渲染进程全面解耦 — 所有 26 个 `src/` 文件（11 stores + 15 pages/components）从 `window.electron.*` 迁移到 `invoke/on/openExternal/getPlatform/getIsDev` bridge API — 涉及文件: `src/stores/*.ts`, `src/pages/**/*.tsx`, `src/components/layout/*.tsx`, `src/App.tsx`

#### 架构 (Architecture)
- Bridge 模式：Electron 侧直接包装 `ipcRenderer`，Tauri 侧将 native channels（`window:*/dialog:*/shell:*/shortcut:*/clipboard:*/screenshot:*/spotlight:*/app:*`）路由到 Tauri commands，其余 channels 代理到 Node.js Sidecar HTTP 端点
- 双运行时兼容：`pnpm dev` = Electron + Vite，`pnpm tauri:dev` = Tauri + Vite (renderer-only)
- Tauri Capabilities：main-window 全权限，spotlight-window 最小权限
- 新增 npm 依赖：@tauri-apps/api, @tauri-apps/plugin-opener, @tauri-apps/cli

---

### [0.6.0-alpha.2] - 2026-02-19

> Phase 5 Week 2: Node.js Sidecar HTTP 服务器 — 让 Tauri 模式下的业务逻辑通过 HTTP 代理运行

#### 新增 (Added)
- 运行时抽象层 — `runtime.ts` 提供 `isElectronMain()/isSidecar()/getUserDataPath()/isPackaged()/getAppVersion()` 等统一 API，自动检测 Electron 或 Sidecar 环境 — 涉及文件: `electron/utils/runtime.ts`
- Sidecar HTTP 服务器 — Node.js 原生 `http` 模块，端口 18790，支持 `POST /ipc/{channel}`、`GET /events` (SSE)、`GET /health` — 涉及文件: `electron/sidecar/server.ts`
- Sidecar 路由映射 — 将 ~108 个 IPC 通道映射为 HTTP 路由，复用全部 `electron/utils/*` 业务逻辑 — 涉及文件: `electron/sidecar/router.ts`
- SSE 事件推送 — `SSEManager` 替代 `mainWindow.webContents.send()`，支持多客户端连接 — 涉及文件: `electron/sidecar/events.ts`
- SSE 客户端 — 渲染进程侧 EventSource 封装，自动重连 — 涉及文件: `src/lib/sse-client.ts`
- Sidecar 构建脚本 — esbuild 打包为 `dist-sidecar/server.js` 单文件 — 涉及文件: `electron/sidecar/build.ts`

#### 变更 (Changed)
- 6 个 Electron 工具模块解耦 — `paths.ts`/`logger.ts`/`openclaw-cli.ts`/`uv-env.ts`/`uv-setup.ts` 从 `import { app } from 'electron'` 改为 `import { ... } from './runtime'` — 涉及文件: `electron/utils/paths.ts`, `electron/utils/logger.ts`, `electron/utils/openclaw-cli.ts`, `electron/utils/uv-env.ts`, `electron/utils/uv-setup.ts`
- GatewayManager 解耦 — `app.isPackaged`/`app.getName()` → `runtime.isPackaged()`/`runtime.getAppName()`，Sidecar 模式跳过 Electron Helper 和 ELECTRON_RUN_AS_NODE — 涉及文件: `electron/gateway/manager.ts`
- ClawHub 解耦 — `app.isPackaged`/`shell.openPath()` → `runtime.isPackaged()`/`runtime.openPath()` — 涉及文件: `electron/gateway/clawhub.ts`
- notification.ts Sidecar 适配 — Sidecar 模式通过 SSE 推送通知事件（Tauri Rust 侧显示） — 涉及文件: `electron/utils/notification.ts`
- screenshot.ts Sidecar 适配 — 无 nativeImage 时 fallback 到原始 base64 — 涉及文件: `electron/utils/screenshot.ts`
- workflow-triggers.ts Sidecar 适配 — clipboard 用 pbpaste/xclip fallback，globalShortcut 仅 Electron 模式注册 — 涉及文件: `electron/utils/workflow-triggers.ts`
- Bridge SSE 增强 — Tauri bridge 的 `on()` 方法对非原生通道使用 SSE 而非 Tauri events — 涉及文件: `src/lib/bridge.ts`
- Tauri lib.rs 增强 — 启动时通过 `tauri-plugin-shell` spawn Node.js sidecar 进程 — 涉及文件: `src-tauri/src/lib.rs`
- package.json 新增脚本 — `sidecar:dev`/`sidecar:build` — 涉及文件: `package.json`

#### 架构 (Architecture)
- 三层架构：Tauri Rust Shell (12 原生命令) → Node.js Sidecar HTTP (108 业务通道) → OpenClaw Gateway WS (port 18789)
- 事件推送：Electron 模式用 `webContents.send()`，Sidecar 模式用 SSE，渲染进程通过 bridge 层统一订阅
- 运行时抽象：`isElectronMain()` 检测 → Electron 优先使用原生 API → Sidecar fallback 到 env + Node.js stdlib
- 零新依赖：Sidecar 使用 Node.js 原生 `http` 模块，无需 Express/Fastify

---

### [0.6.0-alpha.3] - 2026-02-20

> Phase 5 Week 3: 纯逻辑模块 Rust 重写 — Agent/Knowledge/Workflow 存储 CRUD + Provider Registry + Text Chunker

#### 新增 (Added)
- Rust JSON 文件存储抽象 — `JsonStore<T>` 通用读写，原子写入（temp file + rename），匹配 electron-store 行为 — 涉及文件: `src-tauri/src/storage/mod.rs`
- Rust Agent 存储 — `AgentConfig` CRUD + 默认 Agent 自动创建 + 导入导出 — 涉及文件: `src-tauri/src/storage/agents.rs`
- Rust Knowledge 存储 — `KnowledgeBaseConfig` + `DocumentConfig` CRUD + 级联删除 + 统计刷新 — 涉及文件: `src-tauri/src/storage/knowledge.rs`
- Rust Workflow 存储 — `WorkflowConfig` CRUD + 复制 + 导入导出，完整类型树（Node/Edge/Trigger） — 涉及文件: `src-tauri/src/storage/workflows.rs`
- 28 个新 Tauri 命令 — 9 个 `agent:*` + 11 个 `knowledge:*` + 8 个 `workflow:*` CRUD 命令 — 涉及文件: `src-tauri/src/commands/agents_cmd.rs`, `src-tauri/src/commands/knowledge_cmd.rs`, `src-tauri/src/commands/workflows_cmd.rs`
- Rust Provider Registry — 11 个 provider 元数据（env var + default model + OpenClaw config），内部模块暂不暴露命令 — 涉及文件: `src-tauri/src/providers.rs`
- Rust Text Chunker — 递归字符分割 + 重叠分块算法，完整移植 TS 版本，含单元测试 — 涉及文件: `src-tauri/src/text_chunker.rs`
- 应用常量模块 — Gateway/Sidecar 端口、Store 文件名、默认值等 — 涉及文件: `src-tauri/src/config.rs`

#### 变更 (Changed)
- Bridge 存储通道路由 — 新增 `STORAGE_PREFIXES`（`agent:` / `knowledge:` / `workflow:`），位置参数包装为 `{ _args: [...] }` 发送给 Tauri invoke — 涉及文件: `src/lib/bridge.ts`
- lib.rs 扩展 — 注册 28 个新命令（12 → 40），AppState 新增 `data_dir: PathBuf`，setup 阶段初始化数据目录 — 涉及文件: `src-tauri/src/lib.rs`
- Cargo.toml 新增依赖 — uuid v1 (v4), chrono v0.4 (serde), tempfile v3 (dev) — 涉及文件: `src-tauri/Cargo.toml`
- 修复 Sidecar spawn API — `tauri-plugin-shell` v2 的 `spawn()` 返回 `(Receiver, CommandChild)` 元组，destructure 修复 — 涉及文件: `src-tauri/src/lib.rs`

#### 架构 (Architecture)
- 存储通道路径变化：`Renderer → Tauri invoke → Rust 命令 → serde_json → JSON 文件`（替代 `Renderer → HTTP fetch → Sidecar → electron-store`）
- 通道分类：40 个 Rust 原生命令 + ~80 个 Sidecar HTTP 业务通道 + Gateway WS
- 位置参数协议：存储通道使用 `{ _args: Vec<serde_json::Value> }` 包装，Rust 按位置提取
- 响应格式兼容：Rust `json!({ "success": true, ... })` 与 TS 前端期望一致
- 26 个单元测试覆盖所有 CRUD + text_chunker + provider registry

---

### [0.6.0-alpha.4] - 2026-02-20

> Phase 5 Week 4: 数据库 + 搜索模块 Rust 重写 — SQLite 向量搜索 + Embedding API + RAG 引擎 + Workflow Run DB

#### 新增 (Added)
- Rust SQLite 数据库管理 — `DbManager` 懒初始化双 DB 连接（knowledge.db + runs.db），WAL 模式 + busy_timeout 5000ms，embedding BLOB 列迁移 — 涉及文件: `src-tauri/src/database/mod.rs`
- Rust 向量数据库 — 纯 Rust 余弦相似度搜索（替代 sqlite-vec 虚拟表），embedding 存储为 f32 little-endian BLOB，chunks CRUD + KB 记录管理 — 涉及文件: `src-tauri/src/database/vector_db.rs`
- Rust Workflow Run DB — 4 个读/删命令（getRuns/getRun/deleteRun/clearRuns），与 TS better-sqlite3 共享同一 runs.db — 涉及文件: `src-tauri/src/database/workflow_runs.rs`
- Rust Embedding API 客户端 — reqwest 调用 OpenAI 兼容 /embeddings 端点，batch size 32，支持 `provider:default` 和 `provider:<id>` 路由 — 涉及文件: `src-tauri/src/embedding.rs`
- Rust RAG 引擎 — 按 embedding model 分组 KB → 嵌入查询 → 向量搜索 → 合并 top_k → token 截断 — 涉及文件: `src-tauri/src/rag_engine.rs`
- Rust Secure Storage 读取器 — 只读访问 `clawx-providers.json`（provider 配置 + API key），供 Embedding 客户端使用 — 涉及文件: `src-tauri/src/secure_storage.rs`
- 6 个新 Tauri 命令 — `knowledge:search`, `knowledge:rag`, `workflow:getRuns`, `workflow:getRun`, `workflow:deleteRun`, `workflow:clearRuns` — 涉及文件: `src-tauri/src/commands/knowledge_cmd.rs`, `src-tauri/src/commands/workflows_cmd.rs`

#### 变更 (Changed)
- Bridge SIDECAR_OVERRIDES — 新增 14 个通道白名单强制走 Sidecar（尚未迁移的 knowledge pipeline + workflow runtime），修复前缀路由导致非 CRUD 通道在 Tauri 模式失败的关键 Bug — 涉及文件: `src/lib/bridge.ts`
- Knowledge create/delete 增强 — create 同步创建 SQLite KB 记录，delete 先清理 SQLite 数据再删 JSON — 涉及文件: `src-tauri/src/commands/knowledge_cmd.rs`
- AppState 扩展 — 新增 `db_manager: DbManager` 字段 — 涉及文件: `src-tauri/src/lib.rs`
- lib.rs 命令注册 — 40 → 46 个命令 — 涉及文件: `src-tauri/src/lib.rs`
- Cargo.toml 新增依赖 — rusqlite v0.31 (bundled), reqwest v0.12 (json+rustls-tls), dirs v5, byteorder v1 — 涉及文件: `src-tauri/Cargo.toml`

#### 架构 (Architecture)
- 知识库搜索链路变化：`Renderer → Tauri invoke → Rust → reqwest(embedding) → rusqlite(cosine search) → 结果`（替代 `Renderer → HTTP → Sidecar → embedding.ts → vector-db.ts → SQLite`）
- SQLite 共享策略：Rust（rusqlite bundled）和 Sidecar（better-sqlite3）共享同一 DB 文件，WAL 模式并发读写
- 向量搜索策略：不使用 sqlite-vec C 扩展，改为加载 KB 全部 chunk embedding 到内存计算余弦相似度（适用于 < 100k chunks）
- 通道分类更新：46 个 Rust 原生命令 + ~74 个 Sidecar HTTP 业务通道 + Gateway WS
- 48 个单元测试覆盖 DB 初始化、CRUD、余弦相似度、f32 序列化、secure storage、embedding 客户端

---

### [0.6.0-alpha.5] - 2026-02-20

> Phase 5 Week 5: 网络模块 Rust 重写 — Gateway WS 客户端 + Cron + Channel Config + Chat

#### 新增 (Added)
- Rust Settings Store 读取器 — 只读访问 `clawx-settings.json` 获取 gatewayToken — 涉及文件: `src-tauri/src/settings_store.rs`
- Rust Gateway Protocol v3 — OpenClaw 协议帧类型定义（Request/Response/Event），connect 握手帧构建 — 涉及文件: `src-tauri/src/gateway/protocol.rs`
- Rust Gateway WebSocket 客户端 — Actor 模式 (`tokio-tungstenite` + `mpsc`/`oneshot`)，独立 WS 连接用于 RPC 调用，惰性连接（首次 RPC 自动连接），断线标记 + 重连 — 涉及文件: `src-tauri/src/gateway/client.rs`
- Rust Channel Config CRUD — 读写 `~/.openclaw/openclaw.json`，Discord guilds 嵌套结构转换，Telegram allowFrom 数组转换，WhatsApp 插件通道处理 — 涉及文件: `src-tauri/src/channel_config.rs`
- 18 个新 Tauri 命令 — 5 个 `gateway:*` + 6 个 `cron:*` + 6 个 `channel:*` + 1 个 `chat:sendWithMedia` — 涉及文件: `src-tauri/src/commands/gateway_cmd.rs`, `src-tauri/src/commands/cron_cmd.rs`, `src-tauri/src/commands/channel_cmd.rs`, `src-tauri/src/commands/chat_cmd.rs`

#### 变更 (Changed)
- Bridge 路由扩展 — `STORAGE_PREFIXES` 新增 `gateway:`, `cron:`, `channel:`, `chat:` 前缀；`SIDECAR_OVERRIDES` 新增 4 个 Gateway 生命周期 + 4 个 Channel 验证通道 — 涉及文件: `src/lib/bridge.ts`
- Bridge `on()` 事件路由修复 — 仅 `NATIVE_PREFIXES` 使用 Tauri listen，所有 storage-prefixed 事件走 SSE（修复 gateway/cron/channel 事件丢失 Bug） — 涉及文件: `src/lib/bridge.ts`
- AppState 扩展 — 新增 `gateway_client: GatewayClient` 字段 — 涉及文件: `src-tauri/src/lib.rs`
- lib.rs 命令注册 — 46 → 64 个命令 — 涉及文件: `src-tauri/src/lib.rs`
- commands/mod.rs 扩展 — 新增 gateway_cmd, cron_cmd, channel_cmd, chat_cmd 模块 — 涉及文件: `src-tauri/src/commands/mod.rs`
- Cargo.toml 新增依赖 — tokio-tungstenite v0.21 (rustls-tls), futures-util v0.3, base64 v0.22 — 涉及文件: `src-tauri/Cargo.toml`

#### 架构 (Architecture)
- 混合方案：Sidecar 继续管理 Gateway 进程生命周期（start/stop/restart），Rust 建立独立 WS 连接用于 RPC 调用，两个 WS 连接共存
- Cron 链路变化：`Renderer → Tauri invoke → Rust WS → Gateway → response → Rust → Renderer`（替代 `Renderer → HTTP → Sidecar → Sidecar WS → Gateway`）
- Channel Config 链路变化：`Renderer → Tauri invoke → Rust fs → ~/.openclaw/openclaw.json`（替代 `Renderer → HTTP → Sidecar → fs`）
- 事件路由策略：invoke 路由到 Rust 命令，on 事件始终走 SSE（Sidecar GatewayManager 发出），避免事件丢失
- 通道分类更新：64 个 Rust 原生命令 + ~56 个 Sidecar HTTP 业务通道 + Gateway WS
- 55 个单元测试覆盖 protocol 帧序列化、settings_store、channel_config、DB 等

---

### [0.6.0-alpha.6] - 2026-02-20

> Phase 5 Week 6: Provider + Utility 模块 Rust 重写 — Provider CRUD + API Key 验证 + Skill Config + OpenClaw CLI + Log + File Staging + FileSearch + UV

#### 新增 (Added)
- Rust secure_storage 扩展为完整 CRUD — 原子写入（temp file + rename），save_provider/delete_provider/store_api_key/delete_api_key/set_default_provider/get_all_providers_with_key_info — 涉及文件: `src-tauri/src/secure_storage.rs`
- Rust OpenClaw Auth Profiles — 写入 `~/.openclaw/agents/main/agent/auth-profiles.json` 和 `~/.openclaw/openclaw.json`，同步 API key 和 default model 配置 — 涉及文件: `src-tauri/src/openclaw_auth.rs`
- Rust OpenClaw 路径解析 — 检测 OpenClaw 包位置、版本、构建状态 — 涉及文件: `src-tauri/src/openclaw_paths.rs`
- Rust Provider API Key 验证 — 4 种验证 profile（openai-compatible/google-query-key/anthropic-header/openrouter），/models 404 时 fallback 到 /chat/completions probe — 涉及文件: `src-tauri/src/provider_validate.rs`
- Rust 文件暂存 + 图片预览 — 使用 `image` crate 实现 512px max 维度 resize（替代 nativeImage），MIME 类型映射（~40 种扩展名）— 涉及文件: `src-tauri/src/file_staging.rs`
- Rust 文件搜索引擎 — macOS mdfind / 非 macOS walkdir，路径安全检查，二进制检测，内容读取（50KB 限制）— 涉及文件: `src-tauri/src/file_search.rs`
- 37 个新 Tauri 命令 — 12 个 `provider:*` + 3 个 `skill:*` + 7 个 `openclaw:*` + 4 个 `log:*` + 3 个 `file:/media:*` + 2 个 `filesearch:*` + 2 个 `uv:*` — 涉及文件: `src-tauri/src/commands/provider_cmd.rs`, `skill_cmd.rs`, `openclaw_cmd.rs`, `log_cmd.rs`, `file_cmd.rs`, `filesearch_cmd.rs`, `uv_cmd.rs`

#### 变更 (Changed)
- Bridge 路由扩展 — `STORAGE_PREFIXES` 新增 `provider:`, `skill:`, `openclaw:`, `log:`, `file:`, `media:`, `filesearch:`, `uv:` 前缀；`SIDECAR_OVERRIDES` 新增 `log:getRecent`（Sidecar 内存 ring buffer）— 涉及文件: `src/lib/bridge.ts`
- lib.rs 命令注册 — 64 → 101 个命令 — 涉及文件: `src-tauri/src/lib.rs`
- commands/mod.rs 扩展 — 新增 provider_cmd, skill_cmd, openclaw_cmd, log_cmd, file_cmd, filesearch_cmd, uv_cmd 模块 — 涉及文件: `src-tauri/src/commands/mod.rs`
- Cargo.toml 新增依赖 — image v0.25 (png/jpeg/gif/webp/bmp features), walkdir v2 — 涉及文件: `src-tauri/Cargo.toml`

#### 架构 (Architecture)
- Provider 链路变化：`Renderer → Tauri invoke → Rust → clawx-providers.json + auth-profiles.json + openclaw.json`（替代 `Renderer → HTTP → Sidecar → electron-store → fs`）
- File Staging 升级：Rust `image` crate 实现真正的图片 resize（Sidecar 模式下无法调用 nativeImage，只能返回 raw base64）
- 通道分类更新：101 个 Rust 原生命令 + ~25 个 Sidecar HTTP 业务通道 + Gateway WS
- 仅剩 Sidecar 通道：clawhub:* (5) + log:getRecent (1) + gateway:start/stop/restart (3) + channel:requestWhatsAppQr/cancelWhatsAppQr (2) + knowledge pipeline (8) + workflow engine (6)
- 67 个单元测试全部通过 (`cargo test`)

---

### [0.6.0-alpha.7] - 2026-02-20

> Phase 5 Week 7: 全面迁移 — 消除 Sidecar 依赖，22 个通道迁移到 Rust

#### 新增 (Added)
- Rust ClawHub CLI 交互 — `tokio::process::Command` 调用 clawhub CLI，ANSI 转义去除，stdout 解析搜索/列表/安装/卸载/readme — 涉及文件: `src-tauri/src/clawhub.rs`, `src-tauri/src/commands/clawhub_cmd.rs`
- Rust Channel 验证 — `channel:validate`（openclaw doctor CLI）和 `channel:validateCredentials`（Discord/Telegram API 验证）— 涉及文件: `src-tauri/src/channel_validate.rs`
- Rust 文档解析器 — PDF（lopdf）、DOCX（zip + quick-xml `<w:t>` 标签提取）、纯文本，支持页数元数据 — 涉及文件: `src-tauri/src/document_parser.rs`
- Rust 网页抓取 — reqwest + scraper HTML DOM 解析，CSS 选择器内容区检测（article/main/[role=main]），过滤 script/style/nav，保留段落结构 — 涉及文件: `src-tauri/src/web_scraper.rs`
- Rust Embedding API 工具 — 模型发现（get_embedding_options）、维度检测（detect_dimension）、Provider 路由 — 涉及文件: `src-tauri/src/embedding_api.rs`
- Rust 知识管线编排 — parse → chunk → embed (batch 32) → insert → 进度事件发射，完整复现 TS 管线 — 涉及文件: `src-tauri/src/knowledge_pipeline.rs`
- Rust 文件夹监听 — `notify` crate RecursiveMode + 500ms debounce，过滤 pdf/docx/txt/md/csv — 涉及文件: `src-tauri/src/folder_watcher.rs`
- Rust 工作流 DAG 执行引擎 — Kahn 拓扑排序、同层并行、5 种节点类型（Input/Agent/Condition/Merge/Output）、条件路由、合并策略、取消支持 — 涉及文件: `src-tauri/src/workflow_engine.rs`
- Rust 工作流触发器管理 — Cron（cron crate + tokio sleep）、文件变化（notify）、剪贴板（regex 模式匹配）、快捷键映射 — 涉及文件: `src-tauri/src/workflow_triggers.rs`
- Rust Gateway 生命周期管理 — WS 探测 → spawn（tauri-plugin-shell）→ wait_for_ready → 连接 WS 客户端，注入 Provider API keys 为环境变量 — 涉及文件: `src-tauri/src/gateway_lifecycle.rs`
- 22 个新 Tauri 命令 — 5 个 `clawhub:*` + 2 个 `channel:validate*` + 8 个 `knowledge:*`（4 简单 + 4 管线）+ 6 个 `workflow:*`（execute/cancel/triggers/templates）+ 3 个 `gateway:start/stop/restart` — 涉及文件: `src-tauri/src/commands/clawhub_cmd.rs`, `channel_cmd.rs`, `knowledge_cmd.rs`, `workflows_cmd.rs`, `gateway_cmd.rs`

#### 变更 (Changed)
- Bridge 路由更新 — `STORAGE_PREFIXES` 新增 `clawhub:` 前缀；`SIDECAR_OVERRIDES` 从 25 个缩减为 3 个（仅保留 `channel:requestWhatsAppQr`, `channel:cancelWhatsAppQr`, `log:getRecent`）— 涉及文件: `src/lib/bridge.ts`
- lib.rs 命令注册 — 101 → 123 个命令，AppState 新增 4 个字段（`folder_watcher`, `workflow_engine`, `workflow_triggers`, `gateway_process`），`workflow_engine` 和 `gateway_process` 使用 `tokio::sync::Mutex` 以支持 async 操作 — 涉及文件: `src-tauri/src/lib.rs`
- commands/mod.rs 扩展 — 新增 `clawhub_cmd` 模块 — 涉及文件: `src-tauri/src/commands/mod.rs`
- Cargo.toml 新增依赖 — lopdf v0.33（PDF）, zip v2（DOCX ZIP）, quick-xml v0.36（DOCX XML）, scraper v0.20（HTML DOM）, notify v6（文件监听）, cron v0.12（调度）, regex v1 — 涉及文件: `src-tauri/Cargo.toml`

#### 架构 (Architecture)
- 知识管线链路变化：`Renderer → Tauri invoke → Rust → lopdf/zip+quick-xml → text_chunker → reqwest(embedding) → rusqlite → 进度事件`（替代 `Renderer → HTTP → Sidecar → pdf-parse/mammoth → chunk → embedding API → SQLite`）
- 工作流执行链路变化：`Renderer → Tauri invoke → Rust DAG engine → GatewayClient.rpc() → WS → Gateway`（替代 `Renderer → HTTP → Sidecar → WorkflowEngine → WS → Gateway`）
- Gateway 生命周期链路变化：`Renderer → Tauri invoke → Rust → tauri-plugin-shell spawn → WS probe → 连接`（替代 `Renderer → HTTP → Sidecar → child_process spawn`）
- 通道分类更新：123 个 Rust 原生命令 + 3 个 Sidecar HTTP 业务通道 + Gateway WS
- 仅剩 Sidecar 通道（3 个永久保留）：`channel:requestWhatsAppQr`（Baileys npm WebSocket 协议）、`channel:cancelWhatsAppQr`（同上）、`log:getRecent`（Sidecar 进程内存 ring buffer）
- Sidecar 完全移除条件：放弃 WhatsApp channel + `log:getRecent` 改为 Rust 日志文件读取
- 83 个单元测试全部通过（`cargo test`），新增 15 个测试覆盖 document_parser、web_scraper、workflow_engine（图验证/拓扑排序/条件路由/合并策略）、embedding_api、clawhub、channel_validate
- 11 个新 Rust 文件（~2000 行），7 个新 Cargo 依赖

---

*后续变更记录将在此处追加。*
