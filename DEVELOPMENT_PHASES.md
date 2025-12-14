# Development Phases: Virtual IP Browser
## Detailed Implementation Guide with Claude Opus 4.5 Prompts

---

## PHASE 1: Core Browser Engine Enhancement

### Objective:
Implement proper browser functionality with WebView integration, tab management, and navigation controls.

---

### 1.1 WebView Manager Enhancement

**File**: `crates/browser-core/src/webview_manager.rs`

**Claude Opus 4.5 Prompt:**
```
I need you to implement a complete WebView manager for a Tauri-based browser application. The manager should:

REQUIREMENTS:
1. Create and manage multiple WebView instances (one per tab)
2. Handle WebView lifecycle (creation, destruction, switching)
3. Implement proper isolation between tabs (cookies, localStorage, sessionStorage)
4. Support navigation (forward, back, reload, stop)
5. Capture navigation events (page load, title changes, URL changes)
6. Handle WebView errors and crashes gracefully
7. Support custom user agents per tab
8. Implement zoom controls per tab

TECHNICAL CONSTRAINTS:
- Use Tauri 2.0 WebView API
- Must work cross-platform (Windows, macOS, Linux)
- Each tab should be completely isolated
- Support for programmatic navigation
- Event callback system for UI updates

STRUCTURE:
```rust
pub struct WebViewManager {
    webviews: HashMap<TabId, WebViewHandle>,
    active_tab: Option<TabId>,
    tauri_app_handle: AppHandle,
}

impl WebViewManager {
    pub fn create_tab(&mut self, url: &str, proxy_config: ProxyConfig) -> Result<TabId>
    pub fn destroy_tab(&mut self, tab_id: TabId) -> Result<()>
    pub fn switch_to_tab(&mut self, tab_id: TabId) -> Result<()>
    pub fn navigate(&mut self, tab_id: TabId, url: &str) -> Result<()>
    pub fn go_back(&mut self, tab_id: TabId) -> Result<()>
    pub fn go_forward(&mut self, tab_id: TabId) -> Result<()>
    pub fn reload(&mut self, tab_id: TabId) -> Result<()>
    pub fn stop(&mut self, tab_id: TabId) -> Result<()>
    pub fn set_zoom(&mut self, tab_id: TabId, level: f64) -> Result<()>
}
```

Please implement this with proper error handling, async support, and comprehensive documentation.
```

---

### 1.2 Tab Manager Implementation

**File**: `crates/browser-core/src/tab_manager.rs`

**Claude Opus 4.5 Prompt:**
```
Implement a comprehensive tab management system for a privacy-focused browser. Each tab needs:

REQUIREMENTS:
1. Unique tab ID and metadata (title, URL, favicon, loading state)
2. Tab lifecycle management (create, close, reorder)
3. Tab state persistence (for session restore)
4. Tab isolation with separate proxy configurations per tab
5. History tracking per tab
6. Tab pinning functionality
7. Tab grouping/organization
8. Memory management (tab suspension for inactive tabs)

DATA STRUCTURES:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub proxy_id: Option<ProxyId>,
    pub virtual_ip: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_pinned: bool,
    pub is_suspended: bool,
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab_index: usize,
    max_tabs: usize,
}
```

Implement full CRUD operations, tab ordering, and state management with proper error handling.
```

---

### 1.3 Browser Controls & Navigation

**File**: `crates/browser-core/src/browser_controls.rs`

**Claude Opus 4.5 Prompt:**
```
Create a browser control system that handles all user interactions with the browser. Implement:

FEATURES:
1. Navigation controls (back, forward, reload, stop, home)
2. Address bar functionality (URL validation, search engine integration)
3. Bookmark management (add, remove, folders, import/export)
4. History management (view, clear, search)
5. Download manager (track downloads, pause/resume, open location)
6. Developer tools integration (inspect element, console access)
7. Page zoom controls (in, out, reset)
8. Find in page functionality
9. Print page functionality
10. View source functionality

INTEGRATION REQUIREMENTS:
- Must communicate with WebViewManager for actions
- Should emit events for UI updates
- Handle keyboard shortcuts
- Support context menus (right-click)
- Implement search suggestions
- URL autocomplete from history

Please provide a clean API with async/await support and comprehensive error handling.
```

---

## PHASE 2: Proxy & Virtual IP Integration

### Objective:
Implement network traffic routing through proxy servers with virtual IP rotation.

---

### 2.1 HTTP Proxy Implementation

**File**: `crates/browser-core/src/proxy.rs`

**Claude Opus 4.5 Prompt:**
```
Implement a complete HTTP/HTTPS proxy system for routing browser traffic. Requirements:

CORE FUNCTIONALITY:
1. HTTP proxy connection establishment
2. HTTPS proxy with CONNECT tunnel support
3. SOCKS4/SOCKS5 proxy support
4. Proxy authentication (username/password)
5. Connection pooling and reuse
6. Proxy failover and retry logic
7. Request/response interception
8. Header modification (User-Agent, X-Forwarded-For, etc.)
9. Cookie management per proxy
10. DNS resolution through proxy

DATA STRUCTURES:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: ProxyId,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
    pub countries: Vec<String>,
    pub speed_mbps: Option<f64>,
    pub uptime_percent: Option<f64>,
    pub last_checked: Option<DateTime<Utc>>,
}

pub enum ProxyType {
    HTTP,
    HTTPS,
    SOCKS4,
    SOCKS5,
}

pub struct ProxyManager {
    active_proxies: HashMap<ProxyId, ProxyConfig>,
    connection_pool: ConnectionPool,
    health_checker: ProxyHealthChecker,
}
```

REQUIREMENTS:
- Use reqwest for HTTP client with proxy support
- Implement connection health checks
- Support automatic proxy rotation
- Handle proxy errors gracefully
- Log all proxy connections for debugging
- Implement proxy performance metrics

Provide a production-ready implementation with comprehensive error handling and logging.
```

---

### 2.2 Network Request Interceptor

**File**: `crates/browser-core/src/http_client.rs`

**Claude Opus 4.5 Prompt:**
```
Create a network request interceptor that sits between the browser and the internet. This is critical for routing all traffic through proxies.

REQUIREMENTS:
1. Intercept ALL HTTP/HTTPS requests from WebView
2. Route requests through configured proxy
3. Support request modification before sending:
   - Headers (User-Agent, Accept-Language, etc.)
   - Cookies
   - Referrer
4. Support response modification before rendering:
   - Content Security Policy headers
   - Cookie attributes
   - CORS headers
5. Request/response logging for debugging
6. Block requests based on rules (ad blocking, tracking)
7. WebSocket proxy support
8. Support for request priorities
9. Bandwidth throttling per tab
10. Request caching with proxy-aware cache keys

ARCHITECTURE:
```rust
pub struct HttpInterceptor {
    proxy_manager: Arc<Mutex<ProxyManager>>,
    request_filters: Vec<Box<dyn RequestFilter>>,
    response_filters: Vec<Box<dyn ResponseFilter>>,
    cache: RequestCache,
}

impl HttpInterceptor {
    pub async fn intercept_request(&self, req: Request) -> Result<Request>
    pub async fn intercept_response(&self, res: Response) -> Result<Response>
    pub async fn send_request(&self, req: Request, proxy: ProxyConfig) -> Result<Response>
}
```

INTEGRATION:
- Hook into Tauri's protocol handler
- Register custom protocol scheme (e.g., vip://)
- Intercept http:// and https:// requests
- Support file:// for local files

Implement with async/await, proper error handling, and detailed logging.
```

---

### 2.3 Virtual IP Rotation System

**File**: `crates/virtual-ip/src/rotation.rs`

**Claude Opus 4.5 Prompt:**
```
Implement an intelligent proxy/virtual IP rotation system that automatically switches between proxies based on various strategies.

ROTATION STRATEGIES:
1. Time-based rotation (every N minutes)
2. Request-based rotation (every N requests)
3. Domain-based rotation (different proxy per domain)
4. Geographic rotation (rotate through specific countries)
5. Performance-based rotation (use fastest proxies)
6. Round-robin rotation
7. Random rotation
8. Sticky sessions (same proxy for same domain)
9. Failure-triggered rotation (switch on proxy failure)
10. Manual rotation (user-triggered)

DATA STRUCTURES:
```rust
pub enum RotationStrategy {
    TimeBased(Duration),
    RequestBased(u32),
    DomainBased,
    Geographic(Vec<String>),
    PerformanceBased,
    RoundRobin,
    Random,
    StickySession,
    OnFailure,
    Manual,
}

pub struct ProxyRotator {
    strategy: RotationStrategy,
    proxy_pool: Vec<ProxyConfig>,
    current_proxy_index: usize,
    request_count: u32,
    last_rotation: DateTime<Utc>,
    domain_proxy_map: HashMap<String, ProxyId>,
    performance_metrics: HashMap<ProxyId, ProxyMetrics>,
}

#[derive(Debug)]
pub struct ProxyMetrics {
    pub response_time_ms: f64,
    pub success_rate: f64,
    pub last_success: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
}
```

REQUIREMENTS:
- Support multiple concurrent strategies
- Implement proxy health scoring
- Automatic removal of dead proxies
- Proxy performance tracking
- Geographic distribution awareness
- Session persistence for specific domains
- Configurable rotation parameters
- Event notifications on rotation

Provide a robust implementation with comprehensive testing support.
```

---

## PHASE 3: Free Proxy Provider Integration

### Objective:
Integrate multiple free proxy provider APIs to automatically fetch and maintain a pool of working proxies.

---

### 3.1 Proxy Provider Abstraction

**File**: `crates/browser-core/src/free_ip_providers.rs`

**Claude Opus 4.5 Prompt:**
```
Create a unified interface for integrating multiple free proxy provider APIs. The system should automatically fetch, validate, and maintain a pool of working proxies.

PROVIDER SOURCES TO INTEGRATE:
1. ProxyScrape API (https://api.proxyscrape.com/)
2. Free Proxy List (https://free-proxy-list.net/)
3. PubProxy (http://pubproxy.com/)
4. ProxyNova (https://www.proxynova.com/)
5. Geonode (https://geonode.com/free-proxy-list)
6. Spys.one (http://spys.one/)
7. HideMyName (https://hidemy.name/en/proxy-list/)
8. ProxyDB (http://proxydb.net/)

ARCHITECTURE:
```rust
#[async_trait]
pub trait ProxyProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>>;
    async fn fetch_proxies_filtered(&self, filter: ProxyFilter) -> Result<Vec<ProxyConfig>>;
    fn rate_limit(&self) -> Duration;
    fn requires_api_key(&self) -> bool;
}

pub struct ProxyFilter {
    pub countries: Option<Vec<String>>,
    pub proxy_types: Option<Vec<ProxyType>>,
    pub anonymity_levels: Option<Vec<AnonymityLevel>>,
    pub min_uptime_percent: Option<f64>,
    pub max_response_time_ms: Option<u32>,
    pub https_support: Option<bool>,
}

pub enum AnonymityLevel {
    Transparent,
    Anonymous,
    Elite,
}

pub struct ProxyProviderManager {
    providers: Vec<Box<dyn ProxyProvider>>,
    proxy_pool: Vec<ProxyConfig>,
    last_update: HashMap<String, DateTime<Utc>>,
    update_interval: Duration,
}
```

REQUIREMENTS:
- Implement each provider with their specific API/scraping logic
- Handle rate limiting per provider
- Parse different response formats (JSON, HTML, CSV)
- Validate fetched proxies before adding to pool
- Remove duplicate proxies
- Schedule automatic updates
- Support API keys for premium tiers
- Handle provider failures gracefully
- Log all fetch operations
- Implement caching to avoid excessive API calls

For each provider, implement:
1. API endpoint construction
2. Response parsing
3. Error handling
4. Rate limiting
5. Proxy validation

Provide production-ready code with proper error handling and logging.
```

---

### 3.2 Proxy Validation & Health Checking

**File**: `crates/virtual-ip/src/validator.rs`

**Claude Opus 4.5 Prompt:**
```
Implement a comprehensive proxy validation and health checking system to ensure only working proxies are used.

VALIDATION CHECKS:
1. Connection test (can we connect to the proxy?)
2. HTTP/HTTPS functionality test
3. Anonymity level verification (check HTTP headers)
4. Speed test (measure response time)
5. IP leak detection (WebRTC, DNS leaks)
6. Geographic location verification
7. Port scanning detection
8. SSL/TLS support verification
9. Cookie handling test
10. JavaScript execution test (some proxies block JS)

HEALTH MONITORING:
- Periodic health checks (every N minutes)
- Real-time failure detection
- Performance metrics collection
- Automatic proxy quarantine on failures
- Revalidation of quarantined proxies
- Proxy scoring system

DATA STRUCTURES:
```rust
pub struct ProxyValidator {
    test_urls: Vec<String>,
    timeout: Duration,
    concurrent_checks: usize,
    validation_cache: HashMap<ProxyId, ValidationResult>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_working: bool,
    pub response_time_ms: u64,
    pub anonymity_level: AnonymityLevel,
    pub detected_country: Option<String>,
    pub detected_ip: Option<IpAddr>,
    pub supports_https: bool,
    pub has_ip_leak: bool,
    pub error: Option<String>,
    pub validated_at: DateTime<Utc>,
}

pub struct ProxyHealthChecker {
    validator: ProxyValidator,
    check_interval: Duration,
    max_consecutive_failures: u32,
    quarantine_duration: Duration,
}
```

VALIDATION PROCESS:
1. Basic connectivity test (fastest)
2. HTTP GET request to test URL
3. Check response headers for proxy traces
4. Verify IP address matches proxy country
5. Test HTTPS if applicable
6. Measure response time
7. Check for IP leaks (WebRTC, DNS)
8. Record all metrics

INTEGRATION:
- Run validation before adding proxy to pool
- Schedule periodic revalidation
- Remove proxies after N consecutive failures
- Provide real-time health status to UI

Implement with tokio for concurrent validation, proper error handling, and detailed logging.
```

---

### 3.3 Proxy Database & Persistence

**File**: `crates/browser-core/src/storage.rs`

**Claude Opus 4.5 Prompt:**
```
Implement a persistent storage system for proxies, their metadata, and performance history using SQLite.

DATABASE SCHEMA:
```sql
CREATE TABLE proxies (
    id TEXT PRIMARY KEY,
    proxy_type TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT,
    password TEXT,
    country TEXT,
    anonymity_level TEXT,
    source_provider TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL,
    last_validated TEXT,
    UNIQUE(host, port)
);

CREATE TABLE proxy_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proxy_id TEXT NOT NULL,
    response_time_ms REAL NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    checked_at TEXT NOT NULL,
    FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

CREATE TABLE proxy_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proxy_id TEXT NOT NULL,
    tab_id TEXT NOT NULL,
    domain TEXT,
    request_count INTEGER DEFAULT 0,
    bytes_transferred INTEGER DEFAULT 0,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

RUST IMPLEMENTATION:
```rust
pub struct ProxyDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl ProxyDatabase {
    pub async fn new(path: &str) -> Result<Self>
    
    // Proxy CRUD
    pub async fn insert_proxy(&self, proxy: &ProxyConfig) -> Result<()>
    pub async fn update_proxy(&self, proxy: &ProxyConfig) -> Result<()>
    pub async fn delete_proxy(&self, id: &ProxyId) -> Result<()>
    pub async fn get_proxy(&self, id: &ProxyId) -> Result<Option<ProxyConfig>>
    pub async fn list_proxies(&self, filter: ProxyFilter) -> Result<Vec<ProxyConfig>>
    
    // Metrics
    pub async fn record_metric(&self, proxy_id: &ProxyId, metric: ProxyMetric) -> Result<()>
    pub async fn get_metrics(&self, proxy_id: &ProxyId, since: DateTime<Utc>) -> Result<Vec<ProxyMetric>>
    pub async fn get_proxy_statistics(&self, proxy_id: &ProxyId) -> Result<ProxyStatistics>
    
    // Sessions
    pub async fn create_session(&self, session: ProxySession) -> Result<SessionId>
    pub async fn end_session(&self, session_id: SessionId) -> Result<()>
    pub async fn update_session_stats(&self, session_id: SessionId, stats: SessionStats) -> Result<()>
    
    // Settings
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>>
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()>
    
    // Maintenance
    pub async fn cleanup_old_metrics(&self, days: u32) -> Result<usize>
    pub async fn cleanup_inactive_proxies(&self, days: u32) -> Result<usize>
    pub async fn vacuum(&self) -> Result<()>
}
```

REQUIREMENTS:
- Use rusqlite for SQLite access
- Implement connection pooling
- Support async operations (use tokio)
- Handle database migrations
- Implement proper indexing for performance
- Add transaction support
- Implement backup/restore functionality
- Add database optimization (VACUUM, ANALYZE)

Provide a complete, production-ready implementation with error handling and comprehensive documentation.
```

---

## PHASE 4: UI/UX Implementation

### Objective:
Create a modern, intuitive browser interface with Svelte components.

---

### 4.1 Main Browser Window

**File**: `ui-tauri/src/App.svelte`

**Claude Opus 4.5 Prompt:**
```
Create the main browser application UI using Svelte. The interface should be clean, modern, and privacy-focused.

LAYOUT STRUCTURE:
```
┌─────────────────────────────────────────────────────────────┐
│  [Tab1] [Tab2] [Tab3] [+]                     [_] [□] [X]   │
├─────────────────────────────────────────────────────────────┤
│  [←] [→] [⟳] [🏠] [🔒] https://example.com    [☆] [⚙]      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│                      WebView Content Area                     │
│                                                               │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│  🌍 US Proxy | 🟢 Connected | ↓ 1.2 MB/s | ⏱ 45ms          │
└─────────────────────────────────────────────────────────────┘
```

COMPONENTS TO CREATE:
1. TabBar (tab list, add tab button)
2. NavigationBar (back, forward, reload, home)
3. AddressBar (URL input, SSL indicator, bookmark star)
4. ToolBar (settings, downloads, history, bookmarks)
5. StatusBar (proxy status, speed, latency)
6. WebViewContainer (renders the active tab's WebView)
7. ContextMenu (right-click menus)
8. SettingsPanel (sliding panel or modal)
9. ProxySelector (dropdown to choose proxy)
10. NotificationToast (for errors, warnings, info)

SVELTE STRUCTURE:
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import TabBar from './components/TabBar.svelte';
  import NavigationBar from './components/NavigationBar.svelte';
  import AddressBar from './components/AddressBar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  
  let tabs = [];
  let activeTabId = null;
  let currentUrl = '';
  let proxyStatus = { connected: false, country: '', speed: 0, latency: 0 };
  
  // Implement IPC communication with Rust backend
  // Handle keyboard shortcuts
  // Manage application state
</script>

<div class="browser-window">
  <TabBar {tabs} {activeTabId} on:tabChange={handleTabChange} on:newTab={createNewTab} on:closeTab={closeTab} />
  <div class="navigation-area">
    <NavigationBar on:back={goBack} on:forward={goForward} on:reload={reload} on:home={goHome} />
    <AddressBar bind:url={currentUrl} on:navigate={navigateToUrl} />
  </div>
  <div class="webview-container">
    <!-- WebView will be injected here by Tauri -->
  </div>
  <StatusBar {proxyStatus} />
  <SettingsPanel />
</div>

<style>
  /* Modern, clean styling */
  /* Dark mode support */
  /* Responsive design */
</style>
```

REQUIREMENTS:
- Modern, minimalist design (similar to Chrome/Arc)
- Dark mode and light mode support
- Smooth animations and transitions
- Keyboard shortcut support (Ctrl+T, Ctrl+W, Ctrl+Tab, etc.)
- Context menus for tabs and content
- Responsive to window resizing
- Accessibility support (ARIA labels, keyboard navigation)
- Performance optimized (virtualized tab list for many tabs)

Implement with TypeScript for type safety and comprehensive error handling.
```

---

### 4.2 Tab Bar Component

**File**: `ui-tauri/src/components/TabBar.svelte`

**Claude Opus 4.5 Prompt:**
```
Create a comprehensive tab bar component for the browser with drag-and-drop reordering and tab management.

FEATURES:
1. Display all open tabs with title and favicon
2. Show active tab with highlight
3. Show loading indicator when tab is loading
4. Close button per tab (X icon, visible on hover)
5. Drag and drop to reorder tabs
6. Tab context menu (right-click):
   - Close Tab
   - Close Other Tabs
   - Close Tabs to the Right
   - Close All Tabs
   - Duplicate Tab
   - Pin/Unpin Tab
   - Mute/Unmute Tab
7. New tab button (+)
8. Tab overflow handling (show arrows for scrolling)
9. Tab width adjustment based on number of tabs
10. Pinned tab support (smaller, left-aligned)
11. Tab groups/colors (optional)
12. Show proxy indicator per tab (flag icon or color)

COMPONENT STRUCTURE:
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Tab } from '../lib/types';
  
  export let tabs: Tab[];
  export let activeTabId: string;
  
  const dispatch = createEventDispatcher();
  
  let draggedTab: Tab | null = null;
  let dropTargetIndex: number | null = null;
  
  function handleTabClick(tab: Tab) {
    dispatch('tabChange', { tabId: tab.id });
  }
  
  function handleTabClose(tab: Tab, event: Event) {
    event.stopPropagation();
    dispatch('closeTab', { tabId: tab.id });
  }
  
  function handleNewTab() {
    dispatch('newTab');
  }
  
  function handleDragStart(tab: Tab, event: DragEvent) {
    draggedTab = tab;
    // Implement drag logic
  }
  
  function handleDragOver(index: number, event: DragEvent) {
    event.preventDefault();
    dropTargetIndex = index;
  }
  
  function handleDrop(index: number, event: DragEvent) {
    // Implement drop logic
    dispatch('reorderTabs', { from: draggedTab, to: index });
  }
  
  function handleContextMenu(tab: Tab, event: MouseEvent) {
    event.preventDefault();
    // Show context menu
  }
</script>

<div class="tab-bar">
  <div class="tabs-container">
    {#each tabs as tab, index (tab.id)}
      <div
        class="tab"
        class:active={tab.id === activeTabId}
        class:pinned={tab.isPinned}
        class:loading={tab.loading}
        draggable="true"
        on:click={() => handleTabClick(tab)}
        on:dragstart={(e) => handleDragStart(tab, e)}
        on:dragover={(e) => handleDragOver(index, e)}
        on:drop={(e) => handleDrop(index, e)}
        on:contextmenu={(e) => handleContextMenu(tab, e)}
      >
        {#if tab.favicon}
          <img src={tab.favicon} alt="" class="tab-favicon" />
        {:else}
          <span class="tab-icon">🌐</span>
        {/if}
        
        <span class="tab-title">{tab.title || 'New Tab'}</span>
        
        {#if tab.virtualIp}
          <span class="tab-proxy-indicator" title="Using proxy: {tab.virtualIp}">
            🌍
          </span>
        {/if}
        
        {#if tab.loading}
          <span class="tab-loading-spinner">⟳</span>
        {/if}
        
        {#if !tab.isPinned}
          <button
            class="tab-close"
            on:click={(e) => handleTabClose(tab, e)}
            title="Close tab"
          >
            ×
          </button>
        {/if}
      </div>
    {/each}
  </div>
  
  <button class="new-tab-button" on:click={handleNewTab} title="New tab">
    +
  </button>
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    height: 40px;
    padding: 0 8px;
  }
  
  .tabs-container {
    display: flex;
    flex: 1;
    overflow-x: auto;
    scrollbar-width: none;
  }
  
  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 120px;
    max-width: 240px;
    height: 32px;
    padding: 0 12px;
    background: var(--tab-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px 8px 0 0;
    cursor: pointer;
    transition: all 0.2s;
    user-select: none;
  }
  
  .tab:hover {
    background: var(--tab-bg-hover);
  }
  
  .tab.active {
    background: var(--tab-bg-active);
    border-bottom-color: var(--tab-bg-active);
  }
  
  .tab.pinned {
    min-width: 40px;
    max-width: 40px;
    padding: 0 8px;
  }
  
  .tab-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }
  
  .tab-close {
    opacity: 0;
    transition: opacity 0.2s;
  }
  
  .tab:hover .tab-close {
    opacity: 1;
  }
  
  /* Add more styles */
</style>
```

REQUIREMENTS:
- Smooth animations
- Performant with 100+ tabs
- Keyboard navigation support
- Touch-friendly for tablets
- Accessibility support
- Visual feedback for drag and drop

Implement with full TypeScript support and comprehensive event handling.
```

---

