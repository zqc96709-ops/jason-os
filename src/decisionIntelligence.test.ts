import { describe, expect, it } from 'vitest'
import { classifyDecision, runDecisionEngine } from './decisionIntelligence'
import type { RecordData } from './model'

const makeModel = (slug: string, name: string): RecordData => ({ id: `m-${slug}`, entity: 'mentalModels', slug, name, status: 'active', createdAt: '', updatedAt: '' })
const makeLens = (slug: string, modelSlugs: string, frameworkSlugs: string): RecordData => ({ id: `l-${slug}`, entity: 'decisionLenses', slug, name: slug, recommendedModelSlugs: modelSlugs, recommendedFrameworkSlugs: frameworkSlugs, status: 'active', createdAt: '', updatedAt: '' })
const makeFramework = (slug: string, name: string): RecordData => ({ id: `f-${slug}`, entity: 'decisionFrameworks', slug, name, status: 'active', createdAt: '', updatedAt: '' })
const makePrinciple = (slug: string, name: string): RecordData => ({ id: `p-${slug}`, entity: 'ceoPrinciples', slug, name, status: 'active', createdAt: '', updatedAt: '' })

const records: RecordData[] = [
  makeModel('probabilistic-thinking', '概率思维'), makeModel('base-rate', '基准率思维'), makeModel('inversion', '逆向思维'),
  makeModel('opportunity-cost', '机会成本'), makeModel('margin-of-safety', '安全边际'), makeModel('first-principles', '第一性原理'),
  makeModel('second-order-thinking', '二阶思维'), makeModel('circle-of-competence', '能力圈'), makeModel('jtbd', 'JTBD'), makeModel('feedback-loop', '反馈回路'),
  makeLens('investment', 'probabilistic-thinking,base-rate,inversion,opportunity-cost,margin-of-safety', 'strategy-kernel,five-forces'),
  makeLens('strategic', 'first-principles,second-order-thinking,opportunity-cost,circle-of-competence,margin-of-safety', 'strategy-kernel,five-forces,value-chain'),
  makeFramework('strategy-kernel', '战略内核'), makeFramework('five-forces', '波特五力'), makeFramework('value-chain', '价值链'),
  makePrinciple('long-termism', '长期主义'), makePrinciple('customer-obsession', '客户至上'), makePrinciple('day-1', 'Day 1'),
  makePrinciple('effective-executive', '有效管理者'), makePrinciple('strengths-management', '优势管理'), makePrinciple('principle-based-management', '原则化管理'), makePrinciple('ceo-attention', 'CEO 注意力配置'),
]

describe('classifyDecision', () => {
  it('识别投资决策，进入完整流程', () => {
    const result = classifyDecision('我要不要投入100万元做一个新的跨境电商项目？')
    expect(result.decisionType).toBe('investment')
    expect(result.impact).toBe('high')
    expect(result.lensSlug).toBe('investment')
  })
  it('低价值日常问题不应调用大量模型', () => {
    const result = classifyDecision('今天午饭吃什么？')
    expect(result.decisionType).toBe('strategic')
  })
  it('重大不可逆决策应降低置信度', () => {
    const strategic = classifyDecision('是否彻底转型公司战略方向？')
    expect(strategic.impact).toBe('high')
    expect(strategic.confidence).toBeLessThan(70)
  })
})

describe('runDecisionEngine', () => {
  it('投资决策选中概率/基准率/逆向/机会成本/安全边际', () => {
    const result = runDecisionEngine('我要不要投入100万元做一个新的跨境电商项目？', records)
    const slugs = result.models.map((item) => item.model.slug)
    expect(slugs).toContain('probabilistic-thinking')
    expect(slugs).toContain('base-rate')
    expect(slugs).toContain('inversion')
    expect(slugs).toContain('opportunity-cost')
    expect(slugs).toContain('margin-of-safety')
    expect(result.models.length).toBeGreaterThanOrEqual(3)
    expect(result.models.length).toBeLessThanOrEqual(5)
    expect(result.frameworks.map((item) => item.slug)).toContain('strategy-kernel')
    expect(result.frameworks.map((item) => item.slug)).toContain('five-forces')
  })
  it('产出完整决策分析结构', () => {
    const result = runDecisionEngine('要不要进入日本宠物市场？', records)
    expect(result.classification.lensSlug).toBe('market-entry')
    expect(result.options).toContain('Do Nothing：不做，保留资源')
    expect(result.biases.length).toBeGreaterThan(0)
    expect(result.tensions.length).toBeGreaterThan(0)
    expect(result.recommendation.length).toBeGreaterThan(0)
    expect(result.confidence).toBeGreaterThan(0)
  })
})

describe('model slug compatibility', () => {
  it('兼容真实数据库中的中文 slug 模型', () => {
    const chineseRecords: RecordData[] = [
      { id: 'cn-1', entity: 'mentalModels', slug: '概率思维', name: '概率思维', status: 'active', createdAt: '', updatedAt: '' },
      { id: 'cn-2', entity: 'mentalModels', slug: '基准率思维', name: '基准率思维', status: 'active', createdAt: '', updatedAt: '' },
      { id: 'cn-3', entity: 'mentalModels', slug: '逆向思维', name: '逆向思维', status: 'active', createdAt: '', updatedAt: '' },
      { id: 'cn-4', entity: 'mentalModels', slug: '机会成本', name: '机会成本', status: 'active', createdAt: '', updatedAt: '' },
      { id: 'cn-5', entity: 'mentalModels', slug: '安全边际', name: '安全边际', status: 'active', createdAt: '', updatedAt: '' },
      { id: 'lens-investment', entity: 'decisionLenses', slug: 'investment', name: '投资决策', recommendedModelSlugs: 'probabilistic-thinking,base-rate,inversion,opportunity-cost,margin-of-safety', recommendedFrameworkSlugs: '', status: 'active', createdAt: '', updatedAt: '' },
    ]
    const result = runDecisionEngine('我要不要投入100万元做一个新项目？', chineseRecords)
    expect(result.models.map(({ model }) => model.name)).toEqual(['概率思维', '基准率思维', '逆向思维', '机会成本', '安全边际'])
  })
})
