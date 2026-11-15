import { locale } from '@tauri-apps/plugin-os'

// System locale from the OS plugin. navigator.language is not used: on
// WebKitGTK it does not always reflect the system LANG. Outside Tauri (plain
// browser) the plugin is missing, so this falls back to null (English).
export async function systemLocale(): Promise<string | null> {
  try {
    return await locale()
  } catch {
    return null
  }
}
