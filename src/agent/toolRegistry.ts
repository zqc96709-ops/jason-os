import { schemaFor } from './schemaRegistry'
import type { AgentRiskLevel, JsonObjectSchema, ToolDefinition } from './types'
import type { Entity } from '../model'

const outputSchema: JsonObjectSchema = { type: 'object', properties: { ok: { type: 'boolean' }, entityId: { type: 'string' }, message: { type: 'string' } }, required: ['ok'], additionalProperties: true }
const inputFor = (entity: Entity, update = false): JsonObjectSchema => {
  const schema = schemaFor(entity)
  const fields = entity === 'notes' ? ['title', 'content', 'type', 'status'] : (schema?.fields || [])
  return { type: 'object', properties: Object.fromEntries(fields.map((field) => [field, { type: field.endsWith('Ids') ? 'array' : 'string' }])), required: update ? ['id'] : schema?.requiredFields, additionalProperties: false }
}
const tool = (name: string, description: string, entity: Entity, actionType: ToolDefinition['actionType'], riskLevel: AgentRiskLevel, requiresConfirmation: boolean): ToolDefinition => {
  const inputSchema = inputFor(entity, actionType === 'UPDATE' || actionType === 'COMPLETE')
  if (actionType === 'UPDATE' || actionType === 'COMPLETE') inputSchema.properties.id = { type: 'string', description: '必须来自真实本地记录' }
  if (actionType === 'START_TIMER' || actionType === 'STOP_TIMER') inputSchema.required = []
  return { name, description, entity, actionType, riskLevel, requiresConfirmation, inputSchema, outputSchema, idempotencyKey: `${name}:normalized-input`, permission: actionType === 'READ' ? 'local_read' : 'local_write' }
}
const importWorkspaceDocumentToNotebook: ToolDefinition = {
  name: 'importWorkspaceDocumentToNotebook',
  description: '把用户明确指定、且位于 Jason OS docs/ 目录内的本地文档复制到 Notebook Inbox。源文件保持不变，执行前必须确认。',
  entity: 'notebookFiles', actionType: 'CREATE', riskLevel: 'MEDIUM_WRITE', requiresConfirmation: true,
  inputSchema: { type: 'object', properties: { name: { type: 'string', description: '保存到 Notebook 后显示的文件名' }, sourcePath: { type: 'string', description: '用户明确提供的 Jason OS docs 文件路径' }, notebookCategoryId: { type: 'string' }, notebookFolderId: { type: 'string' } }, required: ['name', 'sourcePath'], additionalProperties: false },
  outputSchema, idempotencyKey: 'importWorkspaceDocumentToNotebook:normalized-input', permission: 'local_write',
}

export const toolRegistry: ToolDefinition[] = [
  tool('readWorkspaceDocument', '读取用户明确指定、且位于 Jason OS docs/ 目录内的本地 Markdown、TXT、JSON 或 CSV 文档；只读，不修改文件。', 'dataRecords', 'READ', 'READ', false),
  tool('getProfile', '读取当前用户的我的档案与 AI 上下文；仅在当前问题相关时使用。', 'profiles', 'READ', 'READ', false),
  tool('updateProfile', '更新当前用户的我的档案；必须先向用户展示变更预览并等待确认。', 'profiles', 'UPDATE', 'MEDIUM_WRITE', true),
  importWorkspaceDocumentToNotebook,
  tool('createInbox', '保存用户明确提供的日常工作内容到收集箱，保留原始事实，确认后写入', 'inbox', 'CREATE', 'LOW_WRITE', true),
  tool('createMentalModel', '保存结构化思维模型到思维模型库', 'mentalModels', 'CREATE', 'LOW_WRITE', true),
  tool('getMentalModel', '读取一个思维模型', 'mentalModels', 'READ', 'READ', false),
  tool('searchMentalModels', '搜索可用于当前问题的思维模型', 'mentalModels', 'READ', 'READ', false),
  tool('updateMentalModel', '更新已有思维模型', 'mentalModels', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createNote', '把一段内容保存为 Notebook 自由笔记', 'notes', 'CREATE', 'LOW_WRITE', true),
  tool('getNote', '读取一条 Notebook 笔记', 'notes', 'READ', 'READ', false),
  tool('searchNotes', '搜索历史 Notebook 笔记', 'notes', 'READ', 'READ', false),
  tool('updateNote', '更新已有 Notebook 笔记', 'notes', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createNotebookCategory', '创建 Notebook 自定义分类；不是项目', 'notebookCategories', 'CREATE', 'LOW_WRITE', true),
  tool('getNotebookCategories', '读取 Notebook 分类', 'notebookCategories', 'READ', 'READ', false),
  tool('updateNotebookCategory', '更新 Notebook 分类', 'notebookCategories', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createNotebookFolder', '创建 Notebook 文件夹或子文件夹', 'notebookFolders', 'CREATE', 'LOW_WRITE', true),
  tool('getNotebookFolders', '读取 Notebook 文件夹', 'notebookFolders', 'READ', 'READ', false),
  tool('updateNotebookFolder', '更新 Notebook 文件夹', 'notebookFolders', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('getNotebookFiles', '读取 Notebook 文件元数据', 'notebookFiles', 'READ', 'READ', false),
  tool('searchNotebookFiles', '按文件名、类型、目录和元数据搜索 Notebook 文件', 'notebookFiles', 'READ', 'READ', false),
  tool('updateNotebookFile', '更新 Notebook 文件元数据或移动文件；不修改原始文件内容', 'notebookFiles', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createKnowledge', '创建知识记录', 'knowledge', 'CREATE', 'LOW_WRITE', true), tool('getKnowledge', '读取知识', 'knowledge', 'READ', 'READ', false), tool('searchKnowledge', '搜索知识', 'knowledge', 'READ', 'READ', false), tool('updateKnowledge', '更新知识', 'knowledge', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createGoal', '创建目标', 'goals', 'CREATE', 'LOW_WRITE', true), tool('getGoal', '读取目标', 'goals', 'READ', 'READ', false), tool('searchGoals', '搜索目标', 'goals', 'READ', 'READ', false), tool('updateGoal', '更新目标', 'goals', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createProject', '创建项目并继承当前目标', 'projects', 'CREATE', 'LOW_WRITE', true), tool('getProject', '读取项目', 'projects', 'READ', 'READ', false), tool('searchProjects', '搜索项目', 'projects', 'READ', 'READ', false), tool('updateProject', '更新项目', 'projects', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createTask', '创建任务并继承当前项目和目标', 'tasks', 'CREATE', 'LOW_WRITE', true), tool('getTask', '读取任务', 'tasks', 'READ', 'READ', false), tool('searchTasks', '搜索任务', 'tasks', 'READ', 'READ', false), tool('updateTask', '更新任务', 'tasks', 'UPDATE', 'MEDIUM_WRITE', true), tool('completeTask', '完成任务', 'tasks', 'COMPLETE', 'MEDIUM_WRITE', true),
  tool('startTimer', '开始与当前任务、项目或目标关联的计时', 'timeLogs', 'START_TIMER', 'LOW_WRITE', true), tool('stopTimer', '停止当前计时并生成时间记录', 'timeLogs', 'STOP_TIMER', 'MEDIUM_WRITE', true), tool('createTimeRecord', '创建时间记录', 'timeLogs', 'CREATE', 'LOW_WRITE', true), tool('getTimeRecords', '读取时间记录', 'timeLogs', 'READ', 'READ', false),
  tool('createDecision', '创建结构化决策', 'decisions', 'CREATE', 'MEDIUM_WRITE', true), tool('getDecision', '读取决策', 'decisions', 'READ', 'READ', false), tool('searchDecisions', '搜索决策', 'decisions', 'READ', 'READ', false), tool('updateDecision', '更新决策', 'decisions', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createReview', '创建复盘', 'reviews', 'CREATE', 'LOW_WRITE', true), tool('getReview', '读取复盘', 'reviews', 'READ', 'READ', false), tool('createInsight', '创建洞见', 'insights', 'CREATE', 'LOW_WRITE', true), tool('createPrinciple', '创建原则', 'principles', 'CREATE', 'LOW_WRITE', true), tool('searchPrinciples', '搜索原则', 'principles', 'READ', 'READ', false),
  tool('createExternalSource', '创建外部情报监控源', 'externalSources', 'CREATE', 'MEDIUM_WRITE', true), tool('getExternalSources', '读取情报源与同步状态', 'externalSources', 'READ', 'READ', false), tool('updateExternalSource', '更新或暂停情报源', 'externalSources', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('getExternalBriefing', '读取有证据的 CEO 外部情报简报', 'intelligenceBriefs', 'READ', 'READ', false), tool('getExternalSignals', '读取外部信号及证据口径', 'signals', 'READ', 'READ', false), tool('updateExternalSignal', '观察、忽略或验证外部信号', 'signals', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createOpportunity', '把已确认信号转为机会草稿', 'opportunities', 'CREATE', 'LOW_WRITE', true), tool('getOpportunities', '读取机会与数据缺口', 'opportunities', 'READ', 'READ', false), tool('updateOpportunity', '更新机会评估', 'opportunities', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createOutcome', '创建带预期、实际和证据状态的 Outcome 草稿', 'results', 'CREATE', 'MEDIUM_WRITE', true), tool('getOutcomes', '读取 Outcome 与预期实际偏差', 'results', 'READ', 'READ', false), tool('updateOutcome', '更新 Outcome 实际结果与证据', 'results', 'UPDATE', 'MEDIUM_WRITE', true),
  tool('createFinancialAccount', '创建资金账户', 'financialAccounts', 'CREATE', 'MEDIUM_WRITE', true), tool('getFinancialAccounts', '读取账户与核验状态', 'financialAccounts', 'READ', 'READ', false),
  tool('createFinancialCategory', '创建经营分类', 'financialCategories', 'CREATE', 'LOW_WRITE', true), tool('getFinancialCategories', '读取经营分类', 'financialCategories', 'READ', 'READ', false),
  tool('createFinancialTransaction', '创建财务流水草稿；不得直接入账', 'financialTransactions', 'CREATE', 'HIGH_RISK', true), tool('getFinancialTransactions', '读取财务流水事实', 'financialTransactions', 'READ', 'READ', false), tool('getProjectFinancials', '读取项目时间、资金、Outcome 与数据缺口', 'financialTransactions', 'READ', 'READ', false), tool('voidFinancialTransaction', '作废已入账流水并保留原因', 'financialTransactions', 'UPDATE', 'HIGH_RISK', true),
]

export const toolFor = (name: string) => toolRegistry.find((item) => item.name === name)
