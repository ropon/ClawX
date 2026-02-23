
<p align="center">
  <img src="src/assets/logo.svg" width="128" height="128" alt="ClawX Logo" />
</p>

<h1 align="center">ClawX</h1>

<p align="center">
  <strong>The Desktop Interface for OpenClaw AI Agents</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#why-clawx">Why ClawX</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#development">Development</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-MacOS%20%7C%20Windows%20%7C%20Linux-blue" alt="Platform" />
  <img src="https://img.shields.io/badge/tauri-2-FFC131?logo=tauri" alt="Tauri" />
  <img src="https://img.shields.io/badge/react-19-61DAFB?logo=react" alt="React" />
  <a href="https://discord.com/invite/84Kex3GGAh" target="_blank">
  <img src="https://img.shields.io/discord/1399603591471435907?logo=discord&labelColor=%20%235462eb&logoColor=%20%23f5f5f5&color=%20%235462eb" alt="chat on Discord" />
  </a>
  <img src="https://img.shields.io/github/downloads/ValueCell-ai/ClawX/total?color=%23027DEB" alt="Downloads" />
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License" />
</p>

<p align="center">
  English | <a href="README.zh-CN.md">简体中文</a>
</p>

---

## Overview

**ClawX** bridges the gap between powerful AI agents and everyday users. Built on top of [OpenClaw](https://github.com/OpenClaw), it transforms command-line AI orchestration into an accessible, beautiful desktop experience—no terminal required.

Whether you're automating workflows, managing AI-powered channels, or scheduling intelligent tasks, ClawX provides the interface you need to harness AI agents effectively.

ClawX comes pre-configured with best-practice model providers and natively supports Windows as well as multi-language settings. Of course, you can also fine-tune advanced configurations via **Settings → Advanced → Developer Mode**.

---
## Screenshot

<p align="center">
  <img src="resources/screenshot/chat.png" style="width: 100%; height: auto;">
</p>

<p align="center">
  <img src="resources/screenshot/cron_task.png" style="width: 100%; height: auto;">
</p>

<p align="center">
  <img src="resources/screenshot/skills.png" style="width: 100%; height: auto;">
</p>

<p align="center">
  <img src="resources/screenshot/channels.png" style="width: 100%; height: auto;">
</p>

<p align="center">
  <img src="resources/screenshot/dashboard.png" style="width: 100%; height: auto;">
</p>

<p align="center">
  <img src="resources/screenshot/settings.png" style="width: 100%; height: auto;">
</p>

---

## Why ClawX

Building AI agents shouldn't require mastering the command line. ClawX was designed with a simple philosophy: **powerful technology deserves an interface that respects your time.**

| Challenge | ClawX Solution |
|-----------|----------------|
| Complex CLI setup | One-click installation with guided setup wizard |
| Configuration files | Visual settings with real-time validation |
| Process management | Automatic gateway lifecycle management |
| Multiple AI providers | Unified provider configuration panel |
| Skill/plugin installation | Built-in skill marketplace and management |

### OpenClaw Inside

ClawX is built directly upon the official **OpenClaw** core. Instead of requiring a separate installation, we embed the runtime within the application to provide a seamless "battery-included" experience.

We are committed to maintaining strict alignment with the upstream OpenClaw project, ensuring that you always have access to the latest capabilities, stability improvements, and ecosystem compatibility provided by the official releases.

---

## Features

### Multi-Agent Management
Create multiple Agent roles with independent personas, system prompts, model preferences, and skill sets. Switch between agents in chat or bind them to channels.

### Desktop AI Assistant (Spotlight)
Global hotkey (`Ctrl+Shift+Space`) summons a floating Spotlight-style window for instant AI interaction. Includes clipboard detection, screenshot analysis, quick commands, and file search.

### Knowledge Base & RAG
Build and manage knowledge bases with document upload (PDF, TXT, MD, DOCX), web scraping, and folder watching. Agents automatically retrieve relevant context during conversations.

### Workflow Automation
Visual node-based workflow editor with agent nodes, condition routing, and parallel execution. Chain multiple agents together with cron, file-change, clipboard, or hotkey triggers.

### Intelligent Chat Interface
Communicate with AI agents through a modern chat experience. Support for multiple conversation contexts, message history, file attachments, and rich content rendering with Markdown.

### Multi-Channel Management
Configure and monitor multiple AI channels simultaneously. Each channel operates independently, allowing you to run specialized agents for different tasks.

### Cron-Based Scheduling
Schedule AI tasks to run automatically. Define triggers, set intervals, and let your AI agents work around the clock without manual intervention.

### Extensible Skill System
Extend your AI agents with pre-built skills. Browse, install, and manage skills through the integrated ClawHub marketplace.

### Secure Provider Integration
Connect to multiple AI providers (OpenAI, Anthropic, Google, DeepSeek, Qwen, and more) with credentials stored securely in your system's native keychain.

### Adaptive Theming
Light mode, dark mode, or system-synchronized themes. ClawX adapts to your preferences automatically.

---

## Getting Started

### System Requirements

- **Operating System**: macOS 11+, Windows 10+, or Linux (Ubuntu 20.04+)
- **Memory**: 4GB RAM minimum (8GB recommended)
- **Storage**: 1GB available disk space
- **Rust**: 1.77+ (for building from source)

### Installation

#### Pre-built Releases (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/ValueCell-ai/ClawX/releases) page.

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon | `*.aarch64.dmg` |
| macOS | Intel | `*.x64.dmg` |
| Windows | x64 | `*.msi` or `*.exe` |
| Linux | x64 | `*.AppImage` or `*.deb` |

#### Build from Source

```bash
# Clone the repository
git clone https://github.com/ValueCell-ai/ClawX.git
cd ClawX

# Install frontend dependencies
pnpm install

# Start in development mode (launches Tauri + Vite)
pnpm tauri:dev
```

### First Launch

When you launch ClawX for the first time, the **Setup Wizard** will guide you through:

1. **Language & Region** – Configure your preferred locale
2. **AI Provider** – Enter your API keys for supported providers
3. **Skill Bundles** – Select pre-configured skills for common use cases
4. **Verification** – Test your configuration before entering the main interface

---

## Architecture

ClawX employs a **Tauri 2 native architecture** with Rust handling all backend logic and a WebView rendering the React UI:

```
┌─────────────────────────────────────────────────────────────────┐
│                        ClawX Desktop App                        │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Tauri Main Process (Rust)                     │  │
│  │  • 123+ #[tauri::command] handlers                        │  │
│  │  • Gateway process lifecycle (spawn, monitor, restart)    │  │
│  │  • Ed25519 device authentication                          │  │
│  │  • SQLite databases (knowledge vectors, workflow runs)    │  │
│  │  • File I/O, document parsing, web scraping               │  │
│  │  • System tray, global shortcuts, notifications           │  │
│  │  • Auto-updater with signed releases                      │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                              │ Tauri IPC (invoke / events)       │
│                              ▼                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              WebView (React 19 + TypeScript)              │  │
│  │  • Modern component-based UI (shadcn/ui)                  │  │
│  │  • State management with Zustand stores                   │  │
│  │  • IPC bridge (invoke + event listeners)                  │  │
│  │  • Workflow visual editor (@xyflow/react)                 │  │
│  │  • i18n (English, Chinese, Japanese)                      │  │
│  └───────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               │ WebSocket (JSON-RPC v3)
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                     OpenClaw Gateway                             │
│                                                                  │
│  • AI agent runtime and orchestration                           │
│  • Message channel management                                    │
│  • Skill/plugin execution environment                           │
│  • Provider abstraction layer                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Design Principles

- **Native Performance**: Rust backend with ~5MB binary size (vs ~150MB Electron). Near-zero idle memory overhead.
- **Process Isolation**: The AI runtime operates in a separate process, ensuring UI responsiveness even during heavy computation
- **Graceful Recovery**: Built-in reconnection logic with exponential backoff handles transient failures automatically
- **Secure Storage**: API keys and sensitive data leverage the operating system's native secure storage mechanisms
- **Hot Reload**: Development mode supports instant UI updates without restarting the gateway

---

## Use Cases

### Personal AI Assistant
Configure a general-purpose AI agent that can answer questions, draft emails, summarize documents, and help with everyday tasks—all from a clean desktop interface.

### Automated Monitoring
Set up scheduled agents to monitor news feeds, track prices, or watch for specific events. Results are delivered to your preferred notification channel.

### Developer Productivity
Integrate AI into your development workflow. Use agents to review code, generate documentation, or automate repetitive coding tasks.

### Workflow Automation
Chain multiple skills together to create sophisticated automation pipelines. Process data, transform content, and trigger actions—all orchestrated visually.

---

## Development

### Prerequisites

- **Node.js**: 20+ (LTS recommended)
- **Package Manager**: pnpm 9+
- **Rust**: 1.77+ with `stable` toolchain
- **Platform Dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools (C++ workload)
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev libgtk-3-dev`

### Project Structure

```
ClawX/
├── src-tauri/               # Tauri Main Process (Rust)
│   ├── src/
│   │   ├── commands/       # 123+ #[tauri::command] handlers
│   │   ├── gateway/        # WebSocket JSON-RPC client (Actor pattern)
│   │   ├── database/       # SQLite (knowledge vectors, workflow runs)
│   │   ├── storage/        # JSON file persistence (agents, workflows, knowledge)
│   │   └── ...             # Auth, providers, parsers, scraper, etc.
│   ├── capabilities/       # Tauri permission capabilities
│   ├── icons/              # Application icons (all sizes)
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── src/                     # React Frontend (WebView)
│   ├── components/         # Reusable UI components
│   │   ├── ui/            # Base components (shadcn/ui)
│   │   ├── layout/        # Layout (sidebar, title bar)
│   │   └── common/        # Shared components (error boundary)
│   ├── pages/             # Application pages
│   │   ├── Setup/         # Initial setup wizard
│   │   ├── Dashboard/     # Home dashboard
│   │   ├── Chat/          # AI chat interface
│   │   ├── Agents/        # Agent management
│   │   ├── Spotlight/     # Global quick-access window
│   │   ├── Knowledge/     # Knowledge base management
│   │   ├── Workflows/     # Visual workflow editor
│   │   ├── Channels/      # Channel management
│   │   ├── Skills/        # Skill marketplace
│   │   ├── Cron/          # Scheduled tasks
│   │   └── Settings/      # Configuration panels
│   ├── stores/            # Zustand state stores
│   ├── lib/               # Utilities (bridge, providers)
│   ├── i18n/              # Internationalization (en, zh, ja)
│   └── types/             # TypeScript type definitions
├── docs/                   # Project documentation
├── scripts/               # Build & utility scripts
└── tests/                 # Test suites
```

### Available Commands

```bash
# Development
pnpm tauri:dev            # Start Tauri + Vite with hot reload
pnpm dev                  # Start Vite dev server only

# Quality
pnpm lint                 # Run ESLint
pnpm lint:fix             # Auto-fix issues
pnpm typecheck            # TypeScript validation

# Testing
pnpm test                 # Run Vitest unit tests
pnpm test:watch           # Watch mode
cargo test                # Run Rust unit tests (in src-tauri/)

# Build & Package
pnpm build:vite           # Build frontend only
pnpm tauri:build          # Full production build (Rust + frontend)
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| Runtime | Tauri 2 (Rust) |
| UI Framework | React 19 + TypeScript |
| Styling | Tailwind CSS + shadcn/ui |
| State | Zustand |
| Build | Vite + Tauri CLI |
| Testing | Vitest (frontend) + cargo test (Rust) |
| Workflow Editor | @xyflow/react |
| Animation | Framer Motion |
| Icons | Lucide React |
| i18n | i18next |

---

## Contributing

We welcome contributions from the community! Whether it's bug fixes, new features, documentation improvements, or translations—every contribution helps make ClawX better.

### How to Contribute

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes with clear messages
4. **Push** to your branch
5. **Open** a Pull Request

### Guidelines

- Follow the existing code style (ESLint + Prettier for TS, `cargo fmt` for Rust)
- Write tests for new functionality
- Update documentation as needed
- Keep commits atomic and descriptive

---

## Acknowledgments

ClawX is built on the shoulders of excellent open-source projects:

- [OpenClaw](https://github.com/OpenClaw) – The AI agent runtime
- [Tauri](https://tauri.app/) – Native cross-platform desktop framework
- [React](https://react.dev/) – UI component library
- [shadcn/ui](https://ui.shadcn.com/) – Beautifully designed components
- [Zustand](https://github.com/pmndrs/zustand) – Lightweight state management

---

## Community

Join our community to connect with other users, get support, and share your experiences.

| Enterprise WeChat | Feishu Group | Discord |
| :---: | :---: | :---: |
| <img src="src/assets/community/wecom-qr.png" width="150" alt="WeChat QR Code" /> | <img src="src/assets/community/feishu-qr.png" width="150" alt="Feishu QR Code" /> | <img src="src/assets/community/20260212-185822.png" width="150" alt="Discord QR Code" /> |

---

## License

ClawX is released under the [MIT License](LICENSE). You're free to use, modify, and distribute this software.

---

<p align="center">
  <sub>Built with ❤️ by the ValueCell Team</sub>
</p>
