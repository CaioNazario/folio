import '@fontsource/ibm-plex-mono/400.css'
import '@fontsource/ibm-plex-mono/500.css'
import '@fontsource/ibm-plex-sans/400.css'
import '@fontsource/ibm-plex-sans/500.css'
import '@fontsource/ibm-plex-sans/600.css'
import '@fontsource/source-serif-4/600.css'
import './app/styles.css'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './app/App'
import { loadPreferences } from './features/preferences/api'
import { applyTheme, resolveTheme } from './features/preferences/theme'
import { systemDark } from './features/preferences/useSystemDark'
import { initI18n, languageForPreference } from './lib/i18n'

const root = document.getElementById('root')
if (!root) throw new Error('#root element not found in index.html')

// Stored theme and language apply before the first render, so the window
// never flashes the wrong theme or language.
const preferences = await loadPreferences()
applyTheme(resolveTheme(preferences.theme, systemDark()))
initI18n(await languageForPreference(preferences.language))

createRoot(root).render(
  <StrictMode>
    <App preferences={preferences} />
  </StrictMode>,
)
