import { useState } from 'react'
import { DEFAULT_ZOOM, stepZoom } from './zoom'

// Zoom and fullscreen. Lives with the main area, so the zoom is the same for
// every document during the session (specs/07-reader.md).
export function useReaderState() {
  const [zoom, setZoom] = useState(DEFAULT_ZOOM)
  const [fullscreen, setFullscreen] = useState(false)
  return {
    zoom,
    fullscreen,
    zoomIn: () => setZoom((value) => stepZoom(value, 1)),
    zoomOut: () => setZoom((value) => stepZoom(value, -1)),
    resetZoom: () => setZoom(DEFAULT_ZOOM),
    toggleFullscreen: () => setFullscreen((value) => !value),
    exitFullscreen: () => setFullscreen(false),
  }
}

export type ReaderState = ReturnType<typeof useReaderState>
