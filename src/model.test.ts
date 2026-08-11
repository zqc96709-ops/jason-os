import { describe, expect, it } from 'vitest'
import { isActive, isOverdue, linkedTo, minutesToday, timeline, type RecordData } from './model'

const record = (entity: RecordData['entity'], extra: Record<string, unknown> = {}) => ({ id: Math.random().toString(), entity, createdAt: '1', updatedAt: '2', ...extra }) as RecordData

describe('Jason OS computed views', () => {
  it('keeps completed work out of active focus', () => { expect(isActive(record('tasks', { status: 'completed' }))).toBe(false); expect(isActive(record('goals', { status: 'active' }))).toBe(true) })
  it('totals only today’s recorded reality', () => { const now = new Date(); expect(minutesToday([record('timeLogs', { startAt: now.toISOString(), durationMinutes: 45 }), record('timeLogs', { startAt: '2020-01-01', durationMinutes: 99 })])).toBe(45) })
  it('uses stable occurrence dates and excludes non-timeline entities', () => { expect(timeline([record('tasks', { createdAt: '2026-08-09' }), record('knowledge'), record('decisions', { createdAt: '2026-08-10' })]).map((item) => item.entity)).toEqual(['decisions', 'tasks']) })
  it('detects overdue tasks but ignores completed tasks', () => { expect(isOverdue(record('tasks', { dueDate: '2020-01-01', status: 'todo' }))).toBe(true); expect(isOverdue(record('tasks', { dueDate: '2020-01-01', status: 'completed' }))).toBe(false) })
  it('recognizes direct and multi-value relations', () => { expect(linkedTo(record('tasks', { projectId: 'p1' }), 'p1')).toBe(true); expect(linkedTo(record('knowledge', { projectIds: ['p1', 'p2'] }), 'p2')).toBe(true) })
})
