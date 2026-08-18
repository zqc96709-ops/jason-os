# V2 TODO

- [x] 定义 `buildFollowUpQueue` 失败测试
  - Acceptance: 覆盖逾期/今日、等待结果、等待复盘、显式关联去重
  - Verify: focused 35/35（含 completedAt 跨视图状态、归档证据加载、actualResult/空白值、actualValue=0/金额/证据/终态复盘上下文、结果详情备用入口与首页共用复盘预填、本地日期、阶段配额与刷新降级）
  - Files: `src/workbenchDaily.test.ts`, `src/model.test.ts`, `src/workbenchRefresh.test.ts`

- [x] 实现纯计算跟进队列
  - Acceptance: 不写库、不猜关系，返回安全表单预填值
  - Verify: focused test + production build 通过
  - Files: `src/workbenchDaily.ts`

- [x] 首页接入三阶段跟进区
  - Acceptance: 每项显示阶段、理由、来源和一个明确动作；复用 `onOpen/onCreate`
  - Verify: frontend 82/82 + build + lint；6 条展示采用分阶段轮询，不饿死结果/复盘；首页与详情抽屉共用完整复盘预填；首页日期与队列统一使用本地日历日；归档/外部查询失败降级为空数组、不拖垮主记录
  - Files: `src/App.tsx`, `src/App.css`, `src/workbenchRefresh.ts`

- [x] 真实桌面验收
  - Acceptance: 表单关联 ID 正确；保存前数据库不新增；已有闭环记录不重复提醒
  - Verify: Tauri runtime + SQLite 1/1/0 前后不变 + desktop UIA（打开任务、打开结果表单、读取任务预填“核对失败重试逻辑”、确认项目由上级自动关联、取消）
  - Verify: 首页显示本地日期“8月18日星期二”；结果说明与实际描述保持空白；`[object Object]` 关联标题未复发；最新 HMR 无新 console error
  - Files: no production data mutation; temporary test records = 0

- [x] 独立复审
  - Acceptance: 检查错误关联、自动写入、重复提醒、日期边界和 UI 交互风险
  - Verify: 三轮限域只读复审完成；最终 P0=0 / P1=0 / P2=0，结论“无阻塞项，P2 已关闭”
