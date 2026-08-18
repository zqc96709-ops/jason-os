import { describe, expect, it } from 'vitest'
import { actionsForMessage, replaceActionInMessages, summarizeActionBundle } from './capturePlanning'
import type { AgentAction, ChatMessage } from './types'

const action = (actionId: string, entityType: AgentAction['entityType'], previewTitle: string): AgentAction => ({
  actionId,
  intent: 'CREATE',
  toolName: entityType === 'projects' ? 'createProject' : entityType === 'tasks' ? 'createTask' : 'createOutcome',
  entityType,
  input: { title: previewTitle },
  status: 'CONFIRM_REQUIRED',
  riskLevel: entityType === 'results' ? 'MEDIUM_WRITE' : 'LOW_WRITE',
  requiresConfirmation: true,
  userConfirmed: false,
  idempotencyKey: `key-${actionId}`,
  previewTitle,
  previewFields: [{ label: '名称', value: previewTitle }],
  createdAt: '2026-08-17T10:00:00+08:00',
})

describe('capture planning action bundles', () => {
  it('keeps one legacy action compatible with the new bundle UI', () => {
    const item = action('a1', 'tasks', '准备创建：回访 licc')
    expect(actionsForMessage({ role: 'assistant', content: '草稿', action: item })).toEqual([item])
  })

  it('deduplicates action and actions while preserving project-task-outcome order', () => {
    const project = action('a1', 'projects', '准备创建：创作台批量流程')
    const task = action('a2', 'tasks', '准备创建：核对失败重试')
    const outcome = action('a3', 'results', '准备创建：已完成实机验证')
    const message: ChatMessage = { role: 'assistant', content: '已拆成 3 项草稿', action: project, actions: [project, task, outcome] }

    expect(actionsForMessage(message).map((item) => item.actionId)).toEqual(['a1', 'a2', 'a3'])
    expect(summarizeActionBundle(actionsForMessage(message))).toEqual({ total: 3, pending: 3, succeeded: 0, failed: 0 })
  })

  it('updates one confirmed draft without changing its siblings', () => {
    const project = action('a1', 'projects', '准备创建：创作台批量流程')
    const task = action('a2', 'tasks', '准备创建：核对失败重试')
    const messages: ChatMessage[] = [{ role: 'assistant', content: '草稿', actions: [project, task] }]
    const saved = { ...task, status: 'SUCCESS' as const, userConfirmed: true }

    const updated = replaceActionInMessages(messages, saved)
    expect(updated[0].actions?.map((item) => item.status)).toEqual(['CONFIRM_REQUIRED', 'SUCCESS'])
  })
})
