# Jason OS V2：提醒 → 结果 → 复盘闭环

## Objective
在不改变 V1 已验收 AI Action、多草稿、手动确认和 SQLite 数据契约的前提下，把已有任务与结果转成一个证据驱动的跟进队列：

1. 未完成且今天到期或已经逾期的任务提醒继续行动；
2. 已完成、但尚无关联结果的任务提醒回填结果；
3. 已有结果、但尚无关联复盘的记录提醒开始复盘；
4. 所有写入继续通过现有表单由用户检查并保存，不自动创建结果、复盘、洞见或原则。

## Assumptions
- 本切片的“主动提醒”指 Jason OS 打开后首页主动呈现，不包含 Windows 系统通知、微信或定时后台任务。
- 同一条任务/结果只出现在一个当前阶段；已有显式 taskId/resultId 关联即视为闭环已向后推进。
- 任务结果不能由系统猜成“成功”，因此结果表单只预填来源关系和标题，达成状态仍由用户确认。
- 复盘之后的洞见/原则提升继续使用已有确认流程，不在本切片自动晋级长期记忆。

## Tech Stack
- React + TypeScript + Vitest
- Tauri + Rust
- SQLite records 通用实体表

## Commands
- Focused test: `npx vitest run src/workbenchDaily.test.ts`
- Frontend gate: `npx vitest run && npm run build && git diff --check`
- Rust gate: `cargo fmt --check && cargo test`（在 `src-tauri/`）
- Desktop runtime: `npm run tauri dev`

## Project Structure
- `src/workbenchDaily.ts`: 纯计算的 Daily Brief 与跟进队列
- `src/workbenchDaily.test.ts`: 队列规则与关联去重测试
- `src/App.tsx`: 首页跟进区与现有表单交接
- `src/App.css`: 跟进区结构样式
- `tasks/plan.md`: 本规格与实现计划
- `tasks/todo.md`: 可执行任务清单

## Code Style
```ts
const queue = buildFollowUpQueue(records, new Date('2026-08-18T09:00:00+08:00'))
// 纯函数只返回证据、阶段和安全交接参数；不写数据库。
```

## Testing Strategy
- 小型单元测试：阶段判定、排序、显式关联去重、无证据时空队列。
- 前端全量测试与生产构建。
- Tauri/Rust 全量回归，证明未破坏 AI Action 和 SQLite。
- 真实桌面：至少验证一条“等待结果”和一条“等待复盘”点击后打开正确表单、真实 ID 已预填、保存前无新增记录。

## Boundaries
### Always
- 只使用真实 records 和显式 ID 关系。
- 写入前让用户看到可编辑表单。
- 保留任务、项目、目标、结果来源链。

### Ask first
- Windows 系统通知、微信提醒、后台定时器。
- 数据库新实体或迁移。
- 自动发布洞见、原则或长期记忆。

### Never
- 因时间接近而猜测任务与结果关系。
- 完成任务后自动判定结果已达成。
- 自动把单次复盘提升为长期规则。
- 重置当前未提交的 V1 工作树。

## Success Criteria
- 到期/逾期任务能显示“打开任务”。
- 完成且无结果的任务能显示“回填结果”，表单包含真实 taskId/projectId/goalId。
- 无复盘的结果能显示“开始复盘”，表单包含真实 resultId/taskId/projectId/goalId。
- 已有结果或复盘时对应提醒消失，不重复出现。
- 点击只打开表单，未保存前 SQLite 业务记录数不变。
- 前端、构建、diff、Rust 全绿，真实桌面流程通过。

## Implementation Order
1. RED：为队列规则编写失败测试。
2. GREEN：实现纯函数与安全预填参数。
3. 在首页新增结构明显的三阶段跟进区。
4. 自动化回归。
5. 真实桌面验证和独立复审。
