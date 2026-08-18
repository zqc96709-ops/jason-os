export const UI_SCALE_MIN = 90
export const UI_SCALE_MAX = 140
export const UI_SCALE_STEP = 10
export const UI_SCALE_DEFAULT = 110

export type UiScaleCommand = 'in' | 'out' | 'reset'

export function normalizeUiScale(value: number) {
  if (!Number.isFinite(value) || value <= 0) return UI_SCALE_DEFAULT
  const snapped = Math.round(value / UI_SCALE_STEP) * UI_SCALE_STEP
  return Math.min(UI_SCALE_MAX, Math.max(UI_SCALE_MIN, snapped))
}

export function adjustUiScale(current: number, command: UiScaleCommand) {
  if (command === 'reset') return UI_SCALE_DEFAULT
  return normalizeUiScale(current + (command === 'in' ? UI_SCALE_STEP : -UI_SCALE_STEP))
}
