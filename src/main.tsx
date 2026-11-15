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
import { initI18n, languageForPreference } from './lib/i18n'

const root = document.getElementById('root')
if (!root) throw new Error('#root element not found in index.html')

// The stored language preference comes with the backend; until then the
// interface follows the system.
initI18n(await languageForPreference('system'))

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
