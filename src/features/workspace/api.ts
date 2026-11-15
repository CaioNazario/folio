import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type { ScanProgress } from '@/lib/bindings/ScanProgress'

// Opens the last workspace; false on first run (no workspace yet).
export function openLastWorkspace(): Promise<boolean> {
  return invoke<boolean>('open_last_workspace')
}

export function getDefaultWorkspaceDir(): Promise<string> {
  return invoke<string>('get_default_workspace_dir')
}

export function createWorkspace(path: string): Promise<void> {
  return invoke('create_workspace', { path })
}

export function openWorkspace(path: string): Promise<void> {
  return invoke('open_workspace', { path })
}

// System folder picker; null when cancelled.
export async function pickDirectory(defaultPath: string): Promise<string | null> {
  const chosen = await open({ directory: true, defaultPath })
  return typeof chosen === 'string' ? chosen : null
}

export function onScanProgress(handler: (progress: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>('scan-progress', (event) => handler(event.payload))
}

export function onScanFinished(handler: () => void): Promise<UnlistenFn> {
  return listen('scan-finished', () => handler())
}
