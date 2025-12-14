# Updated Prompts for Optimized Tech Stack
## Claude Opus 4.5 Prompts Adapted for Modern Technologies

---

## 🎯 Overview

This document contains **updated prompts** that leverage the optimized tech stack:
- Tauri 2.0 (instead of 1.5)
- Svelte 5 with Runes (instead of 4)
- sqlx async database (instead of rusqlite)
- reqwest-middleware (auto-retry)
- Security suite (keyring, validator, ammonia)

---

## 📋 Phase 1: Core Browser Engine (UPDATED)

### 1.1 Tab Manager (UPDATED for sqlx)

**File**: `crates/browser-core/src/tab_manager.rs`

**Claude Opus 4.5 Prompt:**
```
I need you to implement a comprehensive tab management system for a privacy-focused browser built with Rust and Tauri 2.0.

REQUIREMENTS:

1. Create a Tab struct with these fields:
   - id: TabId (String newtype)
   - title: String
   - url: String (validated with validator crate)
   - favicon: Option<String>
   - loading: bool
   - can_go_back: bool
   - can_go_forward: bool
   - proxy_id: Option<ProxyId>
   - virtual_ip: Option<IpAddr>
   - created_at: DateTime<Utc>
   - last_active: DateTime<Utc>
   - is_pinned: bool
   - is_suspended: bool

2. Create a TabManager struct with these fields:
   - tabs: Vec<Tab>
   - active_tab_index: usize
   - max_tabs: usize
   - db_pool: SqlitePool (for persistence)

3. Implement these methods (ALL ASYNC with sqlx):
   - async fn new(db_pool: SqlitePool) -> Self
   - async fn create_tab(&mut self, url: &str) -> Result<TabId>
   - async fn close_tab(&mut self, tab_id: TabId) -> Result<()>
   - async fn get_tab(&self, tab_id: TabId) -> Option<&Tab>
   - async fn get_tab_mut(&mut self, tab_id: TabId) -> Option<&mut Tab>
   - async fn switch_to_tab(&mut self, tab_id: TabId) -> Result<()>
   - async fn get_active_tab(&self) -> Option<&Tab>
   - async fn get_all_tabs(&self) -> &Vec<Tab>
   - async fn update_tab_url(&mut self, tab_id: TabId, url: String) -> Result<()>
   - async fn update_tab_title(&mut self, tab_id: TabId, title: String) -> Result<()>
   - async fn pin_tab(&mut self, tab_id: TabId) -> Result<()>
   - async fn unpin_tab(&mut self, tab_id: TabId) -> Result<()>
   - async fn save_to_db(&self) -> Result<()>
   - async fn restore_from_db(&mut self) -> Result<()>

4. Use these dependencies:
   - serde for serialization
   - chrono for timestamps
   - anyhow for application errors
   - thiserror for library errors
   - sqlx for async database
   - validator for URL validation

5. Database integration:
   - Use sqlx::query! macro for type-safe queries
   - Store tabs in database for session restore
   - Use async/await with tokio
   - Handle connection pool properly

6. URL Validation:
   - Use validator crate to validate URLs
   - Prevent invalid URLs from being set
   - Sanitize user input

7. Create wrapper types:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(String);

impl TabId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyId(String);
```

8. Error Handling:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TabError {
    #[error("Tab not found: {0}")]
    TabNotFound(String),
    
    #[error("Maximum tabs exceeded: {0}")]
    MaxTabsExceeded(usize),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}
```

9. Database Schema (for reference):
```sql
CREATE TABLE IF NOT EXISTS tabs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    favicon TEXT,
    proxy_id TEXT,
    virtual_ip TEXT,
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL,
    is_pinned BOOLEAN DEFAULT 0
);
```

10. Include comprehensive inline documentation
11. Make all necessary types serializable with serde
12. Use proper Rust idioms and async/await patterns
13. Add validation for all user inputs using validator crate

Please provide the complete implementation with:
- All necessary imports
- Type definitions
- TabManager implementation with async methods
- Error types using thiserror
- Database integration with sqlx
- Input validation with validator
- Comprehensive documentation
```

---

### 1.2 Database Layer (NEW - Using sqlx)

**File**: `crates/browser-core/src/database.rs`

**Claude Opus 4.5 Prompt:**
```
Create a database layer using sqlx for async SQLite operations.

REQUIREMENTS:

1. Create a Database struct that manages the connection pool:
```rust
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &str) -> Result<Self>
    pub async fn run_migrations(&self) -> Result<()>
    pub fn pool(&self) -> &SqlitePool
}
```

2. Implement initialization:
   - Check if database exists
   - Create if needed using sqlx::Sqlite::create_database()
   - Connect with SqlitePool::connect()
   - Run migrations using sqlx::migrate!()
   - Configure connection pool (max connections, idle timeout)

3. Add methods for common operations:
   - async fn execute_query(&self, query: &str) -> Result<()>
   - async fn transaction<F, T>(&self, f: F) -> Result<T> where F: FnOnce(&mut Transaction) -> Result<T>
   - async fn health_check(&self) -> Result<()>

4. Use compile-time verified queries:
```rust
// This will be checked at compile time!
pub async fn insert_tab(&self, tab: &Tab) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO tabs (id, title, url, created_at)
        VALUES (?, ?, ?, ?)
        "#,
        tab.id,
        tab.title,
        tab.url,
        tab.created_at
    )
    .execute(&self.pool)
    .await?;
    
    Ok(())
}
```

5. Error handling:
```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed")]
    ConnectionFailed(#[from] sqlx::Error),
    
    #[error("Migration failed")]
    MigrationFailed,
    
    #[error("Transaction failed")]
    TransactionFailed,
}
```

6. Configuration:
   - Max connections: 5
   - Idle timeout: 10 minutes
   - Connection timeout: 5 seconds
   - Enable WAL mode for better concurrency

7. Migration setup:
   - Create migrations/ directory
   - Use sqlx::migrate!() macro
   - Support forward and rollback

Please implement with proper async/await, connection pooling, and type-safe queries.
```

---

### 1.3 HTTP Client with Middleware (NEW)

**File**: `crates/browser-core/src/http_client.rs`

**Claude Opus 4.5 Prompt:**
```
Create an HTTP client with automatic retry logic and circuit breaker using reqwest-middleware.

REQUIREMENTS:

1. Create HttpClient struct:
```rust
pub struct HttpClient {
    client: ClientWithMiddleware,
    rate_limiter: Arc<RateLimiter<DirectRateLimiter>>,
}
```

2. Build client with middleware:
```rust
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

impl HttpClient {
    pub fn new() -> Self {
        // Configure retry policy
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(30))
            .build_with_max_retries(3);
        
        // Build base client
        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .user_agent("VirtualIPBrowser/1.0")
            .build()
            .expect("Failed to build HTTP client");
        
        // Add middleware
        let client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        
        // Rate limiter (10 requests per second)
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(nonzero!(10u32))
        ));
        
        Self { client, rate_limiter }
    }
}
```

3. Implement methods with proxy support:
   - async fn get(&self, url: &str) -> Result<Response>
   - async fn get_with_proxy(&self, url: &str, proxy: &ProxyConfig) -> Result<Response>
   - async fn post(&self, url: &str, body: String) -> Result<Response>
   - async fn download(&self, url: &str, path: &Path) -> Result<()>

4. Add rate limiting:
```rust
pub async fn get(&self, url: &str) -> Result<Response> {
    // Wait for rate limit
    self.rate_limiter.until_ready().await;
    
    // Make request (automatically retries on failure)
    let response = self.client
        .get(url)
        .send()
        .await?;
    
    Ok(response)
}
```

5. Proxy configuration:
```rust
pub async fn get_with_proxy(&self, url: &str, proxy: &ProxyConfig) -> Result<Response> {
    self.rate_limiter.until_ready().await;
    
    let proxy_url = format!("{}://{}:{}", 
        proxy.scheme(), proxy.host, proxy.port);
    
    let mut req_proxy = reqwest::Proxy::all(&proxy_url)?;
    
    if let (Some(user), Some(pass)) = (&proxy.username, &proxy.password) {
        req_proxy = req_proxy.basic_auth(user, pass);
    }
    
    // Create temporary client with this proxy
    let client = reqwest::Client::builder()
        .proxy(req_proxy)
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let response = client.get(url).send().await?;
    Ok(response)
}
```

6. Error handling:
```rust
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Proxy connection failed")]
    ProxyFailed(#[from] reqwest::Error),
    
    #[error("Timeout")]
    Timeout,
}
```

7. Features:
   - Automatic retry on transient errors (network issues, 5xx errors)
   - Exponential backoff between retries
   - Circuit breaker pattern
   - Rate limiting
   - Proxy support with authentication
   - Connection pooling
   - Request/response logging

Please implement with comprehensive error handling and async support.
```

---

### 1.4 Web Scraper for Free Proxies (NEW)

**File**: `crates/browser-core/src/scraper_util.rs`

**Claude Opus 4.5 Prompt:**
```
Create a web scraper utility using the scraper crate for parsing HTML from free proxy provider websites.

REQUIREMENTS:

1. Create ProxyScraper struct:
```rust
pub struct ProxyScraper {
    http_client: HttpClient,
}

impl ProxyScraper {
    pub fn new(http_client: HttpClient) -> Self
    
    pub async fn scrape_free_proxy_list(&self) -> Result<Vec<ProxyConfig>>
    pub async fn scrape_proxy_nova(&self) -> Result<Vec<ProxyConfig>>
    pub async fn scrape_spys_one(&self) -> Result<Vec<ProxyConfig>>
}
```

2. Implement Free Proxy List scraper:
```rust
pub async fn scrape_free_proxy_list(&self) -> Result<Vec<ProxyConfig>> {
    use scraper::{Html, Selector};
    
    // Fetch HTML
    let html = self.http_client
        .get("https://free-proxy-list.net/")
        .await?
        .text()
        .await?;
    
    // Parse HTML
    let document = Html::parse_document(&html);
    let table_selector = Selector::parse("table.table tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    
    let mut proxies = Vec::new();
    
    for row in document.select(&table_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        
        if cells.len() >= 7 {
            let ip = cells[0].text().collect::<String>().trim().to_string();
            let port = cells[1].text().collect::<String>().trim();
            let country_code = cells[2].text().collect::<String>().trim().to_string();
            let anonymity = cells[4].text().collect::<String>().trim();
            let https = cells[6].text().collect::<String>().trim();
            
            if let Ok(port) = port.parse::<u16>() {
                proxies.push(ProxyConfig {
                    id: ProxyId::new(),
                    proxy_type: if https == "yes" { 
                        ProxyType::HTTPS 
                    } else { 
                        ProxyType::HTTP 
                    },
                    host: ip,
                    port,
                    username: None,
                    password: None,
                    enabled: true,
                    countries: vec![country_code],
                    speed_mbps: None,
                    uptime_percent: None,
                    last_checked: None,
                });
            }
        }
    }
    
    Ok(proxies)
}
```

3. Add error handling:
```rust
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Failed to fetch page")]
    FetchFailed(#[from] HttpError),
    
    #[error("Failed to parse HTML")]
    ParseFailed,
    
    #[error("No proxies found")]
    NoProxiesFound,
}
```

4. Features:
   - Respect robots.txt
   - Add User-Agent header
   - Rate limiting (1 request per minute)
   - Handle parsing errors gracefully
   - Validate parsed data
   - Cache results
   - Handle different HTML structures per provider

5. Best practices:
   - Use CSS selectors (like jQuery)
   - Handle missing elements gracefully
   - Validate all extracted data
   - Log scraping operations
   - Implement retry on HTML parsing failures

Please implement with proper error handling and respect for websites.
```

---

## 📋 Phase 2: Svelte 5 UI Components (UPDATED)

### 2.1 Main App with Svelte 5 Runes (UPDATED)

**File**: `ui-tauri/src/App.svelte`

**Claude Opus 4.5 Prompt:**
```
Create the main browser application UI using Svelte 5 with the new Runes API.

REQUIREMENTS:

1. Use Svelte 5 Runes for state management:
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  import TabBar from './components/TabBar.svelte';
  import AddressBar from './components/AddressBar.svelte';
  import NavigationBar from './components/NavigationBar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  
  // Svelte 5 Runes API - Reactive state
  let tabs = $state<Tab[]>([]);
  let activeTabId = $state<string | null>(null);
  let currentUrl = $state<string>('');
  let isLoading = $state<boolean>(false);
  let proxyStatus = $state<ProxyStatus>({
    connected: false,
    country: '',
    ip: '',
    downloadSpeed: 0,
    uploadSpeed: 0,
    latency: 0
  });
  
  // Derived state using $derived
  let activeTab = $derived(
    tabs.find(tab => tab.id === activeTabId)
  );
  
  let canGoBack = $derived(
    activeTab?.canGoBack ?? false
  );
  
  let canGoForward = $derived(
    activeTab?.canGoForward ?? false
  );
  
  // Effects using $effect
  $effect(() => {
    // Load tabs on mount
    loadTabs();
    
    // Listen for tab updates
    const unlisten = listen('tab-updated', (event) => {
      updateTab(event.payload);
    });
    
    return () => {
      unlisten.then(fn => fn());
    };
  });
  
  async function loadTabs() {
    try {
      tabs = await invoke<Tab[]>('get_all_tabs');
      if (tabs.length > 0) {
        activeTabId = tabs[0].id;
      }
    } catch (error) {
      console.error('Failed to load tabs:', error);
    }
  }
  
  async function createNewTab() {
    try {
      const tabId = await invoke<string>('create_tab', { 
        url: 'about:blank' 
      });
      
      await loadTabs();
      activeTabId = tabId;
    } catch (error) {
      console.error('Failed to create tab:', error);
    }
  }
  
  async function closeTab(tabId: string) {
    try {
      await invoke('close_tab', { tabId });
      await loadTabs();
      
      if (tabs.length > 0) {
        activeTabId = tabs[0].id;
      }
    } catch (error) {
      console.error('Failed to close tab:', error);
    }
  }
  
  async function navigateToUrl(url: string) {
    if (!activeTabId) return;
    
    try {
      isLoading = true;
      await invoke('navigate', { 
        tabId: activeTabId, 
        url 
      });
    } catch (error) {
      console.error('Navigation failed:', error);
    } finally {
      isLoading = false;
    }
  }
  
  async function goBack() {
    if (!activeTabId) return;
    await invoke('go_back', { tabId: activeTabId });
  }
  
  async function goForward() {
    if (!activeTabId) return;
    await invoke('go_forward', { tabId: activeTabId });
  }
  
  async function reload() {
    if (!activeTabId) return;
    await invoke('reload', { tabId: activeTabId });
  }
  
  function updateTab(updatedTab: Tab) {
    const index = tabs.findIndex(t => t.id === updatedTab.id);
    if (index !== -1) {
      tabs[index] = updatedTab;
    }
  }
</script>

<div class="browser-window">
  <TabBar 
    {tabs} 
    {activeTabId} 
    onnewTab={createNewTab}
    oncloseTab={closeTab}
    ontabChange={(tabId) => activeTabId = tabId}
  />
  
  <div class="navigation-area">
    <NavigationBar 
      {canGoBack}
      {canGoForward}
      {isLoading}
      onback={goBack}
      onforward={goForward}
      onreload={reload}
    />
    
    <AddressBar 
      url={currentUrl}
      {isLoading}
      onnavigate={navigateToUrl}
    />
  </div>
  
  <div class="webview-container">
    <!-- WebView managed by Tauri backend -->
  </div>
  
  <StatusBar {proxyStatus} />
</div>

<style>
  .browser-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  
  .navigation-area {
    display: flex;
    gap: 8px;
    padding: 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .webview-container {
    flex: 1;
    position: relative;
  }
</style>
```

2. Key changes from Svelte 4:
   - Use `$state` instead of `writable` stores
   - Use `$derived` for computed values
   - Use `$effect` instead of `onMount` + reactive statements
   - Event handlers: `onnewTab` instead of `on:newTab`
   - Simpler, more intuitive reactivity

3. Type definitions (create types.ts):
```typescript
export interface Tab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  proxyId?: string;
  virtualIp?: string;
}

export interface ProxyStatus {
  connected: boolean;
  country: string;
  ip: string;
  downloadSpeed: number;
  uploadSpeed: number;
  latency: number;
}
```

4. Tauri 2.0 API changes:
   - Import from '@tauri-apps/api/core' (not 'tauri')
   - Use invoke() from core module
   - Listen from '@tauri-apps/api/event'

Please implement with Svelte 5 runes, TypeScript, and Tauri 2.0 API.
```

---

### 2.2 TabBar Component with Svelte 5 (UPDATED)

**File**: `ui-tauri/src/components/TabBar.svelte`

**Claude Opus 4.5 Prompt:**
```
Create a tab bar component using Svelte 5 Runes API.

REQUIREMENTS:

1. Use Svelte 5 features:
```svelte
<script lang="ts">
  import type { Tab } from '../lib/types';
  
  // Props using Svelte 5 syntax
  let {
    tabs = $bindable<Tab[]>([]),
    activeTabId = $bindable<string | null>(null),
    onnewTab,
    oncloseTab,
    ontabChange
  }: {
    tabs: Tab[];
    activeTabId: string | null;
    onnewTab: () => void;
    oncloseTab: (tabId: string) => void;
    ontabChange: (tabId: string) => void;
  } = $props();
  
  // Local state
  let draggedTab = $state<Tab | null>(null);
  let dropTargetIndex = $state<number | null>(null);
  
  // Derived state
  let pinnedTabs = $derived(
    tabs.filter(tab => tab.isPinned)
  );
  
  let unpinnedTabs = $derived(
    tabs.filter(tab => !tab.isPinned)
  );
  
  function handleTabClick(tab: Tab) {
    ontabChange(tab.id);
  }
  
  function handleCloseTab(tab: Tab, event: MouseEvent) {
    event.stopPropagation();
    oncloseTab(tab.id);
  }
  
  function handleDragStart(tab: Tab, event: DragEvent) {
    draggedTab = tab;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }
  
  function handleDragOver(index: number, event: DragEvent) {
    event.preventDefault();
    dropTargetIndex = index;
  }
  
  function handleDrop(index: number) {
    if (draggedTab) {
      // Emit reorder event
      // ontabReorder(draggedTab.id, index);
    }
    draggedTab = null;
    dropTargetIndex = null;
  }
</script>

<div class="tab-bar">
  <div class="tabs-container">
    {#each tabs as tab, index (tab.id)}
      <button
        class="tab"
        class:active={tab.id === activeTabId}
        class:pinned={tab.isPinned}
        class:loading={tab.loading}
        draggable="true"
        ondragstart={(e) => handleDragStart(tab, e)}
        ondragover={(e) => handleDragOver(index, e)}
        ondrop={() => handleDrop(index)}
        onclick={() => handleTabClick(tab)}
      >
        {#if tab.favicon}
          <img src={tab.favicon} alt="" class="tab-favicon" />
        {:else}
          <span class="tab-icon">🌐</span>
        {/if}
        
        <span class="tab-title">{tab.title || 'New Tab'}</span>
        
        {#if tab.loading}
          <span class="tab-spinner">⟳</span>
        {/if}
        
        {#if !tab.isPinned}
          <button
            class="tab-close"
            onclick={(e) => handleCloseTab(tab, e)}
            title="Close tab"
          >
            ×
          </button>
        {/if}
      </button>
    {/each}
  </div>
  
  <button class="new-tab-button" onclick={onnewTab} title="New tab">
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
    background: none;
    border: none;
    cursor: pointer;
    font-size: 18px;
  }
  
  .tab:hover .tab-close {
    opacity: 1;
  }
  
  .new-tab-button {
    margin-left: 8px;
    width: 32px;
    height: 32px;
    border-radius: 4px;
    background: var(--button-bg);
    border: none;
    cursor: pointer;
    font-size: 20px;
  }
</style>
```

2. Key Svelte 5 changes:
   - `let { props } = $props()` instead of `export let props`
   - `$bindable` for two-way binding
   - `$state` for local reactive state
   - `$derived` for computed values
   - Event handlers: `onclick` instead of `on:click`

Please implement with Svelte 5 syntax and TypeScript types.
```

---

## 🔄 Continue in Next Document...

This document is getting long. Should I:

1. **Continue with more updated prompts** (Phases 3-8)
2. **Create a separate document** for each phase with updated prompts
3. **Create a migration guide** showing old prompt vs new prompt side-by-side
4. **Something else**?

Which would be most helpful?

