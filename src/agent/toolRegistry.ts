import { schemaFor } from './schemaRegistry'
import type { AgentRiskLevel, JsonObjectSchema, ToolDefinition } from './types'
import type { Entity } from '../model'

const outputSchema: JsonObjectSchema = { type: 'object', properties: { ok: { type: 'boolean' }, entityId: { type: 'string' }, message: { type: 'string' } }, required: ['ok'], additionalProperties: true }
const inputFor = (entity: Entity, update = false): JsonObjectSchema => {
  const schema = schemaFor(entity)
  return { type: 'object', properties: Object.fromEntries((schema?.fields || []).map((field) => [field, { type: field.endsWith('Ids') ? 'array' : 'string' }])), required: update ? ['id'] : schema?.requiredFields, additionalProperties: false }
}
const tool = (name: string, description: string, entity: Entity, actionType: ToolDefinition['actionType'], riskLevel: AgentRiskLevel, requiresConfirmation: boolean): ToolDefinition => {
  const inputSchema = inputFor(entity, actionType === 'UPDATE' || actionType === 'COMPLETE')
  if (actionType === 'UPDATE' || actionType === 'COMPLETE') inputSchema.properties.id = { type: 'string', description: '必须来自真实本地记录' }
  if (actionType === 'START_TIMER' || actionType === 'STOP_TIMER') inputSchema.required = []
  return { name, description, entity, actionType, riskLevel, requiresConfirmation, inputSchema, outputSchema, idempotencyKey: `${name}:normalized-input`, permission: actionType === 'READ' ? 'local_read' : 'local_write' }
}

export const toolRegistry: ToolDefinition[] = [
  tool('createMentalModel', '保存结构化思维模型到思维模型库', 'mentalModels', 'CREATE', 'LOW_WRITE', true),
  tool('getMentalModel', '读取一个思维模型', 'mentalModels', 'READ', 'READ', false),
  tool('searchMentalModels', '搜索可用于当前问题的思维模型', 'mentalModels', 'READ', 'READ', false),
  tool('updateMentalModel', '更新已有思维模型', 'mentalModels', 'UPDATE', 'MEDIUM_WRITE', true),
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
]

export const toolFor = (name: string) => toolRegistry.find((item) => item.name === name)
