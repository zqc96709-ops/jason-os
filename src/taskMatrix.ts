import { isOverdue, isToday, type RecordData } from './model'

export type TaskImportance = 'important' | 'not_important'
export type TaskUrgency = 'urgent' | 'not_urgent'
export type TaskQuadrant = 'do' | 'plan' | 'delegate' | 'eliminate'

export const taskMatrixQuadrants: { id: TaskQuadrant; title: string; action: string; importance: TaskImportance; urgency: TaskUrgency }[] = [
  { id: 'do', title: '重要且紧急', action: '立即做', importance: 'important', urgency: 'urgent' },
  { id: 'plan', title: '重要不紧急', action: '计划做', importance: 'important', urgency: 'not_urgent' },
  { id: 'delegate', title: '紧急不重要', action: '委派 / 快速处理', importance: 'not_important', urgency: 'urgent' },
  { id: 'eliminate', title: '不重要不紧急', action: '减少或删除', importance: 'not_important', urgency: 'not_urgent' },
]

export function taskMatrixValues(task: Partial<RecordData>): { importance: TaskImportance; urgency: TaskUrgency } {
  const importance = task.importance === 'important' || task.importance === 'not_important'
    ? task.importance
    : task.priority === 'high' ? 'important' : 'not_important'
  const urgency = task.urgency === 'urgent' || task.urgency === 'not_urgent'
    ? task.urgency
    : isOverdue(task) || isToday(task.dueDate) ? 'urgent' : 'not_urgent'
  return { importance, urgency }
}

export function taskQuadrant(task: Partial<RecordData>): TaskQuadrant {
  const { importance, urgency } = taskMatrixValues(task)
  if (importance === 'important') return urgency === 'urgent' ? 'do' : 'plan'
  return urgency === 'urgent' ? 'delegate' : 'eliminate'
}
