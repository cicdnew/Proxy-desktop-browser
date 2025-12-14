import { writable, get } from 'svelte/store';
import { persist } from './persist';

interface AppSettings {
  virtualScrollEnabled: boolean;
  virtualScrollThreshold: number;
  connectionPoolEnabled: boolean;
  performanceMonitoring: boolean;
  autoCleanupEnabled: boolean;
}

const defaultSettings: AppSettings = {
  virtualScrollEnabled: true,
  virtualScrollThreshold: 50, // Enable virtual scroll when tabs > 50
  connectionPoolEnabled: true,
  performanceMonitoring: true,
  autoCleanupEnabled: true,
};

// Create persistent settings store
export const settingsStore = persist(
  writable<AppSettings>(defaultSettings),
  'virtual-ip-browser-settings'
);

// Individual setting exports for convenience
export const virtualScrollEnabled = writable(get(settingsStore).virtualScrollEnabled);
export const virtualScrollThreshold = writable(get(settingsStore).virtualScrollThreshold);
export const connectionPoolEnabled = writable(get(settingsStore).connectionPoolEnabled);
export const performanceMonitoring = writable(get(settingsStore).performanceMonitoring);
export const autoCleanupEnabled = writable(get(settingsStore).autoCleanupEnabled);

// Sync individual stores with main settings store
settingsStore.subscribe(settings => {
  virtualScrollEnabled.set(settings.virtualScrollEnabled);
  virtualScrollThreshold.set(settings.virtualScrollThreshold);
  connectionPoolEnabled.set(settings.connectionPoolEnabled);
  performanceMonitoring.set(settings.performanceMonitoring);
  autoCleanupEnabled.set(settings.autoCleanupEnabled);
});

// Update settings helper
export function updateSetting<K extends keyof AppSettings>(
  key: K,
  value: AppSettings[K]
): void {
  settingsStore.update(settings => ({
    ...settings,
    [key]: value
  }));
}

// Batch update settings
export function updateSettings(newSettings: Partial<AppSettings>): void {
  settingsStore.update(settings => ({
    ...settings,
    ...newSettings
  }));
}

// Reset to defaults
export function resetSettings(): void {
  settingsStore.set(defaultSettings);
}

// Get all settings
export function getAllSettings(): AppSettings {
  return get(settingsStore);
}
