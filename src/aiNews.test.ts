import { describe, expect, it } from 'vitest'
import { buildRadarData, normalizeRadarStory } from './aiNews'

describe('AI News Radar in-memory data mapping', () => {
  it('keeps original links and recommendation context', () => {
    expect(normalizeRadarStory({ story_id: 's1', title: '中文标题', primary_url: 'https://example.com/news', source: 'OpenAI', source_count: 3, category: 'official', importance_score: 0.91, recommend_reason_zh: '值得关注' })).toMatchObject({
      id: 's1', url: 'https://example.com/news', sourceCount: 3, category: 'official', score: 91, reason: '值得关注',
    })
  })

  it('builds curated and all feeds without persistence-specific fields', () => {
    const data = buildRadarData({ generated_at: '2026-08-10T08:00:00Z', total_items: 1, source_count: 2, items: [{ id: 'all', title: 'All', url: 'https://example.com/all', site_id: 'techurls' }] }, { items: [{ id: 'pick', title: 'Pick', url: 'https://example.com/pick', site_id: 'curated_media' }] })
    expect(data.curated[0].category).toBe('curated')
    expect(data.all[0].category).toBe('aggregate')
    expect(data.totalItems).toBe(1)
  })
})
