import type { AgentAction, ChatMessage } from './types'

export type ActionBundleMessage = ChatMessage & { actions?: AgentAction[] }

export function actionsForMessage(message: ChatMessage): AgentAction[] {
  const candidate = message as ActionBundleMessage
  const actions = candidate.actions?.length ? candidate.actions : candidate.action ? [candidate.action] : []
  const seen = new Set<string>()
  return actions.filter((action) => {
    if (seen.has(action.actionId)) return false
    seen.add(action.actionId)
    return true
  })
}

export function replaceActionInMessages(messages: ChatMessage[], next: AgentAction): ChatMessage[] {
  return messages.map((message) => {
    const candidate = message as ActionBundleMessage
    const actions = actionsForMessage(message)
    if (!actions.some((action) => action.actionId === next.actionId)) return message
    const replaced = actions.map((action) => action.actionId === next.actionId ? next : action)
    return { ...candidate, action: replaced.length === 1 ? replaced[0] : candidate.action, actions: replaced.length > 1 ? replaced : undefined }
  })
}

export function summarizeActionBundle(actions: AgentAction[]) {
  return actions.reduce((summary, action) => {
    summary.total += 1
    if (['CONFIRM_REQUIRED', 'PENDING', 'EXECUTING'].includes(action.status)) summary.pending += 1
    if (action.status === 'SUCCESS') summary.succeeded += 1
    if (action.status === 'FAILED') summary.failed += 1
    return summary
  }, { total: 0, pending: 0, succeeded: 0, failed: 0 })
}
