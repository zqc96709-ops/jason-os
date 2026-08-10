import type { RecordData } from './model'

// Calendar range and normalization rules adapted from the MIT-licensed
// larksuite/cli calendar agenda implementation. Meeting rooms, attendees,
// free/busy, RSVP, scheduling suggestions, and video meetings are excluded.
export type CalendarScope = 'day' | 'week' | 'month'
export type TaskCalendarItem = {
  id: string
  record: RecordData
  start: Date
  end: Date
  startDateKey: string
  allDay: boolean
}

export const calendarDateKey = (date: Date) => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function parseCalendarTime(input: unknown, endOfDay = false): Date | null {
  if (input === undefined || input === null || input === '') return null
  const value = String(input).trim()
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    const [year, month, day] = value.split('-').map(Number)
    return new Date(year, month - 1, day, endOfDay ? 23 : 0, endOfDay ? 59 : 0, endOfDay ? 59 : 0)
  }
  if (/^\d+$/.test(value)) {
    const raw = Number(value)
    return new Date(raw > 1e12 ? raw : raw * 1000)
  }
  const parsed = new Date(value)
  return Number.isNaN(parsed.getTime()) ? null : parsed
}

export function calendarRange(anchor: Date, scope: CalendarScope) {
  const start = new Date(anchor)
  start.setHours(0, 0, 0, 0)
  if (scope === 'week') start.setDate(start.getDate() - ((start.getDay() + 6) % 7))
  if (scope === 'month') { start.setDate(1); start.setDate(start.getDate() - ((start.getDay() + 6) % 7)) }
  const end = new Date(start)
  end.setDate(end.getDate() + (scope === 'day' ? 0 : scope === 'week' ? 6 : 41))
  end.setHours(23, 59, 59, 999)
  return { start, end }
}

export function taskCalendarItems(tasks: RecordData[]) {
  const seen = new Set<string>()
  const items: TaskCalendarItem[] = []
  for (const task of tasks) {
    if (task.status === 'cancelled') continue
    const timed = parseCalendarTime(task.dueAt)
    const start = timed || parseCalendarTime(task.dueDate)
    if (!start) continue
    const allDay = !timed
    const end = allDay ? parseCalendarTime(task.dueDate, true)! : new Date(start.getTime() + Math.max(30, Number(task.estimateMinutes || 30)) * 60_000)
    const key = `${task.id}|${start.getTime()}|${end.getTime()}`
    if (seen.has(key)) continue
    seen.add(key)
    items.push({ id: task.id, record: task, start, end, startDateKey: calendarDateKey(start), allDay })
  }
  return items.sort((a, b) => a.start.getTime() - b.start.getTime())
}
