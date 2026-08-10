import type { Entity, RecordData } from '../model'

export type AgentRiskLevel = 'READ' | 'LOW_WRITE' | 'MEDIUM_WRITE' | 'HIGH_RISK'
export type AgentActionStatus = 'PENDING' | 'CONFIRM_REQUIRED' | 'EXECUTING' | 'SUCCESS' | 'FAILED' | 'CANCELLED'
export type AgentIntent = 'CREATE' | 'READ' | 'UPDATE' | 'DELETE' | 'SEARCH' | 'START_TIMER' | 'STOP_TIMER' | 'ANALYZE' | 'SAVE' | 'LINK' | 'COMPLETE' | 'REVIEW' | 'EXTRACT' | 'SUMMARIZE' | 'CONVERT_TO_MENTAL_MODEL' | 'CONVERT_TO_DECISION' | 'SAVE_MENTAL_MODEL' | string

export type AgentContext = {
  currentRoute: string
  currentEntityType?: Entity
  currentEntityId?: string
  currentGoalId?: string
  currentProjectId?: string
  currentTaskId?: string
  currentUser: string
  selectedItems: string[]
  recentConversation: { role: 'user' | 'assistant'; content: string }[]
  recentActions: Pick<RecordData, 'id' | 'entity'>[] & Record<string, unknown>[]
  localDate: string
  timeZone: string
}

export type AgentAction = {
  actionId: string
  intent: AgentIntent
  toolName: string
  entityType: Entity
  entityId?: string
  input: Record<string, unknown>
  status: AgentActionStatus
  riskLevel: AgentRiskLevel
  requiresConfirmation: boolean
  userConfirmed: boolean
  idempotencyKey: string
  previewTitle: string
  previewFields: { label: string; value: string }[]
  result?: RecordData | Record<string, unknown>
  error?: string
  createdAt: string
  completedAt?: string
}

export type ChatMessage = {
  role: 'user' | 'assistant'
  content: string
  action?: AgentAction
}

export type AgentResponse = {
  answer: string
  context: RecordData[]
  action?: AgentAction
}

export type AgentActionResult = {
  action: AgentAction
  record?: RecordData
  duplicate?: boolean
}

export type JsonFieldSchema = {
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  description?: string
  enum?: string[]
  items?: { type: 'string' | 'number' | 'object' }
}

export type JsonObjectSchema = {
  type: 'object'
  properties: Record<string, JsonFieldSchema>
  required?: string[]
  additionalProperties?: boolean
}

export type SchemaDefinition = {
  entityName: Entity
  description: string
  fields: string[]
  requiredFields: string[]
  relations: Record<string, Entity>
  allowedActions: string[]
  validationRules: string[]
}

export type ToolDefinition = {
  name: string
  description: string
  inputSchema: JsonObjectSchema
  outputSchema: JsonObjectSchema
  entity: Entity
  actionType: 'READ' | 'CREATE' | 'UPDATE' | 'COMPLETE' | 'START_TIMER' | 'STOP_TIMER'
  riskLevel: AgentRiskLevel
  requiresConfirmation: boolean
  idempotencyKey: string
  permission: 'local_read' | 'local_write'
}
