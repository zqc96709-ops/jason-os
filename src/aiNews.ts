export type RadarCategory = 'all' | 'official' | 'curated' | 'builders' | 'creator' | 'aggregate'

export type RadarStory = {
  id: string
  title: string
  titleEn: string
  url: string
  source: string
  sourceNames: string[]
  sourceCount: number
  publishedAt: string
  reason: string
  score?: number
  category: Exclude<RadarCategory, 'all'>
}

export type RadarData = {
  generatedAt: string
  totalItems: number
  sourceCount: number
  curated: RadarStory[]
  all: RadarStory[]
}

export const radarCategories: { id: RadarCategory; label: string }[] = [
  { id: 'all', label: '全部' },
  { id: 'official', label: '官方' },
  { id: 'curated', label: '精选媒体' },
  { id: 'builders', label: '开发者 / X' },
  { id: 'creator', label: '创作者' },
  { id: 'aggregate', label: '聚合' },
]

const objectValue = (value: unknown): Record<string, unknown> => value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : {}
const text = (...values: unknown[]) => String(values.find((value) => typeof value === 'string' && value.trim()) || '')
const numberValue = (...values: unknown[]) => Number(values.find((value) => Number.isFinite(Number(value))) || 0)
const strings = (value: unknown) => Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string' && Boolean(item)) : []

function categoryFor(item: Record<string, unknown>): Exclude<RadarCategory, 'all'> {
  const category = text(item.category)
  const siteId = text(item.site_id, objectValue(item.primary_item).site_id)
  if (category === 'official' || siteId === 'official_ai' || item.source_tier === 'official') return 'official'
  if (category === 'creator' || siteId.startsWith('tikhub_')) return 'creator'
  if (['followbuilders', 'xapi', 'socialdata_x', 'waytoagi'].includes(siteId)) return 'builders'
  if (['curated_media', 'aihot', 'aibreakfast', 'aibase', 'aihubtoday', 'opmlrss', 'bestblogs'].includes(siteId)) return 'curated'
  return 'aggregate'
}

export function normalizeRadarStory(value: unknown, index = 0): RadarStory | null {
  const item = objectValue(value)
  const primary = objectValue(item.primary_item)
  const url = text(item.primary_url, item.url, primary.url)
  const title = text(item.title_zh, item.title, primary.title_zh, primary.title, item.title_en, primary.title_en)
  if (!url || !title) return null
  const sourceNames = strings(item.source_names)
  const sources = Array.isArray(item.sources) ? item.sources.map(objectValue) : []
  if (!sourceNames.length) sources.forEach((source) => { const name = text(source.source, source.source_name); if (name && !sourceNames.includes(name)) sourceNames.push(name) })
  const source = text(item.source, item.source_name, primary.source, primary.source_name, sourceNames[0], '未知来源')
  if (!sourceNames.length) sourceNames.push(source)
  const rawScore = numberValue(item.importance_score, item.importance, item.ai_score, primary.ai_score)
  return {
    id: text(item.story_id, item.id, primary.id, `${url}-${index}`),
    title,
    titleEn: text(item.title_en, primary.title_en),
    url,
    source,
    sourceNames,
    sourceCount: Math.max(1, numberValue(item.source_count, item.item_count, sourceNames.length)),
    publishedAt: text(item.latest_at, item.published_at, primary.published_at, item.first_seen_at),
    reason: text(item.recommend_reason_zh, primary.recommend_reason_zh, item.summary, primary.summary, '该条目通过 AI 相关度与信源质量筛选。'),
    score: rawScore ? Math.round(rawScore <= 1 ? rawScore * 100 : rawScore) : undefined,
    category: categoryFor(item),
  }
}

export function buildRadarData(latestValue: unknown, briefValue: unknown): RadarData {
  const latest = objectValue(latestValue)
  const brief = objectValue(briefValue)
  const normalize = (value: unknown) => (Array.isArray(value) ? value : []).map(normalizeRadarStory).filter((item): item is RadarStory => Boolean(item)).sort((a, b) => new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime())
  return {
    generatedAt: text(latest.generated_at, brief.generated_at),
    totalItems: numberValue(latest.total_items, Array.isArray(latest.items) ? latest.items.length : 0),
    sourceCount: numberValue(latest.source_count),
    curated: normalize(brief.items),
    all: normalize(latest.items),
  }
}
