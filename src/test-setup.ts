import '@testing-library/jest-dom/vitest'
import { clearMocks } from '@tauri-apps/api/mocks'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'
import { initI18n } from '@/lib/i18n'

// Vitest globals are off, so Testing Library cannot register its own cleanup.
afterEach(() => {
  cleanup()
  clearMocks()
})

// Component tests run in Portuguese (Design/specs/11-i18n.md).
initI18n('pt-BR')
