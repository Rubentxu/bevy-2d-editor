/**
 * AI service settings stored in OPFS.
 *
 * Holds user-configurable settings for the AI-assisted editing proxy,
 * currently just the proxy URL.
 */

import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";

const AI_SETTINGS_FILE = "ai-settings.json";

interface AISettings {
  proxy_url: string;
}

const DEFAULT_PROXY_URL = "http://localhost:11435";

async function loadSettings(): Promise<AISettings> {
  const result = await opfsLoadFile(AI_SETTINGS_FILE);
  if (result.ok && result.value) {
    try {
      return JSON.parse(result.value) as AISettings;
    } catch {
      // Corrupt file — fall through to default
    }
  }
  return { proxy_url: DEFAULT_PROXY_URL };
}

async function saveSettings(settings: AISettings): Promise<void> {
  const result = await opfsSaveFile(AI_SETTINGS_FILE, JSON.stringify(settings));
  if (!result.ok) {
    throw new Error(result.error ?? "Failed to save AI settings");
  }
}

/**
 * Get the configured AI proxy URL.
 * Returns the OPFS-stored URL or the default if not yet configured.
 */
export async function getProxyUrl(): Promise<string> {
  const settings = await loadSettings();
  return settings.proxy_url;
}

/**
 * Persist a new AI proxy URL to OPFS.
 */
export async function setProxyUrl(url: string): Promise<void> {
  await saveSettings({ proxy_url: url });
}
