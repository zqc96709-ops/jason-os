import { describe, expect, it } from 'vitest'
import { briefingSignals, detectExternalSignals, type ExternalItem } from './externalIntelligence'

const item = (id: string, platform: string, date: string, author: string, likes = 10): ExternalItem => ({ id, platform, contentType: 'VIDEO_POST', title: id, content: '', author, canonicalUrl: `https://example.com/${id}`, capturedAt: date, publishedAt: date, provider: 'fixture', metrics: { views: likes * 10, likes, comments: 0, shares: 0, saves: 0 } })

describe('External Intelligence signal engine', () => {
  it('requires an absolute increase and independent authors for growth signals', () => {
    const anchor = new Date('2026-08-11T12:00:00Z')
    const baseline = Array.from({ length: 10 }, (_, index) => item(`b${index}`, '抖音', '2026-07-31T12:00:00Z', `b${index}`))
    const current = Array.from({ length: 21 }, (_, index) => item(`c${index}`, '抖音', '2026-08-08T12:00:00Z', `c${index}`))
    expect(detectExternalSignals([...baseline, ...current], anchor).some((signal) => signal.type === 'GROWTH')).toBe(true)
    expect(detectExternalSignals([...baseline, ...current.map((entry) => ({ ...entry, author: 'same' }))], anchor).some((signal) => signal.type === 'GROWTH')).toBe(false)
  })

  it('detects viral items only against a sufficient platform sample', () => {
    const anchor = new Date('2026-08-11T12:00:00Z')
    const items = Array.from({ length: 10 }, (_, index) => item(`i${index}`, '小红书', '2026-08-09T12:00:00Z', `a${index}`, index === 9 ? 100 : 10))
    expect(detectExternalSignals(items, anchor).find((signal) => signal.type === 'VIRAL')?.evidenceItemIds).toEqual(['i9'])
  })

  it('limits the CEO briefing to seven signals', () => {
    const signals = Array.from({ length: 10 }, (_, index) => ({ key: String(index), type: 'GROWTH' as const, title: '', summary: '', status: 'DETECTED' as const, platform: '', baselineValue: 1, currentValue: index + 2, baselineStart: '', baselineEnd: '', currentStart: '', currentEnd: '', changeRate: index, sampleSize: 10, independentAuthorCount: 5, calculationMethod: '', evidenceItemIds: [], detectedAt: '' }))
    expect(briefingSignals(signals)).toHaveLength(7)
    expect(briefingSignals(signals)[0].changeRate).toBe(9)
  })
})
