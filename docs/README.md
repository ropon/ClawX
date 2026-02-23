# ClawX 技术文档

> 本目录是 ClawX 项目所有功能规划、技术设计和变更追踪的管理中心。

## 文档索引

| 文档 | 说明 | 更新频率 |
|---|---|---|
| [ROADMAP.md](./ROADMAP.md) | 功能演进路线图，所有功能的规划与状态追踪 | 每次功能状态变更时 |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 详细技术架构，包括数据模型、API 设计、文件结构 | 每个 Phase 开始前完善 |
| [TAURI_MIGRATION.md](./TAURI_MIGRATION.md) | Electron → Tauri 2 迁移策略与实施步骤 | Phase 5 前持续更新 |
| [CHANGELOG.md](./CHANGELOG.md) | 功能变更日志，记录所有添加/修改/删除 | 每次代码合并时 |

## 工作流程

### 新功能开发流程

```
1. 在 ROADMAP.md 中确认功能 ID 和状态
2. 在 ARCHITECTURE.md 中完成技术设计
3. 开发实现
4. 更新 ROADMAP.md 中的功能状态为「已完成」
5. 在 CHANGELOG.md 中记录变更
```

### 功能修改流程

```
1. 在 CHANGELOG.md 中记录变更原因
2. 如果涉及架构变更，先更新 ARCHITECTURE.md
3. 开发实现
4. 更新 CHANGELOG.md 中的变更记录
```

## Phase 概览

| Phase | 版本 | 核心目标 |
|---|---|---|
| Phase 1 | v0.2.x | 多 Agent 角色管理 |
| Phase 2 | v0.3.x | 桌面助手核心能力（Spotlight、全局快捷键、剪贴板、截图） |
| Phase 3 | v0.4.x | 知识库 & RAG |
| Phase 4 | v0.5.x | Agent 协作 & 高级自动化（工作流编排） |
| Phase 5 | v0.6.x | Electron → Tauri 2 迁移 |
