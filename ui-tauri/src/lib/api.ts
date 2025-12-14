import { invoke } from '@tauri-apps/api/tauri';
import type { 
  Tab, WebviewTab, VirtualIPResponse, ValidationResponse, Country,
  ProxySettings, FreeProxy, ProxyTestResult, PublicIpInfo,
  BackupOptions, BackupInfo, BrowserState, BrowserSettings,
  HistoryEntry, Bookmark
} from './types';
import { apiCache } from './utils';
import { invokeWithRetry, errorHandler } from './errorHandling';

// Auth API stubs
export async function loginUser(credentials: { username: string; password: string }) {
  console.log('Stub: loginUser', credentials);
  return { id: 'demo', username: credentials.username, token: 'stub-token' };
}

export async function registerUser(userData: { username: string; email: string; password: string }) {
  console.log('Stub: registerUser', userData);
  return { id: 'demo', username: userData.username, email: userData.email };
}

export async function createEnterpriseUser(userData: any) {
  console.log('Stub: createEnterpriseUser', userData);
  return { id: 'demo', ...userData };
}

export async function promoteUserToAdmin(userId: string) {
  console.log('Stub: promoteUserToAdmin', userId);
  return { success: true };
}

// Tab Management (Legacy - for IP management)
export async function fetchTabs(): Promise<Tab[]> {
  const cacheKey = 'tabs';
  let tabs = apiCache.get<Tab[]>(cacheKey);
  if (tabs) return tabs;
  
  tabs = await invokeWithRetry('list_tabs', undefined, {
    maxAttempts: 3,
    baseDelay: 1000,
    retryCondition: (error) => error.name === 'NetworkError' || error.status >= 500
  });
  apiCache.set(cacheKey, tabs, 30000); // Cache for 30 seconds
  return tabs;
}

export async function fetchCountries(): Promise<Country[]> {
  const cacheKey = 'countries';
  let countries = apiCache.get<Country[]>(cacheKey);
  if (countries) return countries;
  
  countries = await invoke('list_countries');
  apiCache.set(cacheKey, countries, 300000); // Cache for 5 minutes
  return countries;
}

export async function createTab(countryCode: string): Promise<Tab> {
  const tab = await invoke('create_tab', { countryCode });
  apiCache.delete('tabs'); // Invalidate cache
  return tab;
}

export async function createTabRandom(): Promise<Tab> {
  const tab = await invoke('create_tab_random');
  apiCache.delete('tabs'); // Invalidate cache
  return tab;
}

// Webview Tab Management (Native browser windows)
export async function fetchWebviewTabs(): Promise<WebviewTab[]> {
  return invoke('get_webview_tabs');
}

export async function createWebviewTab(url?: string): Promise<WebviewTab> {
  return invoke('create_webview_tab', { url });
}

export async function navigateWebviewTab(tabId: string, url: string): Promise<void> {
  return invoke('navigate_webview_tab', { tabId, url });
}

export async function closeWebviewTab(tabId: string): Promise<void> {
  return invoke('close_webview_tab', { tabId });
}

export async function focusWebviewTab(tabId: string): Promise<void> {
  return invoke('focus_webview_tab', { tabId });
}


export async function rotateIp(tabId: string, newCountry?: string): Promise<VirtualIPResponse> {
  return invoke('rotate_ip', { tabId, newCountry: newCountry ?? null });
}

export async function validateIp(tabId: string): Promise<ValidationResponse> {
  return invoke('validate_ip', { tabId });
}

// Proxy Management
export async function getProxySettings(): Promise<ProxySettings> {
  return invoke('get_proxy_settings');
}

export async function setProxySettings(settings: ProxySettings): Promise<void> {
  return invoke('set_proxy_settings', { settings });
}

export async function getActiveProxy(): Promise<FreeProxy | null> {
  return invoke('get_active_proxy');
}

export async function setActiveProxy(proxy: FreeProxy | null): Promise<void> {
  return invoke('set_active_proxy', { proxy });
}

// Public IP Detection
export async function detectPublicIp(): Promise<PublicIpInfo> {
  return invoke('detect_public_ip');
}

// Free IP Providers
export async function fetchFreeProxies(): Promise<FreeProxy[]> {
  return invoke('fetch_free_proxies');
}

export async function getFreeProxies(): Promise<FreeProxy[]> {
  return invoke('get_free_proxies');
}

export async function testProxy(proxy: FreeProxy): Promise<ProxyTestResult> {
  return invoke('test_proxy', { proxy });
}

export async function clearFreeProxies(): Promise<void> {
  return invoke('clear_free_proxies');
}

export async function removeDeadProxies(): Promise<void> {
  return invoke('remove_dead_proxies');
}

// Backup & Restore
export async function createBackup(options: BackupOptions): Promise<BackupInfo> {
  return invoke('create_backup', { options });
}

export async function listBackups(): Promise<BackupInfo[]> {
  return invoke('list_backups');
}

export async function restoreBackup(path: string, password?: string): Promise<void> {
  return invoke('restore_backup', { path, password: password ?? null });
}

export async function deleteBackup(id: string): Promise<void> {
  return invoke('delete_backup', { id });
}

// Tab Close
export async function closeTab(tabId: string): Promise<void> {
  await invoke('close_tab', { tabId });
  apiCache.delete('tabs'); // Invalidate cache
}

// Browser Controls
export async function navigate(tabId: string, url: string): Promise<BrowserState> {
  return invoke('navigate', { tabId, url });
}

export async function goBack(tabId: string): Promise<string | null> {
  return invoke('go_back', { tabId });
}

export async function goForward(tabId: string): Promise<string | null> {
  return invoke('go_forward', { tabId });
}

export async function reloadPage(tabId: string): Promise<string | null> {
  return invoke('reload_page', { tabId });
}

export async function getBrowserState(tabId: string): Promise<BrowserState | null> {
  return invoke('get_browser_state', { tabId });
}

export async function updatePageTitle(tabId: string, title: string): Promise<void> {
  return invoke('update_page_title', { tabId, title });
}

export async function getBrowserSettings(): Promise<BrowserSettings> {
  const cacheKey = 'browser_settings';
  let settings = apiCache.get<BrowserSettings>(cacheKey);
  if (settings) return settings;
  
  settings = await invoke('get_browser_settings');
  apiCache.set(cacheKey, settings, 600000); // Cache for 10 minutes
  return settings;
}

export async function setBrowserSettings(settings: BrowserSettings): Promise<void> {
  await invoke('set_browser_settings', { settings });
  apiCache.delete('browser_settings'); // Clear cache after update
}

// History
export async function getHistory(limit: number = 100): Promise<HistoryEntry[]> {
  return invoke('get_history', { limit });
}

export async function searchHistory(query: string): Promise<HistoryEntry[]> {
  return invoke('search_history', { query });
}

export async function clearHistory(): Promise<void> {
  return invoke('clear_history');
}

// Bookmarks
export async function addBookmark(url: string, title: string, folder?: string): Promise<number> {
  return invoke('add_bookmark', { url, title, folder: folder ?? null });
}

export async function getBookmarks(): Promise<Bookmark[]> {
  return invoke('get_bookmarks');
}

export async function deleteBookmark(id: number): Promise<void> {
  return invoke('delete_bookmark', { id });
}
