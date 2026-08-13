import type { RecordData } from './model'
import { recommendMentalModels, type ModelRecommendation } from './mentalModels'

export type ImpactLevel = 'low' | 'medium' | 'high'
export type UrgencyLevel = 'low' | 'medium' | 'high'
export type Reversibility = 'reversible' | 'partially' | 'irreversible'

export type DecisionClassification = {
  decisionType: string
  decisionTypeLabel: string
  lensSlug: string
  impact: ImpactLevel
  urgency: UrgencyLevel
  reversibility: Reversibility
  confidence: number
  reasons: string[]
}

export type BiasCheck = { bias: string; severity: 'low' | 'medium' | 'high'; recommendation: string }
export type ModelTension = { left: string; right: string; note: string }

export type DecisionAnalysis = {
  classification: DecisionClassification
  lens: RecordData | undefined
  models: ModelRecommendation[]
  frameworks: RecordData[]
  principles: RecordData[]
  assumptions: string[]
  supportingCase: string[]
  counterCase: string[]
  biases: BiasCheck[]
  tensions: ModelTension[]
  opportunityCost: string
  informationGaps: string[]
  minimumValidation: string
  options: string[]
  recommendation: string
  confidence: number
  ceoDecision?: string
}

const DECISION_TYPES: { type: string; label: string; lens: string; keywords: string[]; impact: ImpactLevel; urgency: UrgencyLevel; reversibility: Reversibility }[] = [
  { type: 'investment', label: '投资决策', lens: 'investment', keywords: ['投入', '投资', '万元', '预算', '大额', '资金', '融资', '开发新', '建厂', '采购'], impact: 'high', urgency: 'medium', reversibility: 'partially' },
  { type: 'market-entry', label: '市场进入', lens: 'market-entry', keywords: ['进入', '出海', '新市场', '日本', '美国', '海外', '跨境', '拓展市场'], impact: 'high', urgency: 'medium', reversibility: 'partially' },
  { type: 'strategic', label: '战略决策', lens: 'strategic', keywords: ['战略', '方向', '长期', '转型', '定位', '护城河', '愿景'], impact: 'high', urgency: 'medium', reversibility: 'irreversible' },
  { type: 'crisis', label: '危机决策', lens: 'crisis', keywords: ['危机', '紧急', '止损', '倒闭', '风险爆发', '投诉爆发', '舆情', '断供', '封号'], impact: 'high', urgency: 'high', reversibility: 'irreversible' },
  { type: 'product', label: '产品决策', lens: 'product', keywords: ['产品', '功能', '开发', '需求', '选品', '上新', '迭代'], impact: 'medium', urgency: 'medium', reversibility: 'reversible' },
  { type: 'talent', label: '人才决策', lens: 'talent', keywords: ['人才', '员工', '离职', '高管', '招聘', '裁员', '合伙人'], impact: 'high', urgency: 'medium', reversibility: 'irreversible' },
  { type: 'organization', label: '组织决策', lens: 'organization', keywords: ['组织', '团队', '部门', '架构', '分工', '管理'], impact: 'medium', urgency: 'medium', reversibility: 'partially' },
  { type: 'finance', label: '财务决策', lens: 'finance', keywords: ['财务', '现金流', '成本', '借款', '贷款', '分红', '税务'], impact: 'high', urgency: 'medium', reversibility: 'partially' },
]

const BIASES: BiasCheck[] = [
  { bias: '确认偏误', severity: 'medium', recommendation: '主动寻找反对证据，而不是只收集支持证据。' },
  { bias: '过度自信', severity: 'medium', recommendation: '用基准率校准信心，并把预测区间放宽。' },
  { bias: '沉没成本', severity: 'medium', recommendation: '只考虑未来增量，不考虑已经投入的成本。' },
  { bias: '损失厌恶', severity: 'high', recommendation: '把损失和收益放在同一框架下比较，而不是被恐惧支配。' },
  { bias: '从众', severity: 'low', recommendation: '区分“别人都在做”和“这件事本身正确”。' },
]

const TENSIONS: ModelTension[] = [
  { left: '增长', right: '安全边际', note: '追求增长不能以丧失生存缓冲为代价。' },
  { left: '速度', right: '信息完整性', note: '快速行动会牺牲信息，深度分析会错过窗口。' },
  { left: '长期主义', right: '机会成本', note: '押注长期会放弃眼前更确定的收益。' },
  { left: '规模', right: '灵活性', note: '规模化需要标准化，但会降低对变化的响应速度。' },
]

const value = (record: RecordData | undefined, key: string, fallback = '') => String(record?.[key] ?? fallback)

export function classifyDecision(question: string): DecisionClassification {
  const q = question.trim()
  const matched = DECISION_TYPES.find((item) => item.keywords.some((keyword) => q.includes(keyword)))
  const type = matched || { type: 'strategic', label: '战略决策', lens: 'strategic', keywords: [], impact: 'high' as ImpactLevel, urgency: 'medium' as UrgencyLevel, reversibility: 'irreversible' as Reversibility }
  const reasons = [
    `识别为「${type.label}」：问题中包含相关决策信号。`,
    `影响程度：${impactLabel(type.impact)}；紧急程度：${urgencyLabel(type.urgency)}。`,
    `可逆性：${reversibilityLabel(type.reversibility)}。`,
  ]
  const confidence = type.impact === 'high' && type.reversibility === 'irreversible' ? 55 : type.impact === 'low' ? 80 : 68
  return { decisionType: type.type, decisionTypeLabel: type.label, lensSlug: type.lens, impact: type.impact, urgency: type.urgency, reversibility: type.reversibility, confidence, reasons }
}

export const impactLabel = (value: ImpactLevel) => ({ low: '低', medium: '中', high: '高' }[value])
export const urgencyLabel = (value: UrgencyLevel) => ({ low: '低', medium: '中', high: '高' }[value])
export const reversibilityLabel = (value: Reversibility) => ({ reversible: '可逆', partially: '部分可逆', irreversible: '不可逆' }[value])

const active = (records: RecordData[], entity: string) => records.filter((record) => record.entity === entity && record.status !== 'archived')
const MODEL_ALIASES: Record<string, string[]> = {
  'first-principles': ['第一性原理'],
  inversion: ['逆向思维'],
  'second-order-thinking': ['二阶思维'],
  'probabilistic-thinking': ['概率思维'],
  'base-rate': ['基准率思维'],
  'opportunity-cost': ['机会成本'],
  'circle-of-competence': ['能力圈'],
  'margin-of-safety': ['安全边际'],
  incentives: ['激励机制'],
  'feedback-loop': ['反馈回路'],
  'reversible-irreversible': ['可逆/不可逆决策', '可逆 / 不可逆决策'],
  jtbd: ['Jobs to Be Done', 'JTBD'],
  'information-value': ['信息价值'],
  'strategic-inflection': ['战略拐点'],
  'five-forces': ['五力模型', '波特五力'],
  'strategy-kernel': ['战略内核'],
  'value-chain': ['价值链'],
}

const bySlug = (records: RecordData[], slug: string) => {
  const aliases = [slug, ...(MODEL_ALIASES[slug] || [])].map((item) => item.toLowerCase().replaceAll(' ', ''))
  return records.find((record) => [record.slug, record.name].some((value) => aliases.includes(String(value || '').toLowerCase().replaceAll(' ', ''))))
}

export function runDecisionEngine(question: string, records: RecordData[]): DecisionAnalysis {
  const classification = classifyDecision(question)
  const lenses = active(records, 'decisionLenses')
  const frameworks = active(records, 'decisionFrameworks')
  const principles = active(records, 'ceoPrinciples')
  const models = active(records, 'mentalModels')

  const lens = lenses.find((item) => String(item.slug) === classification.lensSlug) || lenses[0]

  const modelSlugs = value(lens, 'recommendedModelSlugs').split(',').map((slug) => slug.trim()).filter(Boolean)
  let selectedModels: ModelRecommendation[] = modelSlugs
    .map((slug) => {
      const model = bySlug(models, slug)
      return model ? { model, score: 50 + (modelSlugs.length - modelSlugs.indexOf(slug)), reasons: ['该视角推荐使用的核心模型。'] } : null
    })
    .filter((item): item is ModelRecommendation => item !== null)
  if (selectedModels.length < 3) {
    const fallback = recommendMentalModels(models, question).filter((item) => !selectedModels.some((existing) => existing.model.id === item.model.id))
    selectedModels = [...selectedModels, ...fallback].slice(0, 5)
  }
  selectedModels = selectedModels.slice(0, 5)

  const frameworkSlugs = value(lens, 'recommendedFrameworkSlugs').split(',').map((slug) => slug.trim()).filter(Boolean)
  const selectedFrameworks = frameworkSlugs.map((slug) => bySlug(frameworks, slug)).filter((item): item is RecordData => Boolean(item)).slice(0, 3)

  const assumptions = [
    '问题定义与事实边界是否准确（未被行业惯例绑架）。',
    '对收益与成本的估计是否过度乐观（需要安全边际）。',
    '这个决策是否真的不可逆，还是存在可先验证的步骤。',
  ]
  const supportingCase = ['如果成功，最可能的路径是：抓住真实需求 + 可验证的差异化 + 足够的缓冲。']
  const counterCase = ['如果失败，最可能因为：高估需求、低估执行成本、忽视竞争者的二阶反应。']
  const opportunityCost = '如果这笔资源（资金、CEO 时间、团队精力）投入别处，是否会带来更高且更确定的回报？'
  const informationGaps = ['尚未确认的关键证据：真实需求规模、单位经济模型、竞争者的反应能力、执行团队匹配度。']
  const minimumValidation = '先花最少钱、最短时间验证最大假设：明确验证目标、预算、周期、样本、指标、成功/失败标准和停止条件。'
  const options = ['Option A：全力投入', 'Option B：小步验证后再投入', 'Option C：暂缓并继续观察', 'Do Nothing：不做，保留资源']

  const recommendation = classification.impact === 'high' && classification.reversibility !== 'reversible'
    ? '建议先做最小验证（Option B），在关键假设被证实前不要一次性投入全部资源；把不可逆动作拆成可逆步骤。'
    : '建议用低成本快速验证核心假设，并设置明确的停止条件。'

  return {
    classification,
    lens,
    models: selectedModels,
    frameworks: selectedFrameworks,
    principles: principles.slice(0, 7),
    assumptions,
    supportingCase,
    counterCase,
    biases: BIASES,
    tensions: TENSIONS,
    opportunityCost,
    informationGaps,
    minimumValidation,
    options,
    recommendation,
    confidence: classification.confidence,
  }
}
