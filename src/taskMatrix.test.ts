import { describe, expect, it } from 'vitest'
import { taskMatrixValues, taskQuadrant } from './taskMatrix'
import type { RecordData } from './model'

const task = (data: Partial<RecordData>) => data

describe('task importance and urgency matrix', () => {
  it('uses explicit importance and urgency when configured', () => {
    expect(taskQuadrant(task({ importance: 'important', urgency: 'not_urgent' }))).toBe('plan')
    expect(taskQuadrant(task({ importance: 'not_important', urgency: 'urgent' }))).toBe('delegate')
  })

  it('places legacy high-priority overdue tasks in important and urgent', () => {
    expect(taskQuadrant(task({ priority: 'high', dueDate: '2020-01-01', status: 'todo' }))).toBe('do')
  })

  it('keeps legacy undated normal tasks in not important and not urgent', () => {
    expect(taskMatrixValues(task({ priority: 'medium', status: 'todo' }))).toEqual({ importance: 'not_important', urgency: 'not_urgent' })
  })
})
