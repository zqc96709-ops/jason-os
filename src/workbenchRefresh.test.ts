import { describe, expect, it } from 'vitest'
import type { RecordData } from './model'
import { loadWorkbenchSnapshot } from './workbenchRefresh'

const task = { id: 'task-1', entity: 'tasks', title: '保留活动记录', createdAt: '1', updatedAt: '2' } as RecordData

describe('loadWorkbenchSnapshot', () => {
  it('keeps active records when optional archived evidence and external queries fail', async () => {
    await expect(loadWorkbenchSnapshot(
      async () => [task],
      async () => { throw new Error('archive unavailable') },
      async () => { throw new Error('external unavailable') },
    )).resolves.toEqual({ records: [task], archivedFollowUpEvidence: [], externalItems: [] })
  })

  it('still rejects when the primary active-record query fails', async () => {
    await expect(loadWorkbenchSnapshot(
      async () => { throw new Error('active unavailable') },
      async () => [],
      async () => [],
    )).rejects.toThrow('active unavailable')
  })
})
