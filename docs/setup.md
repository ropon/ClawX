 一、什么叫“初始化完成”（验收清单）
  - 应用首次启动完成“首 run 引导/预置”流程（不再弹首次向导）。
  - 在用户目录下创建了 OneClaw 的配置目录（例如：~/.oneclaw，或 macOS 的 ~/Library/Application Support/OneClaw；Linux 也可能使用 ~/.oneclaw）。
  - 生成或写入了初始文件：IDENTITY.md（或类似默认 identity）、oneclaw‑preinstalled.json、oneclaw-extensions.json、uv.toml（如果有）、及 logs 目录下有 oneclaw-*.log。
  - 本地存储 / IndexedDB / localStorage 中出现 OneClaw 的 key（例如 oneclaw:gateway-ws-diagnostic、oneclaw:api-log、oneclaw:allow-localhost-fallback 等）。
  - 启动日志含有 “[OneClaw] Application Starting” 或类似标记。
  - 预装技能/插件列表被写入 OneClaw 的配置位置（如果有该功能）。
  - CLI wrapper（resources/cli/*）中的 OPENCLAW_EMBEDDED_IN 已指向 OneClaw，CLI 的“update”说明显示 OneClaw。

  二、本地重测初始化的步骤（快速、可重复）
  准备：先在 repo 根目录打开终端。

  1. 备份/清理现有用户数据（把真实数据先备份）
  - macOS / Linux 示例（做一次备份）：
    - mv ~/.oneclaw ~/.oneclaw.bak 2>/dev/null || true
    - mv ~/Library/Application\ Support/OneClaw ~/Library/Application\ Support/OneClaw.bak 2>/dev/null || true
  - Windows（PowerShell）：
    - Rename-Item $env:LOCALAPPDATA\oneclaw oneclaw.bak -ErrorAction SilentlyContinue
  说明：这样可以把 app 当作“首次运行”来触发初始化逻辑。

  2. 为可重复的测试使用临时用户目录（推荐）
  - 在 macOS / Linux：
    - export ONECLAW_USER_DATA_DIR="$(mktemp -d)"
    - export ONECLAW_E2E=1
  - 在 Windows（Powershell）：
    - $env:ONECLAW_USER_DATA_DIR = New-Item -ItemType Directory (Join-Path $env:TEMP (New-Guid))
    - $env:ONECLAW_E2E = "1"

  3. 启动开发环境（用你平常启动的命令，但确保带上上面 env）
  - 找到 package.json 的启动脚本（如果不确定）：cat package.json 看 scripts
  - 例子（把下面替换为你实际用的启动命令）：
    - ONECLAW_USER_DATA_DIR="$ONECLAW_USER_DATA_DIR" ONECLAW_E2E=1 pnpm install
    - ONECLAW_USER_DATA_DIR="$ONECLAW_USER_DATA_DIR" ONECLAW_E2E=1 pnpm run dev         # 启动前端 dev server
    - 在另一个终端： ONECLAW_USER_DATA_DIR="$ONECLAW_USER_DATA_DIR" ONECLAW_E2E=1 pnpm run electron:dev   # 或你通常运行 electron 的命令
  说明：关键是保证运行时环境变量 ONECLAW_USER_DATA_DIR/ONECLAW_E2E 被传入 Electron main 进程，使其把用户数据写到临时目录，便于检查和清理。

  4. 观察初始化日志和文件（验证点）
  - 查看日志目录（如果使用临时 dir）：
    - ls -la "$ONECLAW_USER_DATA_DIR"
    - ls -la "$ONECLAW_USER_DATA_DIR/logs"
    - tail -f "$ONECLAW_USER_DATA_DIR/logs/oneclaw-*.log"
  - 或系统默认：tail -f ~/.oneclaw/logs/oneclaw-*.log
  - 搜索日志中启动标记：
    - grep -n "\[OneClaw\]" "$ONECLAW_USER_DATA_DIR/logs/"* || true
  - 检查 Identity、preinstalled 文件：
    - cat "$ONECLAW_USER_DATA_DIR/IDENTITY.md"  (或者项目里 openclaw-workspace 指定的位置)
    - ls -la ~/.oneclaw  或 ls -la "$ONECLAW_USER_DATA_DIR"

  5. 检查浏览器端 localStorage（前端 dev server）
  - 打开渲染器（http://localhost:5173 或你的 dev server 地址），打开 DevTools Console：
    - localStorage.getItem('oneclaw:gateway-ws-diagnostic')
    - localStorage.getItem('oneclaw:api-log')
    - localStorage.getItem('oneclaw:allow-localhost-fallback')
  这些键应与初始化逻辑相符（存在或为默认值）。

  6. 验证 CLI wrapper 行为
  - 运行 resources/cli/posix/openclaw update 或 resources/cli/win32/openclaw.cmd update，应该输出 “openclaw is managed by OneClaw ...” 的说明（注意：在开发环境下可能需要调整 PATH）。

  7. 验证迁移（如果你要测试从旧 .clawx 到 .oneclaw 的迁移）
  - 先还原或写入一个模拟旧目录：
    - mv ~/.oneclaw ~/.oneclaw.after-migration  # 如果你之前已经创建
    - mkdir -p ~/.clawx
    - echo "Old identity" > ~/.clawx/IDENTITY.md
  - 启动应用（不设置 ONECLAW_USER_DATA_DIR，这样它会读取真实主目录），观察是否在首次运行时将旧 ~/.clawx 的内容迁移到 ~/.oneclaw（或产生迁移日志）。
  - 查看日志、检查 ~/.oneclaw 是否包含从 ~/.clawx 迁移过来的文件。

  8. 可自动化的单次脚本（示例）
  - 在 repo 根创建一个临时测试脚本 test-init.sh：
    - #!/bin/sh
    - export ONECLAW_USER_DATA_DIR="$(mktemp -d)"
    - export ONECLAW_E2E=1
    - echo "USER DATA DIR: $ONECLAW_USER_DATA_DIR"
    - ONECLAW_USER_DATA_DIR="$ONECLAW_USER_DATA_DIR" ONECLAW_E2E=1 pnpm run electron:dev
  （改成你自己的启动命令）

  三、常见问题 & 排查命令
  - 没创建 ~/.oneclaw？确认 app 是否有权限写 home，或检查 env 是否正确传入到 Electron。
  - 看不到日志：确认日志目录位置（search）： grep -R "OneClaw Session Start" -n ~
  - localStorage 没有键：确保你打开的是 Electron 渲染器或本地 dev server（Electron 的 renderer localStorage 与浏览器不同）。
  - 迁移失败：检查主进程日志是否包含“migrate”或“repair”相关日志行。
  - 托盘/菜单图标没有替换：确认 resources/icons 里对应的 PNG/ICNS/ICO 文件已替换为 OneClaw 版本（我可以替换它们，如果你把新图标文件放到 repo 中）。

  四、我可以为你做的（直接执 行）
  - 我可以在你的仓库中运行一组验证命令并把结果（关键文件/日志片段）贴给你：包括列出 ONECLAW_USER_DATA_DIR 内容、tail 日志输出、确认 localStorage keys（如果你允许我启动 dev server/electron）。
  - 我也可以生成一个可复用的 shell 脚本（test-init.sh），一次性执行“清空、启动、收集证据、还原”的流程，方便你在本地重复跑。

  如果你现在想立刻测试初始化，我可以：
  - （A）给你一个一键脚本（test-init.sh），你本地运行即可；
  - 或者（B）我现在在本地为你执行验证命令并回报结果（需要我运行哪些启动命令？例如：pnpm run dev / pnpm run electron:dev — 把你平常启动项目的命令告诉我）。

  告诉我你偏好 A 还是 B，或者直接粘贴你启动项目的具体命令，我就帮你把验证脚本生成好/运行好。