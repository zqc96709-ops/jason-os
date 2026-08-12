import { describe, expect, it } from 'vitest'
import { recommendMentalModels } from './mentalModels'
import type { RecordData } from './model'

const models = [
  ['first-principles', '第一性原理', '问题认知'], ['jtbd', 'Jobs to Be Done', '客户与产品'], ['probabilistic-thinking', '概率思维', '决策判断'], ['opportunity-cost', '机会成本', '决策判断'], ['margin-of-safety', '安全边际', '风险与错误'], ['inversion', '逆向思维', '风险与错误'],
  ['five-forces', '五力模型', '战略'], ['value-chain', '价值链', '战略'], ['circle-of-competence', '能力圈', '问题认知'], ['strategic-inflection', '战略拐点', '战略'],
  ['incentives', '激励机制', '组织'], ['effective-manager', '有效管理者', '组织'], ['strengths-management', '优势管理', '组织'], ['second-order-thinking', '二阶思维', '风险与错误'], ['feedback-loop', '反馈回路', '客户与产品'],
  ['innovators-dilemma', '创新者窘境', '战略'], ['long-termism', '长期主义', '学习与进化'],
].map(([slug, name, category]) => ({ id: slug, entity: 'mentalModels', slug, name, category, categoryId: category, definition: `${name} 定义`, coreIdea: `${name} 核心`, triggerConditions: '重大决策,高不确定性', sourcePerson: '测试来源', tags: 'CEO,决策' })) as unknown as RecordData[]

const names = (question: string) => recommendMentalModels(models, question).map(({ model }) => model.name)

describe('CEO mental model recommendation V1', () => {
  it('recommends product investment models', () => expect(names('是否投入50万元开发一个新产品？')).toEqual(expect.arrayContaining(['第一性原理', 'Jobs to Be Done', '概率思维', '机会成本', '安全边际', '逆向思维'].slice(0, 5))))
  it('recommends market entry models', () => expect(names('是否进入日本市场？')).toEqual(expect.arrayContaining(['五力模型', '价值链', '能力圈', '机会成本', '概率思维'].slice(0, 5))))
  it('recommends organization models', () => expect(names('团队核心员工离职率越来越高怎么办？')).toEqual(expect.arrayContaining(['激励机制', '有效管理者', '优势管理', '二阶思维', '反馈回路'].slice(0, 5))))
  it('recommends competitor response models', () => expect(names('竞争对手突然开始降价怎么办？')).toEqual(expect.arrayContaining(['五力模型', '二阶思维', '激励机制', '战略拐点', '逆向思维'].slice(0, 5))))
  it('recommends AI industry change models', () => expect(names('AI正在改变我们的行业，我应该怎么办？')).toEqual(expect.arrayContaining(['第一性原理', '战略拐点', '创新者窘境', '长期主义', '机会成本'].slice(0, 5))))
})
