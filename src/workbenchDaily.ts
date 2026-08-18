import type { RecordData } from './model'

export type DailyLane = 'business' | 'product' | 'personal'
export type DailyAction = {
  recordId: string
  title: string
  lane: DailyLane
  reason: string
  nextStep: string
  source: string
}
export type DailyBrief = {
  actions: DailyAction[]
  observation: string
  practice: string
  correction: string
}
export type FollowUpStage = 'action' | 'result' | 'review'
export type FollowUpItem = {
  key: string
  stage: FollowUpStage
  sourceId: string
  title: string
  reason: string
  cta: string
  createEntity?: 'results' | 'reviews'
  initial?: Partial<RecordData>
}

const text = (value: unknown) => String(value || '').trim()
const localDate = (date: Date) => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}
const present = (value: unknown) => {
  if (value === undefined || value === null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (Array.isArray(value)) return value.length > 0
  return true
}
const usable = (record: RecordData) => !record.deletedAt && !record.archivedAt
const taskCompleted = (record: RecordData) => text(record.status) === 'completed' || present(record.completedAt)
const resultHasActualEvidence = (record: RecordData) =>
  ['ACHIEVED', 'PARTIALLY_ACHIEVED', 'MISSED', 'CANCELLED'].includes(text(record.status)) ||
  [record.actual, record.actualResult, record.actualValue, record.actualAmountMinor, record.evidence].some(present)
const resultStatusLabel = (status: string) => ({ ACHIEVED: '已达成', PARTIALLY_ACHIEVED: '部分达成', MISSED: '未达成', CANCELLED: '已取消' } as Record<string, string>)[status] || status
const resultActualSummary = (record: RecordData) => {
  const narrative = text(record.actual) || text(record.actualResult)
  if (narrative) return narrative
  if (present(record.actualValue)) return `实际数值：${String(record.actualValue)}${text(record.unit) ? ` ${text(record.unit)}` : ''}`
  if (present(record.actualAmountMinor)) return `实际金额（最小货币单位）：${String(record.actualAmountMinor)}${text(record.currency) ? ` ${text(record.currency)}` : ''}`
  if (present(record.evidence)) return typeof record.evidence === 'string' && record.evidence.trim() ? `结果证据：${record.evidence.trim()}` : '已记录结果证据'
  return text(record.status) ? `结果状态：${resultStatusLabel(text(record.status))}` : undefined
}

export const reviewInitialForResult = (result: RecordData): Partial<RecordData> => ({
  title: `复盘：${titleForBrief(result)}`,
  resultId: result.id,
  taskId: result.taskId,
  projectId: result.projectId,
  goalId: result.goalId,
  whatHappened: resultActualSummary(result),
})

export const workbenchDateLabel = (date = new Date()) => date.toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'long' })

export function prepareInboxCapture(content: string): Pick<RecordData, 'content' | 'status'> {
  const normalized = content.trim()
  if (!normalized) throw new Error('请先输入正在发生的事')
  return { content: normalized, status: 'unprocessed' }
}
const lower = (record: RecordData) => [record.title, record.description, record.why, record.nextAction, record.tags, record.content].map(text).join(' ').toLowerCase()
const titleForBrief = (record: RecordData) => text(record.title) || text(record.content) || '未命名记录'
const laneFor = (record: RecordData): DailyLane => {
  const value = lower(record)
  if (/健身|睡眠|休息|游戏|个人|情绪|身体/.test(value)) return 'personal'
  if (/产品|创作台|话术|jason|功能|界面|开发|代码|测试|批量|工作台/.test(value)) return 'product'
  return 'business'
}
const priorityScore = (record: RecordData, now: Date) => {
  const due = text(record.dueDate)
  const overdue = due && due < localDate(now)
  const priority = text(record.priority)
  return (overdue ? 100 : 0) + (priority === 'high' ? 50 : priority === 'medium' ? 20 : 0) + (due ? 5 : 0)
}
const reasonFor = (record: RecordData, now: Date) => {
  const reasons: string[] = []
  if (text(record.dueDate) && text(record.dueDate) < localDate(now)) reasons.push('已逾期')
  if (text(record.priority) === 'high') reasons.push('高优先级')
  if (text(record.nextAction)) reasons.push('已有明确下一步')
  if (!reasons.length) reasons.push('来自你的真实记录')
  return reasons.join(' · ')
}
const nextStepFor = (record: RecordData) => text(record.nextAction) || (record.entity === 'projects' ? '先确认项目当前最小可交付结果' : '用 15 分钟完成第一步，并记录实际结果')

export function buildDailyBrief(records: RecordData[], now = new Date()): DailyBrief {
  const candidates = records
    .filter((record) => ['tasks', 'projects', 'inbox'].includes(record.entity))
    .filter((record) => record.entity !== 'inbox' || Boolean(text(record.content)))
    .filter((record) => !(record.entity === 'tasks' && taskCompleted(record)) && !['completed', 'cancelled', 'archived'].includes(text(record.status)) && !record.archivedAt && !record.deletedAt)
    .map((record) => ({ record, lane: laneFor(record), score: priorityScore(record, now) }))
    .sort((a, b) => b.score - a.score || text(a.record.createdAt).localeCompare(text(b.record.createdAt)))

  const actions: DailyAction[] = []
  for (const lane of ['business', 'product', 'personal'] as DailyLane[]) {
    const item = candidates.find((candidate) => candidate.lane === lane && !actions.some((action) => action.recordId === candidate.record.id))
    if (!item) continue
    actions.push({ recordId: item.record.id, title: titleForBrief(item.record), lane, reason: reasonFor(item.record, now), nextStep: nextStepFor(item.record), source: `${item.record.entity} · ${item.record.id}` })
  }

  return {
    actions,
    observation: actions.length ? `今天先看这 ${actions.length} 件事；每一项都来自已有记录，不凭空增加任务。` : '当前真实记录不足，Jason 不替你编造今天的优先级。先用“告诉 Jason”说一件正在发生的事。',
    practice: actions.some((action) => action.lane === 'product') ? '产品工作先验证最小闭环，再扩展功能。' : '今天完成一件事后，补记实际结果。',
    correction: actions.some((action) => action.reason.includes('已逾期')) ? '有逾期事项，先处理或明确暂停，不要继续堆新任务。' : '暂未发现需要立即纠偏的明确证据。',
  }
}

export function followUpRecordsWithEvidence(activeRecords: RecordData[], archivedRecords: RecordData[]): RecordData[] {
  const seen = new Set(activeRecords.map((record) => record.id))
  const archivedEvidence = archivedRecords.filter((record) =>
    ['results', 'reviews'].includes(record.entity) && !record.deletedAt && !seen.has(record.id))
  return [...activeRecords, ...archivedEvidence]
}

export function buildFollowUpQueue(records: RecordData[], now = new Date()): FollowUpItem[] {
  const today = localDate(now)
  const tasks = records.filter((record) => record.entity === 'tasks' && usable(record))
  const results = records.filter((record) => record.entity === 'results' && usable(record))
  const resultEvidence = records.filter((record) => record.entity === 'results' && !record.deletedAt)
  const reviews = records.filter((record) => record.entity === 'reviews' && !record.deletedAt)

  const actionItems = tasks
    .filter((task) => !taskCompleted(task) && !['cancelled', 'archived'].includes(text(task.status)))
    .filter((task) => Boolean(text(task.dueDate)) && text(task.dueDate) <= today)
    .sort((a, b) => text(a.dueDate).localeCompare(text(b.dueDate)) || text(a.createdAt).localeCompare(text(b.createdAt)))
    .map<FollowUpItem>((task) => ({
      key: `action:${task.id}`,
      stage: 'action',
      sourceId: task.id,
      title: titleForBrief(task),
      reason: text(task.dueDate) < today ? '任务已逾期' : '任务今天到期',
      cta: '打开任务',
    }))

  const resultItems = tasks
    .filter(taskCompleted)
    .filter((task) => !resultEvidence.some((result) => result.taskId === task.id && resultHasActualEvidence(result)))
    .sort((a, b) => text(a.completedAt).localeCompare(text(b.completedAt)) || text(a.createdAt).localeCompare(text(b.createdAt)))
    .map<FollowUpItem>((task) => ({
      key: `result:${task.id}`,
      stage: 'result',
      sourceId: task.id,
      title: titleForBrief(task),
      reason: '任务已完成，尚未记录实际结果',
      cta: '回填结果',
      createEntity: 'results',
      initial: {
        title: `结果：${titleForBrief(task)}`,
        taskId: task.id,
        projectId: task.projectId,
        goalId: task.goalId,
        date: today,
      },
    }))

  const reviewItems = results
    .filter(resultHasActualEvidence)
    .filter((result) => !reviews.some((review) => review.resultId === result.id))
    .sort((a, b) => text(a.date || a.updatedAt || a.createdAt).localeCompare(text(b.date || b.updatedAt || b.createdAt)))
    .map<FollowUpItem>((result) => ({
      key: `review:${result.id}`,
      stage: 'review',
      sourceId: result.id,
      title: titleForBrief(result),
      reason: '实际结果已记录，尚未复盘',
      cta: '开始复盘',
      createEntity: 'reviews',
      initial: reviewInitialForResult(result),
    }))

  return [...actionItems, ...resultItems, ...reviewItems]
}

export function selectFollowUpsForDisplay(queue: FollowUpItem[], limit = 6): FollowUpItem[] {
  if (limit <= 0) return []
  const stages: FollowUpStage[] = ['action', 'result', 'review']
  const buckets = new Map(stages.map((stage) => [stage, queue.filter((item) => item.stage === stage)]))
  const selected: FollowUpItem[] = []
  for (let index = 0; selected.length < limit; index += 1) {
    let added = false
    for (const stage of stages) {
      const item = buckets.get(stage)?.[index]
      if (!item) continue
      selected.push(item)
      added = true
      if (selected.length === limit) break
    }
    if (!added) break
  }
  return selected
}
