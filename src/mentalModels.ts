import type { RecordData } from './model'

export const mentalModelCategories = [
  { id: 'all', label: '全部' },
  { id: 'problem-cognition', label: '问题认知' },
  { id: 'risk-error', label: '风险与错误' },
  { id: 'decision-judgment', label: '决策判断' },
  { id: 'strategy', label: '战略' },
  { id: 'customer-product', label: '客户与产品' },
  { id: 'growth', label: '增长' },
  { id: 'organization', label: '组织' },
  { id: 'learning-evolution', label: '学习与进化' },
] as const

export type ModelRecommendation = { model: RecordData; score: number; reasons: string[] }

const rules: { keywords: string[]; slugs: string[]; reason: string }[] = [
  { keywords: ['50万', '50万元', '投入', '投资', '大额', '预算', '新产品', '开发产品'], slugs: ['first-principles', 'jtbd', 'probabilistic-thinking', 'opportunity-cost', 'margin-of-safety', 'inversion'], reason: '当前涉及较大投入、产品假设与不确定性。' },
  { keywords: ['日本市场', '进入市场', '出海', '新市场', '市场进入'], slugs: ['five-forces', 'value-chain', 'circle-of-competence', 'opportunity-cost', 'probabilistic-thinking', 'strategic-inflection'], reason: '当前需要同时判断行业结构、能力边界与市场变化。' },
  { keywords: ['员工离职', '离职率', '团队', '核心员工', '人才', '组织'], slugs: ['incentives', 'effective-manager', 'strengths-management', 'second-order-thinking', 'feedback-loop'], reason: '当前问题涉及激励、管理和组织反馈回路。' },
  { keywords: ['竞争对手', '降价', '竞品', '价格战'], slugs: ['five-forces', 'second-order-thinking', 'incentives', 'strategic-inflection', 'inversion'], reason: '竞争动作可能改变行业结构，并带来二阶影响。' },
  { keywords: ['AI', '人工智能', '行业改变', '技术变化', '技术冲击'], slugs: ['first-principles', 'strategic-inflection', 'innovators-dilemma', 'long-termism', 'opportunity-cost'], reason: '当前需要区分技术变化、旧假设失效和长期选择。' },
  { keywords: ['复盘', '结果', '经验', '学习'], slugs: ['retrospective', 'hypothesis-update', 'experience-extraction', 'decision-feedback'], reason: '当前重点是把结果转化为可复用学习。' },
  { keywords: ['风险', '失败', '最坏', '不确定', '安全'], slugs: ['inversion', 'premortem', 'margin-of-safety', 'probabilistic-thinking', 'second-order-thinking'], reason: '当前需要先识别失败路径、缓冲和连锁影响。' },
]

const textOf = (model: RecordData) => [model.name, model.definition, model.coreIdea, model.corePrinciple, model.applicationScenarios, model.useCases, model.triggerConditions, model.trigger, model.tags, model.sourcePerson, model.source].filter(Boolean).join(' ').toLowerCase()
const slugOf = (model: RecordData) => String(model.slug || '').toLowerCase()
const tokens = (value: string) => value.toLowerCase().split(/\s+|[，。、“”‘’；：,.;:!?！？/]+/).filter(Boolean)

export function recommendMentalModels(records: RecordData[], question: string): ModelRecommendation[] {
  const models = records.filter((record) => record.entity === 'mentalModels' && record.status !== 'archived')
  const query = question.trim()
  const matchedRules = rules.filter((rule) => rule.keywords.some((keyword) => query.includes(keyword)))
  const scores = new Map<string, ModelRecommendation>()
  const add = (model: RecordData, score: number, reason: string) => {
    const current = scores.get(model.id) || { model, score: 0, reasons: [] }
    current.score += score
    if (!current.reasons.includes(reason)) current.reasons.push(reason)
    scores.set(model.id, current)
  }
  for (const rule of matchedRules) {
    for (const slug of rule.slugs) {
      const model = models.find((item) => slugOf(item) === slug || textOf(item).includes(slug.replaceAll('-', ' ')))
      if (model) add(model, 10 + Math.max(1, rule.slugs.length - rule.slugs.indexOf(slug)), rule.reason)
    }
  }
  const queryTokens = tokens(query)
  for (const model of models) {
    const modelTokens = tokens(textOf(model))
    const overlap = queryTokens.filter((token) => modelTokens.includes(token)).length
    if (overlap) add(model, overlap * 2, '模型内容与当前问题存在关键词匹配。')
  }
  if (!scores.size) {
    for (const model of models.filter((item) => ['decision-judgment', 'risk-error', 'strategy'].includes(String(item.categoryId))).slice(0, 5)) add(model, 1, '当前没有足够上下文，先提供 CEO 决策基础检查。')
  }
  return [...scores.values()].sort((a, b) => b.score - a.score || String(a.model.name).localeCompare(String(b.model.name))).slice(0, 5)
}

export const modelCategoryLabel = (model: RecordData) => String(model.category || mentalModelCategories.find((item) => item.id === model.categoryId)?.label || '待归类')
export const modelSourcePerson = (model: RecordData) => String(model.sourcePerson || model.source || '未记录')
export const modelTrigger = (model: RecordData) => String(model.triggerConditions || model.trigger || model.useCases || '未记录')
export const modelDefinition = (model: RecordData) => String(model.definition || model.coreIdea || model.corePrinciple || '未记录')
