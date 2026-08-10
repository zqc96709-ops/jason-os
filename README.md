# Jason OS V3

Local-first、Desktop-first 的个人操作系统，用来连接：

`Capture → Plan → Focus → Work → Time → Result → Review → Insight → Principle → Mental Model → Decision → Action`

## 开发运行

```bash
pnpm install
source "$HOME/.cargo/env"
pnpm tauri dev
```

## 验证

```bash
pnpm test
pnpm lint
pnpm build
cd src-tauri && cargo test
cd .. && pnpm tauri build --debug
```

## 快捷键

- `⌘ K`：Command Palette
- `⌘ Shift Space`：快速收集
- `Esc`：关闭搜索、命令、AI 或详情抽屉

## 本地数据

```text
~/Library/Application Support/com.jasonos.desktop/
├── jason-os.sqlite3
├── attachments/
├── exports/
└── backups/
```

HackStart API Key 保存于 应用私有凭据文件（权限 0600），不写入 SQLite 或导出文件。

## 构建产物

```text
src-tauri/target/debug/bundle/macos/Jason OS.app
```

## AI 服务商

AI 首席助理支持独立选择服务商和模型：

- HackStart：`gpt-5.5`
- DeepSeek：`deepseek-v4-pro`、`deepseek-v4-flash`
- MiniMax：`MiniMax-M3`

每个服务商的 API Key 分别保存到 应用私有凭据文件（权限 0600）。切换服务商时不会覆盖其他服务商的 Key。
- 火山引擎 Agent Plan：`kimi-k3`（Responses API，`/api/plan/v3/responses`）
