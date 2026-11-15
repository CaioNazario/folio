// Zoom from 50% to 300% in 10% steps (specs/07-reader.md, "Controles").
export const MIN_ZOOM = 50
export const MAX_ZOOM = 300
export const DEFAULT_ZOOM = 100
const STEP = 10

export function stepZoom(zoom: number, direction: 1 | -1): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom + direction * STEP))
}
