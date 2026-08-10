import { describe, expect, it } from 'vitest'
import { calendarDateKey, calendarRange, parseCalendarTime, taskCalendarItems } from './calendar'
import type { RecordData } from './model'
const task = (id: string, data: Partial<RecordData>) => ({ id, entity: 'tasks', createdAt: '', updatedAt: '', title: id, ...data }) as RecordData

describe('Lark CLI compatible calendar rules', () => {
  it('snaps date-only values to local day boundaries', () => {
    expect(parseCalendarTime('2026-08-10')?.getHours()).toBe(0)
    const end = parseCalendarTime('2026-08-10', true)!
    expect([end.getHours(), end.getMinutes(), end.getSeconds()]).toEqual([23, 59, 59])
  })
  it('starts weeks on Monday and returns a seven-day inclusive range', () => {
    const range = calendarRange(new Date(2026, 7, 12), 'week')
    expect(calendarDateKey(range.start)).toBe('2026-08-10')
    expect(calendarDateKey(range.end)).toBe('2026-08-16')
  })
  it('filters cancelled tasks, deduplicates, separates all-day and timed tasks, and sorts', () => {
    const timed = task('timed', { dueAt: '2026-08-10T09:30', estimateMinutes: 60 })
    const items = taskCalendarItems([task('later', { dueDate: '2026-08-11' }), timed, timed, task('cancelled', { dueDate: '2026-08-10', status: 'cancelled' })])
    expect(items.map((item) => item.id)).toEqual(['timed', 'later'])
    expect(items[0].allDay).toBe(false)
    expect(items[1].allDay).toBe(true)
  })
})
