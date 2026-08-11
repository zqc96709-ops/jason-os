export type ExternalMetrics = { views: number; likes: number; comments: number; shares: number; saves: number; followers?: number }
export type ExternalItem = {
  id: string
  platform: string
  externalId?: string
  contentType: string
  title: string
  content: string
  author: string
  canonicalUrl: string
  coverUrl?: string
  publishedAt?: string
  capturedAt: string
  expiresAt?: string
  provider: string
  metrics: ExternalMetrics
}
export type DetectedSignal = {
  key: string
  type: 'GROWTH' | 'VIRAL'
  title: string
  summary: string
  status: 'DETECTED'
  platform: string
  baselineValue: number
  currentValue: number
  baselineStart: string
  baselineEnd: string
  currentStart: string
  currentEnd: string
  changeRate: number
  sampleSize: number
  independentAuthorCount: number
  calculationMethod: string
  evidenceItemIds: string[]
  detectedAt: string
}

const timestamp = (value?: string) => {
  if (!value) return 0
  if (/^\d+$/.test(value)) { const raw = Number(value); return raw > 1e12 ? raw : raw * 1000 }
  const parsed = new Date(value).getTime(); return Number.isNaN(parsed) ? 0 : parsed
}
const engagement = (item: ExternalItem) => item.metrics.likes + item.metrics.comments + item.metrics.shares + item.metrics.saves
const median = (values: number[]) => { const sorted = [...values].sort((a, b) => a - b); if (!sorted.length) return 0; const middle = Math.floor(sorted.length / 2); return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2 }
const uniqueAuthors = (items: ExternalItem[]) => new Set(items.map((item) => item.author.trim()).filter(Boolean)).size

export function detectExternalSignals(items: ExternalItem[], anchor = new Date()): DetectedSignal[] {
  const end = anchor.getTime(); const currentStart = end - 7 * 86_400_000; const baselineStart = end - 14 * 86_400_000
  const windows = { baselineStart: new Date(baselineStart).toISOString(), baselineEnd: new Date(currentStart).toISOString(), currentStart: new Date(currentStart).toISOString(), currentEnd: anchor.toISOString() }
  const platforms = [...new Set(items.map((item) => item.platform).filter(Boolean))]
  const signals: DetectedSignal[] = []
  platforms.forEach((platform) => {
    const platformItems = items.filter((item) => item.platform === platform)
    const current = platformItems.filter((item) => { const time = timestamp(item.publishedAt || item.capturedAt); return time >= currentStart && time <= end })
    const baseline = platformItems.filter((item) => { const time = timestamp(item.publishedAt || item.capturedAt); return time >= baselineStart && time < currentStart })
    const authors = uniqueAuthors(current); const increase = current.length - baseline.length; const changeRate = baseline.length ? increase / baseline.length : 0
    if (baseline.length > 0 && current.length >= 10 && authors >= 5 && increase >= 10 && changeRate >= .5) {
      signals.push({ key: `growth:${platform}:${anchor.toISOString().slice(0, 10)}`, type: 'GROWTH', title: `${platform} 内容活动明显增长`, summary: `最近 7 天 ${current.length} 条，前 7 天 ${baseline.length} 条，增长 ${Math.round(changeRate * 100)}%。`, status: 'DETECTED', platform, baselineValue: baseline.length, currentValue: current.length, ...windows, changeRate, sampleSize: current.length, independentAuthorCount: authors, calculationMethod: '当前7天独立内容数 vs 前7天独立内容数；最低10条、5位作者、绝对新增10条', evidenceItemIds: current.slice(0, 30).map((item) => item.id), detectedAt: anchor.toISOString() })
    }
    const values = platformItems.map(engagement).filter((value) => value > 0); const normal = median(values)
    if (platformItems.length >= 10 && normal > 0) {
      current.filter((item) => engagement(item) >= normal * 3 && engagement(item) >= 50).slice(0, 3).forEach((item) => signals.push({ key: `viral:${item.id}`, type: 'VIRAL', title: `${platform} 出现高互动内容`, summary: `“${item.title}”互动 ${engagement(item)}，约为同平台样本中位数的 ${(engagement(item) / normal).toFixed(1)} 倍。`, status: 'DETECTED', platform, baselineValue: normal, currentValue: engagement(item), ...windows, changeRate: engagement(item) / normal - 1, sampleSize: platformItems.length, independentAuthorCount: uniqueAuthors(platformItems), calculationMethod: '单条互动量 vs 当前样本互动量中位数；最低10条样本且至少3倍', evidenceItemIds: [item.id], detectedAt: anchor.toISOString() }))
    }
  })
  return signals.sort((a, b) => b.changeRate - a.changeRate)
}

export const briefingSignals = (signals: DetectedSignal[]) => [...signals].sort((a, b) => b.changeRate - a.changeRate).slice(0, 7)
