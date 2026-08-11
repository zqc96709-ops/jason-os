import { describe, expect, it } from 'vitest'
import { filterTimelineItems, groupTimelineItems, timelineImportance, timelineOccurredAt, timelineProjection, timelineTimeMeaning, timelineTimestamp } from './timeline'
import type { RecordData } from './model'

const record = (entity: RecordData['entity'], id: string, data: Record<string, unknown> = {}) => ({ id, entity, createdAt: '2026-08-01T10:00:00-07:00', updatedAt: '2026-08-11T10:00:00-07:00', ...data }) as RecordData

describe('timeline evidence foundation', () => {
  it('keeps legacy records anchored to createdAt instead of moving on edit', () => {
    expect(timelineOccurredAt(record('reviews', 'r1'))).toBe('2026-08-01T10:00:00-07:00')
  })

  it('separates planned task dates from actual completion', () => {
    expect(timelineOccurredAt(record('tasks', 'planned', { dueDate: '2026-08-12', status: 'todo' }))).toBe('2026-08-12')
    expect(timelineTimeMeaning(record('tasks', 'planned', { dueDate: '2026-08-12', status: 'todo' }))).toBe('planned')
    expect(timelineOccurredAt(record('tasks', 'done', { dueDate: '2026-08-12', completedAt: '2026-08-11T09:00', status: 'completed' }))).toBe('2026-08-11T09:00')
    expect(timelineTimeMeaning(record('tasks', 'done', { completedAt: '2026-08-11T09:00', status: 'completed' }))).toBe('actual')
  })

  it('marks decisions, results, reviews and insights as key evidence', () => {
    expect(['decisions', 'results', 'reviews', 'insights'].map((entity) => timelineImportance(record(entity as RecordData['entity'], entity)))).toEqual(['key', 'key', 'key', 'key'])
  })

  it('parses date-only values in local time and sorts by occurredAt', () => {
    expect(new Date(timelineTimestamp('2026-08-11')).getDate()).toBe(11)
    const items = timelineProjection([record('reviews', 'old'), record('tasks', 'future', { dueDate: '2026-08-12', status: 'todo' })])
    expect(items.map((item) => item.id)).toEqual(['future', 'old'])
  })

  it('filters key items by range, project and goal without leaking other projects', () => {
    const items = timelineProjection([
      record('decisions', 'd1', { occurredAt: '2026-08-11T09:00:00-07:00', projectId: 'p1', goalId: 'g1' }),
      record('results', 'r1', { occurredAt: '2026-08-10T09:00:00-07:00', projectId: 'p2', goalId: 'g2' }),
      record('tasks', 't1', { occurredAt: '2026-08-11T10:00:00-07:00', projectId: 'p1', goalId: 'g1', status: 'todo' }),
    ])
    const filtered = filterTimelineItems(items, { mode: 'key', range: 'week', entity: 'all', projectId: 'p1', goalId: 'g1' }, new Date(2026, 7, 11))
    expect(filtered.map((item) => item.id)).toEqual(['d1'])
  })

  it('groups filtered items by local calendar date', () => {
    const groups = groupTimelineItems(timelineProjection([
      record('decisions', 'd1', { occurredAt: '2026-08-11T09:00:00-07:00' }),
      record('results', 'r1', { occurredAt: '2026-08-11T08:00:00-07:00' }),
      record('reviews', 'v1', { occurredAt: '2026-08-10T08:00:00-07:00' }),
    ]), new Date(2026, 7, 11))
    expect(groups.map((group) => [group.label, group.items.length])).toEqual([['今天', 2], ['昨天', 1]])
  })

})
