import { useEffect, useState } from 'react'

// True only after `active` has stayed true for `delayMs`, so a fast load
// never flashes the loading text (specs/07-reader.md, "Estados do palco").
export function useDelayedFlag(active: boolean, delayMs: number): boolean {
  const [elapsed, setElapsed] = useState(false)
  useEffect(() => {
    if (!active) return
    const timer = setTimeout(() => setElapsed(true), delayMs)
    return () => {
      clearTimeout(timer)
      setElapsed(false)
    }
  }, [active, delayMs])
  return active && elapsed
}
