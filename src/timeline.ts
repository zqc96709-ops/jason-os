import type { Entity, RecordData } from './model'

export type TimelineTimeMeaning = 'planned' | 'actual' | 'recorded'
export type TimelineImportance = 'key' | 'normal'
export type TimelineEvidenceLevel = 'REALITY' | 'USER_CONFIRMED' | 'AI_CONFIRMED' | 'AI_SUGGESTION'

export type TimelineProjectionItem = {
  id: string
  record: RecordData
  occurredAt: string
  timeMeaning: TimelineTimeMeaning
  importance: TimelineImportance
  evidenceLevel: TimelineEvidenceLevel
  goalId?: string
  projectId?: string
  taskId?: string
}

export const timelineEntityTypes: Entity[] = ['goals', 'projects', 'tasks', 'timeLogs', 'events', 'results', 'reviews', 'insights', 'decisions', 'timelineEvents']

const text = (value: unknown) => typeof value === 'string' && value.trim() ? value.trim() : ''
const first = (...values: unknown[]) => values.map(text).find(Boolean) || ''

export function timelineOccurredAt(record: Partial<RecordData>): string {
  const explicit = text(record.occurredAt)
  if (explicit) return explicit
  if (record.entity === 'timeLogs' || record.entity === 'events') return first(record.startAt, record.createdAt)
  if (record.entity === 'decisions') return first(record.date, record.decisionDate, record.createdAt)
  if (record.entity === 'results') return first(record.date, record.completedAt, record.createdAt)
  if (record.entity === 'tasks') {
    if (record.status === 'completed') return first(record.completedAt, record.createdAt)
    return first(record.dueAt, record.dueDate, record.createdAt)
  }
  return first(record.createdAt, record.updatedAt)
}

export function timelineTimeMeaning(record: Partial<RecordData>): TimelineTimeMeaning {
  if (record.timeMeaning === 'planned' || record.timeMeaning === 'actual' || record.timeMeaning === 'recorded') return record.timeMeaning
  if (record.entity === 'tasks') return record.status === 'completed' ? 'actual' : record.dueAt || record.dueDate ? 'planned' : 'recorded'
  if (record.entity === 'timeLogs' || record.entity === 'results' || record.entity === 'timelineEvents') return 'actual'
  if (record.entity === 'events') return 'planned'
  return 'recorded'
}

export function timelineImportance(record: Partial<RecordData>): TimelineImportance {
  if (record.timelineImportance === 'key' || record.timelineImportance === 'normal') return record.timelineImportance
  if (['decisions', 'results', 'reviews', 'insights'].includes(String(record.entity))) return 'key'
  if (record.entity === 'projects' && ['blocked', 'completed'].includes(String(record.status))) return 'key'
  if (record.entity === 'projects' && ['at_risk', 'blocked'].includes(String(record.health))) return 'key'
  if (record.entity === 'goals' && ['completed', 'paused'].includes(String(record.status))) return 'key'
  if (record.entity === 'tasks' && record.status === 'completed' && (record.priority === 'high' || record.importance === 'important')) return 'key'
  return 'normal'
}

export function timelineEvidenceLevel(record: Partial<RecordData>): TimelineEvidenceLevel {
  if (['REALITY', 'USER_CONFIRMED', 'AI_CONFIRMED', 'AI_SUGGESTION'].includes(String(record.evidenceLevel))) return record.evidenceLevel as TimelineEvidenceLevel
  if (record.entity === 'timeLogs' || record.entity === 'results' || record.entity === 'timelineEvents') return 'REALITY'
  return record.agentActionId ? 'AI_CONFIRMED' : 'USER_CONFIRMED'
}

export function timelineProjection(records: RecordData[]): TimelineProjectionItem[] {
  return records.filter((record) => timelineEntityTypes.includes(record.entity)).map((record) => ({
    id: record.id,
    record,
    occurredAt: timelineOccurredAt(record),
    timeMeaning: timelineTimeMeaning(record),
    importance: timelineImportance(record),
    evidenceLevel: timelineEvidenceLevel(record),
    goalId: text(record.goalId) || undefined,
    projectId: text(record.projectId) || undefined,
    taskId: text(record.taskId) || undefined,
  })).sort((a, b) => timelineTimestamp(b.occurredAt) - timelineTimestamp(a.occurredAt))
}

export function timelineTimestamp(value: string): number {
  if (!value) return 0
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    const [year, month, day] = value.split('-').map(Number)
    return new Date(year, month - 1, day).getTime()
  }
  if (/^\d+$/.test(value)) {
    const raw = Number(value)
    return raw > 1e12 ? raw : raw * 1000
  }
  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? 0 : parsed
}

export const timelineRecords = (records: RecordData[]) => timelineProjection(records).map((item) => item.record)

export type TimelineMode = 'key' | 'all'
export type TimelineRange = 'today' | 'week' | 'month' | '90d' | 'all'
export type TimelineFilter = {
  mode: TimelineMode
  range: TimelineRange
  entity: Entity | 'all'
  projectId: string | 'all' | 'unlinked'
  goalId: string | 'all' | 'unlinked'
}

export function timelineProjectId(item: TimelineProjectionItem): string | undefined {
  return item.projectId || (item.record.entity === 'projects' ? item.record.id : undefined)
}

export function timelineGoalId(item: TimelineProjectionItem): string | undefined {
  return item.goalId || (item.record.entity === 'goals' ? item.record.id : undefined)
}

export function timelineRangeBounds(range: TimelineRange, anchor = new Date()): { start: number; end: number } | null {
  if (range === 'all') return null
  const start = new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate())
  const end = new Date(start)
  if (range === 'week') {
    start.setDate(start.getDate() - ((start.getDay() + 6) % 7))
    end.setTime(start.getTime()); end.setDate(end.getDate() + 6)
  } else if (range === 'month') {
    start.setDate(1)
    end.setFullYear(start.getFullYear(), start.getMonth() + 1, 0)
  } else if (range === '90d') {
    start.setDate(start.getDate() - 89)
  }
  end.setHours(23, 59, 59, 999)
  return { start: start.getTime(), end: end.getTime() }
}

export function filterTimelineItems(items: TimelineProjectionItem[], filter: TimelineFilter, anchor = new Date()): TimelineProjectionItem[] {
  const bounds = timelineRangeBounds(filter.range, anchor)
  return items.filter((item) => {
    if (filter.mode === 'key' && item.importance !== 'key') return false
    if (filter.entity !== 'all' && item.record.entity !== filter.entity) return false
    const projectId = timelineProjectId(item)
    if (filter.projectId === 'unlinked' ? projectId : filter.projectId !== 'all' && projectId !== filter.projectId) return false
    const goalId = timelineGoalId(item)
    if (filter.goalId === 'unlinked' ? goalId : filter.goalId !== 'all' && goalId !== filter.goalId) return false
    if (bounds) { const timestamp = timelineTimestamp(item.occurredAt); if (timestamp < bounds.start || timestamp > bounds.end) return false }
    return true
  })
}

export function timelineDateKey(value: string): string {
  const timestamp = timelineTimestamp(value)
  if (!timestamp) return 'unknown'
  const date = new Date(timestamp)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

export function timelineDateLabel(key: string, anchor = new Date()): string {
  if (key === 'unknown') return '时间未知'
  const [year, month, day] = key.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  const today = new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate())
  const delta = Math.round((today.getTime() - date.getTime()) / 86_400_000)
  if (delta === 0) return '今天'
  if (delta === 1) return '昨天'
  return date.toLocaleDateString('zh-CN', { year: date.getFullYear() === today.getFullYear() ? undefined : 'numeric', month: 'long', day: 'numeric', weekday: 'short' })
}

export function groupTimelineItems(items: TimelineProjectionItem[], anchor = new Date()) {
  const groups = new Map<string, TimelineProjectionItem[]>()
  items.forEach((item) => { const key = timelineDateKey(item.occurredAt); groups.set(key, [...(groups.get(key) || []), item]) })
  return [...groups.entries()].map(([key, groupedItems]) => ({ key, label: timelineDateLabel(key, anchor), items: groupedItems }))
}
