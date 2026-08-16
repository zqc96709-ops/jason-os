import type { Entity, RecordData } from '../model'
import type { AgentContext, ChatMessage } from './types'

const routeEntity: Partial<Record<string, Entity>> = { tasks: 'tasks', time: 'timeLogs', projects: 'projects', outcomes: 'results', finance: 'financialTransactions', knowledge: 'knowledge', reviews: 'reviews', insights: 'insights', principles: 'principles', mentalModels: 'mentalModels', notebook: 'notes', decisions: 'decisions', events: 'events', people: 'people', externalIntelligence: 'signals' }
const stringValue = (value: unknown) => typeof value === 'string' && value ? value : undefined

export function buildAgentContext({ currentRoute, records, selectedProjectId, detailId, conversation }: { currentRoute: string; records: RecordData[]; selectedProjectId: string | null; detailId: string | null; conversation: ChatMessage[] }): AgentContext {
  const current = records.find((record) => record.id === detailId) || records.find((record) => record.id === selectedProjectId)
  const project = current?.entity === 'projects' ? current : records.find((record) => record.entity === 'projects' && record.id === current?.projectId)
  const task = current?.entity === 'tasks' ? current : records.find((record) => record.entity === 'tasks' && record.id === current?.taskId)
  const goalId = stringValue(current?.goalId) || stringValue(project?.goalId) || (current?.entity === 'goals' ? current.id : undefined)
  return {
    currentRoute,
    currentEntityType: current?.entity || routeEntity[currentRoute],
    currentEntityId: current?.id,
    currentGoalId: goalId,
    currentProjectId: current?.entity === 'projects' ? current.id : stringValue(current?.projectId) || project?.id,
    currentTaskId: task?.id,
    currentUser: 'local-user',
    selectedItems: [selectedProjectId, detailId].filter((value): value is string => Boolean(value)),
    recentConversation: conversation.slice(-10).map(({ role, content }) => ({ role, content })),
    recentActions: records.filter((record) => record.entity === 'agentActions').slice(0, 10).map((record) => ({
      id: record.id,
      entity: record.entity,
      actionType: record.actionType,
      intent: record.intent,
      status: record.status,
      previewTitle: record.previewTitle,
      createdAt: record.createdAt,
      completedAt: record.completedAt,
    })),
    localDate: new Date().toLocaleDateString('en-CA'),
    timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  }
}
