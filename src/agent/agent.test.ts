import { describe, expect, it } from 'vitest'
import { buildAgentContext } from './contextEngine'
import { schemaFor } from './schemaRegistry'
import { toolFor, toolRegistry } from './toolRegistry'
import type { RecordData } from '../model'

const record = (entity: RecordData['entity'], id: string, data: Record<string, unknown> = {}) => ({ id, entity, createdAt: '', updatedAt: '', ...data }) as RecordData

describe('Jason OS Agent registries and context', () => {
  it('describes the MentalModel schema and save tool', () => {
    expect(schemaFor('mentalModels')?.requiredFields).toContain('name')
    expect(toolFor('createMentalModel')).toMatchObject({ entity: 'mentalModels', riskLevel: 'LOW_WRITE', requiresConfirmation: true })
    expect(toolRegistry.length).toBeGreaterThanOrEqual(30)
  })

  it('derives Goal → Project → Task context from the current task', () => {
    const records = [record('goals', 'g1'), record('projects', 'p1', { goalId: 'g1' }), record('tasks', 't1', { projectId: 'p1', goalId: 'g1' })]
    expect(buildAgentContext({ currentRoute: 'tasks', records, selectedProjectId: null, detailId: 't1', conversation: [] })).toMatchObject({ currentEntityType: 'tasks', currentEntityId: 't1', currentTaskId: 't1', currentProjectId: 'p1', currentGoalId: 'g1' })
  })

  it('uses the current page entity when no record is selected', () => {
    expect(buildAgentContext({ currentRoute: 'mentalModels', records: [], selectedProjectId: null, detailId: null, conversation: [] }).currentEntityType).toBe('mentalModels')
  })
})
