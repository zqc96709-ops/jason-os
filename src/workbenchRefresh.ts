import type { ExternalItem } from './externalIntelligence'
import type { RecordData } from './model'

export type WorkbenchSnapshot = {
  records: RecordData[]
  archivedFollowUpEvidence: RecordData[]
  externalItems: ExternalItem[]
}

export const loadWorkbenchSnapshot = async (
  loadActive: () => Promise<RecordData[]>,
  loadArchivedEvidence: () => Promise<RecordData[]>,
  loadExternalItems: () => Promise<ExternalItem[]>,
): Promise<WorkbenchSnapshot> => {
  const [records, archivedFollowUpEvidence, externalItems] = await Promise.all([
    loadActive(),
    loadArchivedEvidence().catch(() => [] as RecordData[]),
    loadExternalItems().catch(() => [] as ExternalItem[]),
  ])

  return { records, archivedFollowUpEvidence, externalItems }
}
