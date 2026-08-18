import { describe, expect, it } from 'vitest'
import { buildDailyBrief, buildFollowUpQueue, followUpRecordsWithEvidence, prepareInboxCapture, reviewInitialForResult, selectFollowUpsForDisplay, workbenchDateLabel } from './workbenchDaily'
import type { RecordData } from './model'

const record = (id: string, entity: RecordData['entity'], fields: Record<string, unknown> = {}): RecordData => ({
  id,
  entity,
  createdAt: '2026-08-16T08:00:00.000Z',
  updatedAt: '2026-08-16T08:00:00.000Z',
  ...fields,
})

describe('buildDailyBrief', () => {
  it('prioritizes overdue and high-priority work with evidence', () => {
    const brief = buildDailyBrief([
      record('task-high', 'tasks', { title: '回访高价值客户', status: 'todo', priority: 'high', dueDate: '2026-08-15', tags: 'ToAPIs,客户' }),
      record('task-low', 'tasks', { title: '整理桌面', status: 'todo', priority: 'low', dueDate: '2026-08-20' }),
    ], new Date('2026-08-16T09:00:00+08:00'))

    expect(brief.actions[0]).toMatchObject({ recordId: 'task-high', lane: 'business' })
    expect(brief.actions[0].reason).toContain('已逾期')
    expect(brief.actions[0].reason).toContain('高优先级')
  })

  it('keeps business, product and personal work in distinct lanes', () => {
    const brief = buildDailyBrief([
      record('business', 'tasks', { title: '确认客户复购机会', status: 'todo', tags: 'ToAPIs,客户' }),
      record('product', 'projects', { title: '创作台批量流程', status: 'active', nextAction: '核对失败重试', tags: '产品' }),
      record('personal', 'tasks', { title: '跑步机训练 30 分钟', status: 'todo', tags: '健身' }),
    ], new Date('2026-08-16T09:00:00+08:00'))

    expect(brief.actions.map((item) => item.lane)).toEqual(['business', 'product', 'personal'])
  })

  it('shows an honest empty state instead of inventing priorities', () => {
    const brief = buildDailyBrief([], new Date('2026-08-16T09:00:00+08:00'))
    expect(brief.actions).toEqual([])
    expect(brief.observation).toContain('真实记录不足')
  })

  it('rejects an empty inbox capture before it can create a fake Daily Brief item', () => {
    expect(() => prepareInboxCapture('   ')).toThrow('请先输入正在发生的事')
  })

  it('preserves the spoken fact when preparing an inbox record', () => {
    expect(prepareInboxCapture('今天先回访高价值客户')).toEqual({ content: '今天先回访高价值客户', status: 'unprocessed' })
  })

  it('ignores malformed empty inbox rows in the Daily Brief', () => {
    const brief = buildDailyBrief([record('empty-inbox', 'inbox', { status: 'unprocessed' })])
    expect(brief.actions).toEqual([])
  })

  it('keeps a legacy completedAt task out of the action brief', () => {
    const brief = buildDailyBrief([
      record('legacy-done', 'tasks', { title: '旧任务', status: 'todo', completedAt: '2026-08-17T18:00:00+08:00', dueDate: '2026-08-17' }),
    ], new Date(2026, 7, 18, 9, 0, 0))

    expect(brief.actions).toEqual([])
  })

  it('uses the same local calendar day as follow-up reasons', () => {
    const now = new Date(2026, 7, 18, 1, 0, 0)
    now.toISOString = () => '2026-08-17T17:00:00.000Z'
    const brief = buildDailyBrief([
      record('overdue-local', 'tasks', { title: '本地昨日任务', status: 'todo', dueDate: '2026-08-17' }),
    ], now)

    expect(brief.actions[0].reason).toContain('已逾期')
  })
})

describe('buildFollowUpQueue', () => {
  const now = new Date(2026, 7, 18, 9, 0, 0)

  it('surfaces overdue and today tasks as evidence-backed action reminders', () => {
    const queue = buildFollowUpQueue([
      record('overdue', 'tasks', { title: '核对失败重试', status: 'todo', dueDate: '2026-08-17' }),
      record('today', 'tasks', { title: '回访重点客户', status: 'todo', dueDate: '2026-08-18' }),
      record('future', 'tasks', { title: '下周整理', status: 'todo', dueDate: '2026-08-25' }),
    ], now)

    expect(queue.map((item) => item.sourceId)).toEqual(['overdue', 'today'])
    expect(queue[0]).toMatchObject({ stage: 'action', reason: '任务已逾期', cta: '打开任务' })
    expect(queue[1]).toMatchObject({ stage: 'action', reason: '任务今天到期', cta: '打开任务' })
  })

  it('asks for a result after a completed task without guessing whether it succeeded', () => {
    const queue = buildFollowUpQueue([
      record('task-done', 'tasks', { title: '核对失败重试', status: 'completed', completedAt: '2026-08-18T02:00:00.000Z', projectId: 'project-1', goalId: 'goal-1' }),
    ], now)

    expect(queue).toHaveLength(1)
    expect(queue[0]).toMatchObject({
      stage: 'result', sourceId: 'task-done', cta: '回填结果', createEntity: 'results',
      initial: { title: '结果：核对失败重试', taskId: 'task-done', projectId: 'project-1', goalId: 'goal-1', date: '2026-08-18' },
    })
    expect(queue[0].initial?.status).toBeUndefined()
  })

  it('uses only an explicit taskId result to close the result reminder', () => {
    const queue = buildFollowUpQueue([
      record('task-done', 'tasks', { title: '核对失败重试', status: 'completed', completedAt: '2026-08-18T02:00:00.000Z', projectId: 'project-1' }),
      record('unrelated-result', 'results', { title: '同项目的其他结果', projectId: 'project-1', actual: '已记录' }),
    ], now)

    expect(queue.some((item) => item.stage === 'result' && item.sourceId === 'task-done')).toBe(true)
  })

  it('does not treat a planned outcome as the actual result of a completed task', () => {
    const queue = buildFollowUpQueue([
      record('task-done', 'tasks', { title: '核对失败重试', status: 'completed', completedAt: '2026-08-18T02:00:00.000Z' }),
      record('planned-result', 'results', { title: '预期结果', taskId: 'task-done', status: 'PLANNED', expected: '重试不重复扣费' }),
    ], now)

    expect(queue.some((item) => item.stage === 'result' && item.sourceId === 'task-done')).toBe(true)
    expect(queue.some((item) => item.stage === 'review' && item.sourceId === 'planned-result')).toBe(false)
  })

  it('treats completedAt as completed even when a legacy status still says todo', () => {
    const queue = buildFollowUpQueue([
      record('legacy-done', 'tasks', { title: '旧任务', status: 'todo', completedAt: '2026-08-17T18:00:00+08:00', dueDate: '2026-08-17' }),
    ], now)

    expect(queue.some((item) => item.stage === 'action' && item.sourceId === 'legacy-done')).toBe(false)
    expect(queue).toContainEqual(expect.objectContaining({ stage: 'result', sourceId: 'legacy-done' }))
  })

  it('moves an explicitly linked result into the review stage without repeating the result reminder', () => {
    const queue = buildFollowUpQueue([
      record('task-done', 'tasks', { title: '核对失败重试', status: 'completed', completedAt: '2026-08-18T02:00:00.000Z', projectId: 'project-1', goalId: 'goal-1' }),
      record('result-1', 'results', { title: '失败重试核对结果', taskId: 'task-done', projectId: 'project-1', goalId: 'goal-1', actual: '重试不会重复扣费', date: '2026-08-18' }),
    ], now)

    expect(queue.some((item) => item.stage === 'result' && item.sourceId === 'task-done')).toBe(false)
    expect(queue).toContainEqual(expect.objectContaining({
      stage: 'review', sourceId: 'result-1', cta: '开始复盘', createEntity: 'reviews',
      initial: { title: '复盘：失败重试核对结果', resultId: 'result-1', taskId: 'task-done', projectId: 'project-1', goalId: 'goal-1', whatHappened: '重试不会重复扣费' },
    }))
  })

  it('removes a result from the queue once an explicit review exists', () => {
    const queue = buildFollowUpQueue([
      record('result-1', 'results', { title: '失败重试核对结果', taskId: 'task-done' }),
      record('review-1', 'reviews', { title: '失败重试复盘', resultId: 'result-1' }),
    ], now)

    expect(queue).toEqual([])
  })

  it('counts an archived review as existing evidence without surfacing it again', () => {
    const queue = buildFollowUpQueue([
      record('result-1', 'results', { title: '已落地结果', status: 'ACHIEVED', actual: '真实结果' }),
      record('review-1', 'reviews', { resultId: 'result-1', archivedAt: '2026-08-18T10:00:00+08:00' }),
    ], now)

    expect(queue).toEqual([])
  })

  it('counts an archived actual result as evidence without asking to refill or review it', () => {
    const queue = buildFollowUpQueue([
      record('task-done', 'tasks', { title: '已完成任务', status: 'completed', completedAt: '2026-08-17T18:00:00+08:00' }),
      record('result-1', 'results', { taskId: 'task-done', status: 'ACHIEVED', actual: '真实结果', archivedAt: '2026-08-18T10:00:00+08:00' }),
    ], now)

    expect(queue).toEqual([])
  })

  it('accepts a trimmed legacy actualResult but rejects blank actual text', () => {
    const queue = buildFollowUpQueue([
      record('legacy-result', 'results', { title: '旧结果', actualResult: '  已完成真实交付  ' }),
      record('blank-result', 'results', { title: '空白结果', actual: '   ' }),
    ], now)

    expect(queue).toContainEqual(expect.objectContaining({ stage: 'review', sourceId: 'legacy-result', initial: expect.objectContaining({ whatHappened: '已完成真实交付' }) }))
    expect(queue.some((item) => item.sourceId === 'blank-result')).toBe(false)
  })

  it('prefills review context for zero, money, evidence-only and terminal-status results', () => {
    const queue = buildFollowUpQueue([
      record('zero-result', 'results', { title: '零增长', actualValue: 0, unit: '人' }),
      record('money-result', 'results', { title: '收入结果', actualAmountMinor: 0, currency: 'CNY' }),
      record('evidence-result', 'results', { title: '证据结果', evidence: { url: 'proof' } }),
      record('status-result', 'results', { title: '终态结果', status: 'MISSED' }),
    ], now)

    const context = Object.fromEntries(queue.map((item) => [item.sourceId, item.initial?.whatHappened]))
    expect(context['zero-result']).toBe('实际数值：0 人')
    expect(context['money-result']).toBe('实际金额（最小货币单位）：0 CNY')
    expect(context['evidence-result']).toBe('已记录结果证据')
    expect(context['status-result']).toBe('结果状态：未达成')
  })
})

describe('reviewInitialForResult', () => {
  it('keeps the drawer conversion aligned with follow-up review context', () => {
    const result = record('result-zero', 'results', {
      title: '零增长结果',
      taskId: 'task-1',
      projectId: 'project-1',
      goalId: 'goal-1',
      actualValue: 0,
      unit: '人',
    })

    expect(reviewInitialForResult(result)).toEqual({
      title: '复盘：零增长结果',
      resultId: 'result-zero',
      taskId: 'task-1',
      projectId: 'project-1',
      goalId: 'goal-1',
      whatHappened: '实际数值：0 人',
    })
  })
})

describe('workbenchDateLabel', () => {
  it('formats the local calendar day used by follow-up classification', () => {
    expect(workbenchDateLabel(new Date(2026, 7, 18, 1, 0, 0))).toContain('8月18日')
  })
})

describe('selectFollowUpsForDisplay', () => {
  it('keeps every non-empty stage reachable within a six-row homepage limit', () => {
    const queue = [
      ...Array.from({ length: 7 }, (_, index) => ({ key: `action:${index}`, stage: 'action' as const, sourceId: `task-${index}`, title: `行动 ${index}`, reason: '到期', cta: '打开任务' })),
      { key: 'result:1', stage: 'result' as const, sourceId: 'task-done', title: '等待结果', reason: '已完成', cta: '回填结果' },
      { key: 'review:1', stage: 'review' as const, sourceId: 'result-1', title: '等待复盘', reason: '已有结果', cta: '开始复盘' },
    ]

    const visible = selectFollowUpsForDisplay(queue, 6)

    expect(visible).toHaveLength(6)
    expect(new Set(visible.map((item) => item.stage))).toEqual(new Set(['action', 'result', 'review']))
  })
})

describe('followUpRecordsWithEvidence', () => {
  it('adds only archived result and review evidence to active production records', () => {
    const activeTask = record('task-done', 'tasks', { status: 'completed' })
    const archivedResult = record('result-1', 'results', { taskId: 'task-done', status: 'ACHIEVED', archivedAt: '2026-08-18T10:00:00+08:00' })
    const archivedReview = record('review-1', 'reviews', { resultId: 'result-1', archivedAt: '2026-08-18T10:05:00+08:00' })
    const archivedTask = record('old-task', 'tasks', { archivedAt: '2026-08-18T10:10:00+08:00' })

    expect(followUpRecordsWithEvidence([activeTask], [archivedResult, archivedReview, archivedTask]).map((item) => item.id)).toEqual(['task-done', 'result-1', 'review-1'])
  })
})
