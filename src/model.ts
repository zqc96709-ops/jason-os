import { timelineOccurredAt, timelineRecords } from './timeline'
export type Entity =
  | 'goals' | 'keyResults' | 'projects' | 'tasks' | 'hypotheses' | 'experiments' | 'timeLogs' | 'results'
  | 'reviews' | 'knowledge' | 'insights' | 'principles' | 'mentalModels' | 'mentalModelUsages'
  | 'decisions' | 'inbox' | 'events' | 'people' | 'dataRecords' | 'attachments' | 'timelineEvents' | 'agentRuns' | 'agentActions'
  | 'externalSources' | 'signals' | 'opportunities' | 'intelligenceBriefs'

export type RecordData = Record<string, unknown> & {
  id: string
  entity: Entity
  createdAt: string
  updatedAt: string
  archivedAt?: string
  deletedAt?: string
  relationType?: string
  relationDirection?: 'incoming' | 'outgoing'
}

export type FieldOption = string | { value: string; label: string }
export type Field = {
  key: string
  label: string
  multiline?: boolean
  type?: 'date' | 'datetime-local' | 'number' | 'select'
  options?: FieldOption[]
  relation?: Entity
  multiple?: boolean
  placeholder?: string
}
export type EntityConfig = {
  entity: Entity
  label: string
  singular: string
  icon: string
  titleKey: string
  description: string
  fields: Field[]
}

const option = (value: string, label = value): FieldOption => ({ value, label })
const statuses = {
  goal: [option('active', '进行中'), option('planned', '计划中'), option('paused', '已暂停'), option('completed', '已完成'), option('archived', '已归档')],
  project: [option('active', '进行中'), option('planned', '计划中'), option('blocked', '受阻'), option('paused', '已暂停'), option('completed', '已完成')],
  task: [option('inbox', '收集箱'), option('todo', '待办'), option('in_progress', '进行中'), option('waiting', '等待中'), option('completed', '已完成'), option('cancelled', '已取消')],
  hypothesis: [option('untested', '未测试'), option('testing', '测试中'), option('validated', '已验证'), option('rejected', '已否定'), option('inconclusive', '无结论')],
  decision: [option('pending', '待决定'), option('decided', '已决定'), option('monitoring', '观察中'), option('validated', '已验证'), option('partially_correct', '部分正确'), option('wrong', '错误'), option('unknown', '未知')],
}
const priorities = [option('high', '高'), option('medium', '中'), option('low', '低')]

export const entities: EntityConfig[] = [
  { entity: 'goals', label: '目标', singular: '目标', icon: '◎', titleKey: 'title', description: '目标定义方向与为什么值得投入。', fields: [
    { key: 'title', label: '目标' }, { key: 'description', label: '说明', multiline: true }, { key: 'why', label: '为什么重要', multiline: true },
    { key: 'status', label: '状态', type: 'select', options: statuses.goal }, { key: 'timeframe', label: '时间范围' },
    { key: 'progress', label: '进度（%）', type: 'number' }, { key: 'priority', label: '优先级', type: 'select', options: priorities },
    { key: 'startDate', label: '开始日期', type: 'date' }, { key: 'targetDate', label: '目标日期', type: 'date' },
  ] },
  { entity: 'keyResults', label: '关键结果', singular: '关键结果', icon: '↗', titleKey: 'title', description: '用可衡量结果判断目标是否真正推进。', fields: [
    { key: 'goalId', label: '所属目标', relation: 'goals' }, { key: 'title', label: '关键结果' }, { key: 'targetValue', label: '目标值', type: 'number' },
    { key: 'currentValue', label: '当前值', type: 'number' }, { key: 'unit', label: '单位' }, { key: 'status', label: '状态', type: 'select', options: statuses.goal },
  ] },
  { entity: 'projects', label: '项目', singular: '项目', icon: '◈', titleKey: 'title', description: '项目是把目标变成结果的工作空间。', fields: [
    { key: 'title', label: '项目名称' }, { key: 'goalId', label: '所属目标', relation: 'goals' }, { key: 'why', label: '为什么做', multiline: true },
    { key: 'description', label: '说明', multiline: true }, { key: 'status', label: '状态', type: 'select', options: statuses.project },
    { key: 'health', label: '健康度', type: 'select', options: [option('healthy', '健康'), option('at_risk', '有风险'), option('blocked', '受阻')] },
    { key: 'progress', label: '进度（%）', type: 'number' }, { key: 'priority', label: '优先级', type: 'select', options: priorities },
    { key: 'startDate', label: '开始日期', type: 'date' }, { key: 'targetDate', label: '目标日期', type: 'date' },
    { key: 'blockers', label: '阻塞', multiline: true }, { key: 'nextAction', label: '下一步行动', multiline: true },
    { key: 'sourceDecisionId', label: '来源决策', relation: 'decisions' }, { key: 'sourceOpportunityId', label: '来源机会', relation: 'opportunities' }, { key: 'sourceSignalIds', label: '来源信号', relation: 'signals', multiple: true },
  ] },
  { entity: 'tasks', label: '任务', singular: '任务', icon: '□', titleKey: 'title', description: '任务是可以立即执行的下一步行动。', fields: [
    { key: 'title', label: '任务' }, { key: 'description', label: '说明', multiline: true }, { key: 'decisionId', label: '来源决策', relation: 'decisions' }, { key: 'projectId', label: '所属项目', relation: 'projects' },
    { key: 'goalId', label: '所属目标', relation: 'goals' }, { key: 'status', label: '状态', type: 'select', options: statuses.task },
    { key: 'priority', label: '优先级', type: 'select', options: priorities },
    { key: 'importance', label: '重要程度', type: 'select', options: [option('important', '重要'), option('not_important', '不重要')] },
    { key: 'urgency', label: '紧急程度', type: 'select', options: [option('urgent', '紧急'), option('not_urgent', '不紧急')] },
    { key: 'dueDate', label: '截止日期', type: 'date' }, { key: 'dueAt', label: '具体时间', type: 'datetime-local' },
    { key: 'estimateMinutes', label: '预估分钟', type: 'number' }, { key: 'tags', label: '标签（逗号分隔）' },
    { key: 'dependencyIds', label: '依赖任务', relation: 'tasks', multiple: true }, { key: 'completedAt', label: '完成时间', type: 'datetime-local' },
  ] },
  { entity: 'hypotheses', label: '假设', singular: '假设', icon: '⌁', titleKey: 'title', description: '用实验和证据验证，而不是用感觉判断。', fields: [
    { key: 'title', label: '假设名称' }, { key: 'statement', label: '假设陈述', multiline: true }, { key: 'rationale', label: '依据', multiline: true },
    { key: 'projectId', label: '所属项目', relation: 'projects' }, { key: 'experimentId', label: '实验', relation: 'experiments' },
    { key: 'expectedOutcome', label: '预期结果', multiline: true }, { key: 'actualOutcome', label: '实际结果', multiline: true },
    { key: 'evidence', label: '证据', multiline: true }, { key: 'status', label: '状态', type: 'select', options: statuses.hypothesis }, { key: 'conclusion', label: '结论', multiline: true },
  ] },
  { entity: 'experiments', label: '实验', singular: '实验', icon: '⚗', titleKey: 'title', description: '为假设设计可执行、可复核的验证动作。', fields: [
    { key: 'title', label: '实验名称' }, { key: 'projectId', label: '所属项目', relation: 'projects' }, { key: 'hypothesisId', label: '验证假设', relation: 'hypotheses' },
    { key: 'method', label: '方法', multiline: true }, { key: 'successCriteria', label: '成功标准', multiline: true }, { key: 'startDate', label: '开始日期', type: 'date' },
    { key: 'endDate', label: '结束日期', type: 'date' }, { key: 'status', label: '状态', type: 'select', options: [option('planned', '计划中'), option('running', '进行中'), option('completed', '已完成'), option('cancelled', '已取消')] },
  ] },
  { entity: 'timeLogs', label: '时间', singular: '时间记录', icon: '◷', titleKey: 'title', description: '时间记录现实，而不是计划。', fields: [
    { key: 'title', label: '活动' }, { key: 'notes', label: '备注', multiline: true }, { key: 'startAt', label: '开始时间', type: 'datetime-local' },
    { key: 'endAt', label: '结束时间', type: 'datetime-local' }, { key: 'durationMinutes', label: '时长（分钟）', type: 'number' },
    { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'taskId', label: '任务', relation: 'tasks' },
    { key: 'category', label: '类别' }, { key: 'energyLevel', label: '精力（1-5）', type: 'number' },
  ] },
  { entity: 'results', label: '结果', singular: '结果', icon: '✓', titleKey: 'title', description: '记录现实结果，并与预期比较。', fields: [
    { key: 'title', label: '结果名称' }, { key: 'expected', label: '预期', multiline: true }, { key: 'actual', label: '实际', multiline: true },
    { key: 'variance', label: '偏差', multiline: true }, { key: 'impact', label: '影响', multiline: true }, { key: 'evidence', label: '证据', multiline: true },
    { key: 'date', label: '日期', type: 'date' }, { key: 'source', label: '来源' }, { key: 'taskId', label: '任务', relation: 'tasks' },
    { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'decisionId', label: '决策', relation: 'decisions' }, { key: 'hypothesisId', label: '假设', relation: 'hypotheses' },
  ] },
  { entity: 'reviews', label: '复盘', singular: '复盘', icon: '◑', titleKey: 'title', description: '把现实转化为可以改变未来行动的经验。', fields: [
    { key: 'title', label: '复盘主题' }, { key: 'whatHappened', label: '发生了什么', multiline: true }, { key: 'whyItHappened', label: '为什么', multiline: true },
    { key: 'whatWorked', label: '哪些有效', multiline: true }, { key: 'whatFailed', label: '哪些无效', multiline: true }, { key: 'lesson', label: '学到了什么', multiline: true },
    { key: 'doDifferently', label: '下次如何不同', multiline: true }, { key: 'nextAction', label: '下一步行动', multiline: true },
    { key: 'taskId', label: '任务', relation: 'tasks' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' },
    { key: 'periodStart', label: '周期开始', type: 'date' }, { key: 'periodEnd', label: '周期结束', type: 'date' }, { key: 'resultId', label: '结果', relation: 'results' }, { key: 'decisionId', label: '决策', relation: 'decisions' },
  ] },
  { entity: 'knowledge', label: '知识', singular: '知识', icon: '⌘', titleKey: 'title', description: '保存可以长期复用的信息与理解。', fields: [
    { key: 'title', label: '标题' }, { key: 'content', label: '内容', multiline: true }, { key: 'source', label: '来源' }, { key: 'category', label: '分类' },
    { key: 'tags', label: '标签（逗号分隔）' }, { key: 'projectIds', label: '相关项目', relation: 'projects', multiple: true }, { key: 'reviewIds', label: '相关复盘', relation: 'reviews', multiple: true },
  ] },
  { entity: 'insights', label: '洞见', singular: '洞见', icon: '✦', titleKey: 'statement', description: '从经验中提炼“我发现了什么”。', fields: [
    { key: 'statement', label: '洞见' }, { key: 'explanation', label: '解释', multiline: true }, { key: 'evidence', label: '证据', multiline: true },
    { key: 'source', label: '来源' }, { key: 'taskId', label: '任务', relation: 'tasks' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' },
    { key: 'reviewId', label: '复盘', relation: 'reviews' }, { key: 'resultId', label: '结果', relation: 'results' }, { key: 'knowledgeId', label: '知识', relation: 'knowledge' }, { key: 'confidence', label: '置信度（0-100）', type: 'number' },
  ] },
  { entity: 'principles', label: '原则', singular: '原则', icon: '∴', titleKey: 'statement', description: '记录长期相信并愿意用于决策的原则。', fields: [
    { key: 'statement', label: '原则' }, { key: 'explanation', label: '解释', multiline: true }, { key: 'evidence', label: '证据', multiline: true },
    { key: 'examples', label: '示例', multiline: true }, { key: 'limitations', label: '局限', multiline: true }, { key: 'source', label: '来源' },
    { key: 'usage', label: '使用说明', multiline: true }, { key: 'insightIds', label: '来源洞见', relation: 'insights', multiple: true }, { key: 'reviewIds', label: '来源复盘', relation: 'reviews', multiple: true }, { key: 'mentalModelIds', label: '相关思维模型', relation: 'mentalModels', multiple: true },
  ] },
  { entity: 'mentalModels', label: '思维模型', singular: '思维模型', icon: '◇', titleKey: 'name', description: '可反复调用并验证有效性的认知工具。', fields: [
    { key: 'name', label: '名称' }, { key: 'category', label: '类型' }, { key: 'corePrinciple', label: '核心原则', multiline: true }, { key: 'problem', label: '解决的问题', multiline: true },
    { key: 'framework', label: '框架', multiline: true }, { key: 'steps', label: '流程步骤', multiline: true }, { key: 'keyQuestions', label: '核心问题', multiline: true },
    { key: 'useCases', label: '适用场景', multiline: true }, { key: 'outputTemplate', label: '输出模板', multiline: true }, { key: 'source', label: '来源' }, { key: 'tags', label: '标签（逗号分隔）' },
    { key: 'definition', label: '定义（兼容旧记录）', multiline: true }, { key: 'trigger', label: '何时使用（兼容旧记录）', multiline: true },
    { key: 'questions', label: '关键问题（兼容旧记录）', multiline: true }, { key: 'method', label: '方法（兼容旧记录）', multiline: true }, { key: 'application', label: '应用（兼容旧记录）', multiline: true },
    { key: 'limitations', label: '局限', multiline: true }, { key: 'examples', label: '示例', multiline: true },
    { key: 'insightId', label: '来源洞见', relation: 'insights' }, { key: 'reviewId', label: '来源复盘', relation: 'reviews' }, { key: 'principleIds', label: '相关原则', relation: 'principles', multiple: true },
  ] },
  { entity: 'mentalModelUsages', label: '模型使用', singular: '模型使用记录', icon: '⇄', titleKey: 'context', description: '追踪思维模型是否真正改善了结果。', fields: [
    { key: 'mentalModelId', label: '思维模型', relation: 'mentalModels' }, { key: 'decisionId', label: '决策', relation: 'decisions' },
    { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'context', label: '使用情境', multiline: true }, { key: 'outcome', label: '结果', multiline: true },
    { key: 'effective', label: '是否有效', type: 'select', options: [option('yes', '有效'), option('partial', '部分有效'), option('no', '无效'), option('unknown', '未知')] }, { key: 'notes', label: '备注', multiline: true },
  ] },
  { entity: 'decisions', label: '决策日志', singular: '决策', icon: '◆', titleKey: 'title', description: '记录问题、预测、结果与校准过程。', fields: [
    { key: 'title', label: '决策名称' }, { key: 'problem', label: '问题', multiline: true }, { key: 'context', label: '背景', multiline: true },
    { key: 'options', label: '备选方案', multiline: true }, { key: 'selectedOption', label: '选择', multiline: true }, { key: 'reasoning', label: '理由', multiline: true },
    { key: 'evidence', label: '证据', multiline: true }, { key: 'prediction', label: '预测', multiline: true }, { key: 'confidence', label: '置信度（0-100）', type: 'number' },
    { key: 'date', label: '决策日期', type: 'date' }, { key: 'status', label: '状态', type: 'select', options: statuses.decision }, { key: 'outcome', label: '实际结果', multiline: true },
    { key: 'taskId', label: '任务', relation: 'tasks' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' },
    { key: 'knowledgeId', label: '知识', relation: 'knowledge' }, { key: 'insightId', label: '洞见', relation: 'insights' }, { key: 'resultId', label: '结果', relation: 'results' }, { key: 'reviewId', label: '复盘', relation: 'reviews' },
    { key: 'signalIds', label: '来源信号', relation: 'signals', multiple: true }, { key: 'opportunityId', label: '来源机会', relation: 'opportunities' },
    { key: 'principleIds', label: '调用原则', relation: 'principles', multiple: true }, { key: 'mentalModelIds', label: '调用思维模型', relation: 'mentalModels', multiple: true },
  ] },
  { entity: 'externalSources', label: '情报源', singular: '情报源', icon: '◉', titleKey: 'name', description: '定义 Jason OS 应持续观察的关键词、账号、竞品和主题。', fields: [
    { key: 'name', label: '名称' }, { key: 'type', label: '类型', type: 'select', options: [option('LINK', '固定链接'), option('KEYWORD', '关键词'), option('ACCOUNT', '账号'), option('COMPETITOR', '竞品'), option('MARKET', '市场'), option('TOPIC', '主题')] },
    { key: 'platform', label: '平台', type: 'select', options: ['微信公众号', '抖音', '小红书'] }, { key: 'query', label: '关键词 / 账号' }, { key: 'url', label: '公开链接' },
    { key: 'status', label: '状态', type: 'select', options: [option('active', '启用'), option('paused', '暂停'), option('unsupported', '接口待接入')] }, { key: 'pollInterval', label: '同步频率', type: 'select', options: [option('manual', '仅手动'), option('daily', '应用打开时每日一次')] },
    { key: 'providerPreference', label: '采集服务', type: 'select', options: [option('auto', '自动选择'), option('redfox', 'RedFoxHub')] }, { key: 'lastPolledAt', label: '最近同步', type: 'datetime-local' },
    { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'projectId', label: '项目', relation: 'projects' },
  ] },
  { entity: 'signals', label: '外部信号', singular: '外部信号', icon: '⌁', titleKey: 'title', description: '由可追溯的外部变化形成，等待观察、验证或进入决策。', fields: [
    { key: 'title', label: '信号' }, { key: 'type', label: '类型', type: 'select', options: ['GROWTH', 'VIRAL', 'COMPETITOR_ACTIVITY', 'DEMAND', 'RISK'] }, { key: 'summary', label: '摘要', multiline: true },
    { key: 'status', label: '状态', type: 'select', options: [option('DETECTED', '已发现'), option('WATCHING', '观察中'), option('VALIDATED', '已验证'), option('DISMISSED', '已忽略'), option('EXPIRED', '已过期'), option('CONVERTED', '已转化')] },
    { key: 'platform', label: '平台' }, { key: 'topic', label: '主题' }, { key: 'baselineValue', label: '基线值' }, { key: 'currentValue', label: '当前值' }, { key: 'changeRate', label: '变化率' },
    { key: 'baselineStart', label: '基线开始', type: 'date' }, { key: 'baselineEnd', label: '基线结束', type: 'date' }, { key: 'currentStart', label: '当前开始', type: 'date' }, { key: 'currentEnd', label: '当前结束', type: 'date' },
    { key: 'sampleSize', label: '样本量' }, { key: 'independentAuthorCount', label: '独立作者' }, { key: 'calculationMethod', label: '计算口径', multiline: true }, { key: 'evidenceItemIds', label: '证据内容 ID', multiple: true },
    { key: 'detectedAt', label: '发现时间', type: 'datetime-local' }, { key: 'expiresAt', label: '过期时间', type: 'datetime-local' }, { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'projectId', label: '项目', relation: 'projects' },
    { key: 'opportunityId', label: '机会', relation: 'opportunities' }, { key: 'decisionId', label: '决策', relation: 'decisions' },
  ] },
  { entity: 'opportunities', label: '机会', singular: '机会', icon: '◇', titleKey: 'title', description: '由已确认的信号形成，进入 CEO 判断而不是自动立项。', fields: [
    { key: 'title', label: '机会名称' }, { key: 'problem', label: '要解决的问题', multiline: true }, { key: 'hypothesis', label: '机会假设', multiline: true }, { key: 'market', label: '市场' }, { key: 'category', label: '分类' },
    { key: 'status', label: '状态', type: 'select', options: [option('draft', '草稿'), option('evaluating', '评估中'), option('decided', '已决策'), option('dismissed', '已放弃')] },
    { key: 'signalIds', label: '来源信号', relation: 'signals', multiple: true }, { key: 'evidence', label: '证据', multiline: true }, { key: 'dataGaps', label: '数据缺口', multiline: true },
    { key: 'demandEvidence', label: '需求证据', multiline: true }, { key: 'growthEvidence', label: '增长证据', multiline: true }, { key: 'competitionEvidence', label: '竞争证据', multiline: true }, { key: 'contentEvidence', label: '内容证据', multiline: true }, { key: 'marginEvidence', label: '利润证据', multiline: true }, { key: 'strategicFit', label: '战略适配', multiline: true },
    { key: 'decisionId', label: '决策', relation: 'decisions' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'resultId', label: '结果', relation: 'results' },
  ] },
  { entity: 'intelligenceBriefs', label: '情报简报', singular: '情报简报', icon: '✺', titleKey: 'title', description: '把少量重要信号、机会和风险整理成 CEO 可以快速阅读的简报。', fields: [
    { key: 'title', label: '简报标题' }, { key: 'periodStart', label: '周期开始', type: 'date' }, { key: 'periodEnd', label: '周期结束', type: 'date' }, { key: 'signalIds', label: '信号', relation: 'signals', multiple: true }, { key: 'opportunityIds', label: '机会', relation: 'opportunities', multiple: true }, { key: 'riskSignalIds', label: '风险信号', relation: 'signals', multiple: true }, { key: 'summary', label: '摘要', multiline: true }, { key: 'dataGaps', label: '数据缺口', multiline: true }, { key: 'generatedAt', label: '生成时间', type: 'datetime-local' }, { key: 'status', label: '状态', type: 'select', options: [option('draft', '草稿'), option('published', '已确认')] },
  ] },
  { entity: 'inbox', label: '收集箱', singular: '收集', icon: '↓', titleKey: 'content', description: '先记录事实，稍后再分类。', fields: [
    { key: 'content', label: '内容或链接', multiline: true, placeholder: '粘贴微信公众号、抖音、小红书、X、Instagram、Reddit、Facebook 等公开链接' }, { key: 'type', label: '建议类型' }, { key: 'status', label: '状态', type: 'select', options: [option('unprocessed', '待处理'), option('processed', '已处理'), option('archived', '已归档')] },
    { key: 'platform', label: '来源平台' }, { key: 'sourceUrl', label: '原始链接' }, { key: 'author', label: '作者' }, { key: 'captureStatus', label: '采集状态' },
  ] },
  { entity: 'events', label: '事件', singular: '事件', icon: '●', titleKey: 'title', description: '记录会议、约会、截止日与外部事件。', fields: [
    { key: 'title', label: '事件' }, { key: 'type', label: '类型', type: 'select', options: ['会议', '约会', '截止日', '重要事件', '外部事件'] },
    { key: 'startAt', label: '开始', type: 'datetime-local' }, { key: 'endAt', label: '结束', type: 'datetime-local' }, { key: 'location', label: '地点' },
    { key: 'notes', label: '备注', multiline: true }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'goalId', label: '目标', relation: 'goals' },
    { key: 'personIds', label: '人物', relation: 'people', multiple: true }, { key: 'decisionId', label: '决策', relation: 'decisions' }, { key: 'taskId', label: '任务', relation: 'tasks' },
  ] },
  { entity: 'people', label: '人物', singular: '人物', icon: '♙', titleKey: 'name', description: '为项目、事件和决策提供关系上下文，不是 CRM。', fields: [
    { key: 'name', label: '姓名' }, { key: 'organization', label: '组织' }, { key: 'role', label: '角色' }, { key: 'contact', label: '联系方式' }, { key: 'notes', label: '备注', multiline: true },
    { key: 'projectIds', label: '相关项目', relation: 'projects', multiple: true },
  ] },
  { entity: 'dataRecords', label: '数据记录', singular: '数据记录', icon: '▦', titleKey: 'title', description: '未来扩展的结构化本地数据。', fields: [{ key: 'type', label: '类型' }, { key: 'title', label: '标题' }, { key: 'dataJson', label: 'JSON 数据', multiline: true }, { key: 'source', label: '来源' }] },
  { entity: 'attachments', label: '附件', singular: '附件', icon: '⊞', titleKey: 'fileName', description: '保存在本地文件系统的附件元数据。', fields: [{ key: 'fileName', label: '文件名' }, { key: 'path', label: '本地路径' }, { key: 'mimeType', label: 'MIME 类型' }, { key: 'relatedEntityType', label: '关联实体类型' }, { key: 'relatedEntityId', label: '关联实体 ID' }] },
  { entity: 'timelineEvents', label: '时间线事件', singular: '时间线事件', icon: '⌁', titleKey: 'title', description: '保存关键状态变化的不可变证据，不作为普通内容手动创建。', fields: [
    { key: 'title', label: '事件' }, { key: 'eventType', label: '事件类型' }, { key: 'occurredAt', label: '发生时间', type: 'datetime-local' },
    { key: 'timeMeaning', label: '时间语义' }, { key: 'timelineImportance', label: '重要性' }, { key: 'evidenceLevel', label: '证据等级' },
    { key: 'sourceEntityType', label: '来源类型' }, { key: 'sourceEntityId', label: '来源记录 ID' }, { key: 'beforeValue', label: '变更前' }, { key: 'afterValue', label: '变更后' },
    { key: 'goalId', label: '目标', relation: 'goals' }, { key: 'projectId', label: '项目', relation: 'projects' }, { key: 'taskId', label: '任务', relation: 'tasks' },
  ] },
  { entity: 'agentRuns', label: 'AI 运行', singular: 'AI 运行', icon: 'AI', titleKey: 'input', description: 'AI 分析的审计记录。', fields: [] },
  { entity: 'agentActions', label: 'AI 操作', singular: 'AI 操作', icon: '→', titleKey: 'actionType', description: 'AI 实际动作的审计记录。', fields: [] },
]

export const configFor = (entity: Entity) => entities.find((item) => item.entity === entity)!
export const titleFor = (record: Partial<RecordData>) => String(record.title || record.name || record.statement || record.content || record.context || '未命名')
export const descriptionFor = (record: Partial<RecordData>) => String(record.description || record.why || record.statement || record.content || record.problem || record.actual || record.actualResult || record.lesson || record.corePrinciple || record.framework || record.notes || '')
export const statusLabel = (value: unknown) => ({ active: '进行中', planned: '计划中', paused: '已暂停', completed: '已完成', archived: '已归档', blocked: '受阻', healthy: '健康', at_risk: '有风险', inbox: '收集箱', todo: '待办', in_progress: '进行中', waiting: '等待中', cancelled: '已取消', untested: '未测试', testing: '测试中', validated: '已验证', rejected: '已否定', inconclusive: '无结论', pending: '待决定', decided: '已决定', monitoring: '观察中', partially_correct: '部分正确', wrong: '错误', unknown: '未知', unprocessed: '待处理', processed: '已处理', running: '进行中' }[String(value)] || String(value || '未设置'))
export const priorityLabel = (value: unknown) => ({ high: '高', medium: '中', low: '低' }[String(value)] || String(value || '未设置'))
export const isActive = (record: Partial<RecordData>) => !['completed', 'archived', 'processed', 'cancelled', 'validated', 'rejected', 'wrong', '已完成', '已归档', '已处理'].includes(String(record.status || 'active'))
export const localDateKey = (value: Date | string | number = new Date()) => { const date = value instanceof Date ? value : new Date(Number(value) || value); return Number.isNaN(date.getTime()) ? '' : `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}` }
export const recordDate = timelineOccurredAt
export const isToday = (value: unknown) => localDateKey(String(value || '')) === localDateKey()
export const isOverdue = (record: Partial<RecordData>) => Boolean(record.dueDate) && localDateKey(String(record.dueDate)) < localDateKey() && !['completed', 'cancelled'].includes(String(record.status))
export const durationMinutes = (record: Partial<RecordData>) => Number(record.durationMinutes || record.duration || 0)
export const minutesToday = (records: RecordData[]) => records.filter((record) => record.entity === 'timeLogs' && isToday(record.startAt)).reduce((total, record) => total + durationMinutes(record), 0)
export const timeline = timelineRecords
export const linkedTo = (record: Partial<RecordData>, id: string) => Object.entries(record).some(([key, value]) => (key.endsWith('Id') && value === id) || (key.endsWith('Ids') && (Array.isArray(value) ? value.includes(id) : String(value || '').split(',').map((item) => item.trim()).includes(id))))
export const percent = (value: unknown) => Math.max(0, Math.min(100, Number(value || 0)))
