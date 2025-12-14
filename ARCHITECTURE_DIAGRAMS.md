# Architecture Diagrams & Visual Guides
## Visual Overview of Virtual IP Browser Architecture

---

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         VIRTUAL IP BROWSER                               │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    FRONTEND LAYER (Svelte)                       │   │
│  │                                                                   │   │
│  │  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌──────────┐      │   │
│  │  │ Tab Bar  │  │ Address   │  │Navigation│  │ Status   │      │   │
│  │  │Component │  │   Bar     │  │   Bar    │  │   Bar    │      │   │
│  │  └─────┬────┘  └─────┬─────┘  └────┬─────┘  └────┬─────┘      │   │
│  │        │             │              │             │              │   │
│  │        └─────────────┴──────────────┴─────────────┘              │   │
│  │                          │                                        │   │
│  │                    [Tauri IPC]                                   │   │
│  └────────────────────────┼──────────────────────────────────────────┘   │
│                            │                                              │
│  ┌────────────────────────▼──────────────────────────────────────────┐   │
│  │                  BACKEND LAYER (Rust)                              │   │
│  │                                                                     │   │
│  │  ┌──────────────────────────────────────────────────────────┐    │   │
│  │  │              BROWSER CORE MODULE                          │    │   │
│  │  │                                                            │    │   │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │    │   │
│  │  │  │ Tab Manager  │  │   WebView    │  │   Browser    │  │    │   │
│  │  │  │              │  │   Manager    │  │  Controls    │  │    │   │
│  │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │    │   │
│  │  │         │                  │                  │           │    │   │
│  │  │         └──────────────────┴──────────────────┘           │    │   │
│  │  │                            │                               │    │   │
│  │  └────────────────────────────┼───────────────────────────────┘    │   │
│  │                               │                                     │   │
│  │  ┌────────────────────────────▼───────────────────────────────┐    │   │
│  │  │              NETWORK LAYER                                  │    │   │
│  │  │                                                              │    │   │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │    │   │
│  │  │  │    HTTP      │  │    Proxy     │  │   Request    │    │    │   │
│  │  │  │ Interceptor  │  │   Manager    │  │   Filter     │    │    │   │
│  │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │    │   │
│  │  │         │                  │                  │             │    │   │
│  │  │         └──────────────────┴──────────────────┘             │    │   │
│  │  │                            │                                 │    │   │
│  │  └────────────────────────────┼─────────────────────────────────┘    │   │
│  │                               │                                       │   │
│  │  ┌────────────────────────────▼─────────────────────────────────┐    │   │
│  │  │           VIRTUAL IP MODULE                                   │    │   │
│  │  │                                                                │    │   │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │    │   │
│  │  │  │    Proxy     │  │   Provider   │  │   Rotation   │      │    │   │
│  │  │  │  Validator   │  │   Manager    │  │   Strategy   │      │    │   │
│  │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │    │   │
│  │  │         │                  │                  │               │    │   │
│  │  │         └──────────────────┴──────────────────┘               │    │   │
│  │  │                            │                                   │    │   │
│  │  └────────────────────────────┼───────────────────────────────────┘    │   │
│  │                               │                                         │   │
│  │  ┌────────────────────────────▼─────────────────────────────────┐      │   │
│  │  │              STORAGE LAYER                                    │      │   │
│  │  │                                                                │      │   │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │      │   │
│  │  │  │   Proxy DB   │  │  Bookmarks   │  │   History    │      │      │   │
│  │  │  │   (SQLite)   │  │     DB       │  │     DB       │      │      │   │
│  │  │  └──────────────┘  └──────────────┘  └──────────────┘      │      │   │
│  │  │                                                               │      │   │
│  │  └───────────────────────────────────────────────────────────────┘      │   │
│  │                                                                           │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
└───────────────────────────┬───────────────────────────────────────────────┘
                            │
                            ▼
            ┌───────────────────────────────────┐
            │    FREE PROXY PROVIDERS (APIs)    │
            │                                    │
            │  • ProxyScrape  • PubProxy        │
            │  • FreeProxyList • Geonode        │
            │  • ProxyNova    • Spys.one        │
            └───────────────┬───────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │   INTERNET   │
                    └──────────────┘
```

---

## Request Flow Diagram

```
User Action (Click Link)
        │
        ▼
┌───────────────────┐
│  Address Bar UI   │ (Svelte Component)
└────────┬──────────┘
         │ emit('navigate')
         ▼
┌─────────────────────┐
│   Tauri IPC Call    │ invoke('navigate', {tab_id, url})
└────────┬────────────┘
         │
         ▼
┌──────────────────────┐
│  Browser Controls    │ (Rust Backend)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   Tab Manager        │ get_tab(tab_id)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   WebView Manager    │ navigate(webview, url)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│  HTTP Interceptor    │ intercept_request(url)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   Proxy Manager      │ get_active_proxy()
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   Proxy Rotator      │ should_rotate() ?
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│  reqwest HTTP Client │ send_with_proxy(request, proxy)
└────────┬─────────────┘
         │
         ▼
      INTERNET
         │
         ▼ (Response)
┌──────────────────────┐
│  Response Filter     │ modify_headers(response)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   WebView Renderer   │ display(content)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   User sees page     │
└──────────────────────┘
```

---

## Tab Isolation Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        BROWSER WINDOW                        │
│                                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │  Tab 1   │  │  Tab 2   │  │  Tab 3   │                  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                  │
│       │             │             │                          │
└───────┼─────────────┼─────────────┼──────────────────────────┘
        │             │             │
        ▼             ▼             ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│ Isolation  │ │ Isolation  │ │ Isolation  │
│ Context 1  │ │ Context 2  │ │ Context 3  │
│            │ │            │ │            │
│ • Cookies  │ │ • Cookies  │ │ • Cookies  │
│ • Storage  │ │ • Storage  │ │ • Storage  │
│ • Cache    │ │ • Cache    │ │ • Cache    │
│ • Sessions │ │ • Sessions │ │ • Sessions │
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘
      │              │              │
      ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│  Proxy 1   │ │  Proxy 2   │ │  Proxy 3   │
│  (US)      │ │  (UK)      │ │  (DE)      │
│  1.2.3.4   │ │  5.6.7.8   │ │  9.10.11.12│
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘
      │              │              │
      └──────────────┴──────────────┘
                     │
                     ▼
              ┌─────────────┐
              │  INTERNET   │
              └─────────────┘

Each tab is completely isolated from others!
```

---

## Proxy Provider Workflow

```
┌─────────────────────────────────────────────────────────────┐
│              PROXY PROVIDER MANAGER                          │
└────────────────┬────────────────────────────────────────────┘
                 │
    ┌────────────┴────────────┬────────────┬────────────┐
    │                         │            │            │
    ▼                         ▼            ▼            ▼
┌──────────┐           ┌──────────┐  ┌──────────┐  ┌──────────┐
│ProxyScrape│          │ PubProxy │  │FreeProxy│  │  Geonode │
│  Provider │          │ Provider │  │  List   │  │ Provider │
└─────┬────┘           └─────┬────┘  └────┬─────┘  └────┬─────┘
      │                      │            │             │
      │ fetch_proxies()      │            │             │
      │                      │            │             │
      ▼                      ▼            ▼             ▼
   [API Call]             [API Call]  [Web Scrape]  [API Call]
      │                      │            │             │
      │                      │            │             │
      ▼                      ▼            ▼             ▼
  [JSON]                   [JSON]      [HTML]         [JSON]
      │                      │            │             │
      └──────────────────────┴────────────┴─────────────┘
                             │
                             ▼
                    ┌────────────────┐
                    │  Deduplicate   │
                    │   (Remove      │
                    │  Duplicates)   │
                    └────────┬───────┘
                             │
                             ▼
                    ┌────────────────┐
                    │    Validate    │
                    │  (Test each    │
                    │    proxy)      │
                    └────────┬───────┘
                             │
                   ┌─────────┴─────────┐
                   │                   │
              Working              Dead
                   │                   │
                   ▼                   ▼
          ┌────────────────┐   ┌────────────┐
          │  Add to Pool   │   │  Discard   │
          └────────┬───────┘   └────────────┘
                   │
                   ▼
          ┌────────────────┐
          │  Save to DB    │
          └────────────────┘
```

---

## Proxy Rotation Strategies

### 1. Time-Based Rotation
```
Time:     0min    5min    10min   15min   20min
          │       │       │       │       │
Proxy:    [US]────[UK]────[DE]────[FR]────[CA]
          
Switch proxy every N minutes
```

### 2. Request-Based Rotation
```
Requests: 0       50      100     150     200
          │       │       │       │       │
Proxy:    [US]────[UK]────[DE]────[FR]────[CA]
          
Switch proxy every N requests
```

### 3. Domain-Based Rotation
```
google.com  ──→  [US Proxy]
amazon.com  ──→  [UK Proxy]
github.com  ──→  [DE Proxy]
reddit.com  ──→  [FR Proxy]

Each domain gets its own proxy (sticky session)
```

### 4. Geographic Rotation
```
Round 1: US Proxies [1.2.3.4, 5.6.7.8]
         ↓
Round 2: UK Proxies [9.10.11.12, 13.14.15.16]
         ↓
Round 3: DE Proxies [17.18.19.20, 21.22.23.24]
         ↓
Repeat

Rotate through specific countries in order
```

---

## Data Flow: User Creates New Tab

```
┌──────────────┐
│ User clicks  │
│  "New Tab"   │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  TabBar.svelte   │  emit('newTab')
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  App.svelte      │  invoke('create_tab', {url})
└──────┬───────────┘
       │ [IPC]
       ▼
┌──────────────────┐
│  Tauri Command   │  #[tauri::command]
│  create_tab()    │  async fn create_tab()
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  TabManager      │  let tab_id = self.create_tab(url)?
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Generate TabId  │  TabId::new() -> uuid
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Create Tab      │  Tab { id, url, title, ... }
│  Struct          │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Assign Proxy    │  get_next_proxy() -> proxy_id
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Create          │  Isolation context for tab
│  Isolation       │
│  Context         │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  WebViewManager  │  create_webview(tab_id, url, proxy)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Tauri WebView   │  WebView::new(config)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Return TabId    │  Ok(tab_id)
└──────┬───────────┘
       │ [IPC Response]
       ▼
┌──────────────────┐
│  Update UI       │  tabs.push(new_tab)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Render new tab  │  User sees new tab in UI
└──────────────────┘
```

---

## Database Schema Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      SQLITE DATABASE                         │
└─────────────────────────────────────────────────────────────┘

┌────────────────────┐
│      proxies       │
├────────────────────┤
│ id (PK)            │◄─────┐
│ proxy_type         │      │
│ host               │      │
│ port               │      │
│ username           │      │
│ password           │      │
│ country            │      │
│ anonymity_level    │      │
│ source_provider    │      │
│ is_active          │      │
│ created_at         │      │
│ last_validated     │      │
└────────────────────┘      │
                            │
┌────────────────────┐      │
│  proxy_metrics     │      │
├────────────────────┤      │
│ id (PK)            │      │
│ proxy_id (FK)      │──────┘
│ response_time_ms   │
│ success            │
│ error_message      │
│ checked_at         │
└────────────────────┘

┌────────────────────┐
│  proxy_sessions    │
├────────────────────┤
│ id (PK)            │
│ proxy_id (FK)      │──────┐
│ tab_id             │      │
│ domain             │      │
│ request_count      │      │
│ bytes_transferred  │      │
│ started_at         │      │
│ ended_at           │      │
└────────────────────┘      │
                            │
┌────────────────────┐      │
│    bookmarks       │      │
├────────────────────┤      │
│ id (PK)            │      │
│ title              │      │
│ url                │      │
│ favicon            │      │
│ folder_id (FK)     │──┐   │
│ tags               │  │   │
│ created_at         │  │   │
│ modified_at        │  │   │
└────────────────────┘  │   │
                        │   │
┌────────────────────┐  │   │
│ bookmark_folders   │  │   │
├────────────────────┤  │   │
│ id (PK)            │◄─┘   │
│ name               │      │
│ parent_id          │      │
│ position           │      │
└────────────────────┘      │
                            │
┌────────────────────┐      │
│     history        │      │
├────────────────────┤      │
│ id (PK)            │      │
│ url                │      │
│ title              │      │
│ visit_count        │      │
│ last_visit         │      │
│ first_visit        │      │
└────────────────────┘      │
                            │
┌────────────────────┐      │
│     settings       │      │
├────────────────────┤      │
│ key (PK)           │      │
│ value              │      │
│ updated_at         │      │
└────────────────────┘      │
```

---

## File Structure Diagram

```
virtual-ip-browser/
│
├── Cargo.toml                    (Workspace configuration)
├── Cargo.lock
│
├── crates/
│   ├── browser-core/             (Main browser logic)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tab_manager.rs    ◄── Tab management
│   │   │   ├── webview_manager.rs ◄── WebView management
│   │   │   ├── browser_controls.rs ◄── Navigation controls
│   │   │   ├── proxy.rs          ◄── Proxy connection
│   │   │   ├── http_client.rs    ◄── HTTP interception
│   │   │   ├── tab_isolation.rs  ◄── Cookie/storage isolation
│   │   │   ├── fingerprint.rs    ◄── Fingerprinting protection
│   │   │   ├── storage.rs        ◄── Database layer
│   │   │   ├── free_ip_providers.rs ◄── Provider abstraction
│   │   │   └── download_manager.rs ◄── Downloads
│   │   └── tests/
│   │       └── tab_lifecycle.rs
│   │
│   ├── virtual-ip/               (Proxy & IP management)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs         ◄── Data structures
│   │   │   ├── generator.rs      ◄── IP generation
│   │   │   ├── rotation.rs       ◄── Proxy rotation
│   │   │   └── validator.rs      ◄── Proxy validation
│   │   └── tests/
│   │
│   └── api-server/               (Optional REST API)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   └── main.rs
│       └── tests/
│
├── ui-tauri/                     (Tauri frontend)
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   │
│   ├── src/                      (Svelte UI)
│   │   ├── App.svelte            ◄── Main application
│   │   ├── main.ts
│   │   ├── app.css
│   │   │
│   │   ├── components/
│   │   │   ├── TabBar.svelte     ◄── Tab bar
│   │   │   ├── AddressBar.svelte ◄── Address bar
│   │   │   ├── NavigationBar.svelte ◄── Navigation
│   │   │   ├── StatusBar.svelte  ◄── Status bar
│   │   │   ├── SettingsPanel.svelte ◄── Settings
│   │   │   └── UpdateNotification.svelte
│   │   │
│   │   └── lib/
│   │       ├── types.ts          ◄── TypeScript types
│   │       └── api.ts            ◄── Tauri IPC wrapper
│   │
│   └── src-tauri/                (Tauri backend)
│       ├── Cargo.toml
│       ├── tauri.conf.json       ◄── Tauri configuration
│       ├── build.rs
│       └── src/
│           └── main.rs           ◄── Tauri entry point
│
└── [PLAN DOCUMENTS]/             (These documentation files)
    ├── MASTER_INDEX.md
    ├── GETTING_STARTED.md
    ├── QUICKSTART_GUIDE.md
    ├── DEVELOPMENT_PHASES.md
    ├── PHASE_5_UI_COMPONENTS.md
    ├── PHASE_6_ADVANCED_FEATURES.md
    ├── PHASE_7_TESTING_SECURITY.md
    ├── PHASE_8_DEPLOYMENT.md
    ├── PROXY_PROVIDERS_DETAILED.md
    ├── WINDSURF_USAGE_GUIDE.md
    ├── IMPLEMENTATION_CHECKLIST.md
    └── ARCHITECTURE_DIAGRAMS.md  ◄── This file
```

---

## Component Dependency Graph

```
                    ┌───────────────┐
                    │  App.svelte   │ (Top level)
                    └───────┬───────┘
                            │
            ┌───────────────┼───────────────┐
            │               │               │
            ▼               ▼               ▼
    ┌──────────┐    ┌─────────────┐  ┌──────────┐
    │ TabBar   │    │ AddressBar  │  │StatusBar │
    └─────┬────┘    └──────┬──────┘  └─────┬────┘
          │                │               │
          └────────────────┴───────────────┘
                           │
                           ▼
                  ┌────────────────┐
                  │  Tauri Backend │
                  └────────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
   ┌──────────┐    ┌──────────────┐  ┌──────────┐
   │   Tab    │    │   WebView    │  │  Proxy   │
   │ Manager  │    │   Manager    │  │ Manager  │
   └────┬─────┘    └──────┬───────┘  └────┬─────┘
        │                 │                │
        │                 │                │
        │                 ▼                │
        │          ┌──────────────┐        │
        │          │    HTTP      │        │
        └─────────►│ Interceptor  │◄───────┘
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │   Provider   │
                   │   Manager    │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │   Database   │
                   └──────────────┘
```

---

## State Management Flow

```
┌─────────────────────────────────────────────────────────┐
│                   APPLICATION STATE                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────┐  ┌────────────────┐               │
│  │  Browser State │  │   Proxy State  │               │
│  │                │  │                │               │
│  │  • tabs[]      │  │  • active_proxy│               │
│  │  • active_tab  │  │  • proxy_pool  │               │
│  │  • history     │  │  • rotation    │               │
│  │  • bookmarks   │  │  • statistics  │               │
│  └────────┬───────┘  └────────┬───────┘               │
│           │                   │                        │
│           └───────────┬───────┘                        │
│                       │                                 │
└───────────────────────┼─────────────────────────────────┘
                        │
                ┌───────┴───────┐
                │               │
                ▼               ▼
        ┌─────────────┐  ┌─────────────┐
        │  Persist to │  │  Sync to UI │
        │  Database   │  │  (IPC Events)│
        └─────────────┘  └─────────────┘
```

---

These diagrams provide a visual understanding of the system architecture. Use them as reference when implementing components!

