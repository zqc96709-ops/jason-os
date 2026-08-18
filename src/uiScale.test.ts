import { describe, expect, it } from 'vitest'
import { adjustUiScale, normalizeUiScale } from './uiScale'

describe('manual UI scale', () => {
  it('defaults invalid stored values to a readable 110 percent', () => {
    expect(normalizeUiScale(Number.NaN)).toBe(110)
    expect(normalizeUiScale(0)).toBe(110)
  })

  it('snaps and clamps scale to the supported 90 to 140 range', () => {
    expect(normalizeUiScale(117)).toBe(120)
    expect(normalizeUiScale(70)).toBe(90)
    expect(normalizeUiScale(180)).toBe(140)
  })

  it('zooms in, out and resets without leaving the supported range', () => {
    expect(adjustUiScale(110, 'in')).toBe(120)
    expect(adjustUiScale(110, 'out')).toBe(100)
    expect(adjustUiScale(140, 'in')).toBe(140)
    expect(adjustUiScale(90, 'out')).toBe(90)
    expect(adjustUiScale(130, 'reset')).toBe(110)
  })
})
