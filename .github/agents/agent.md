---
name: deep pro expert experience programmer, developer
description: Pro advances most expert experience developer, programmer.
---

# GitHub Copilot Coding Agent - Virtual IP Browser Project

## 🎯 Project Overview

**Desktop Browser Application** with proxy and virtual IP management:
- Multi-tab browser with Chromium engine
- Per-tab proxy configuration and rotation
- Virtual IP generation and management
- Browser fingerprint protection
- Cookie/storage isolation per tab
- Free proxy provider integration

### Tech Stack
- **Backend**: Rust + Tauri 2.0
- **Frontend**: Svelte + TypeScript + Vite
- **Database**: SQLite (via sqlx)
- **Browser Engine**: Chromium (chromiumoxide)
- **Async Runtime**: Tokio

### Project Structure
```
proxy-desktop-browser/
├── crates/
│   ├── browser-core/      # Core browser logic
│   ├── virtual-ip/        # IP generation & rotation
├── ui-tauri/              # Tauri desktop app
│   ├── src-tauri/         # Rust backend
│   └── src/               # Svelte frontend
```

---

## 🏗️ Architecture

### Core Components

**1. Browser Core (`crates/browser-core/`)**
- `chromium_engine.rs` - Browser engine integration
- `browser_tab_manager.rs` - Tab lifecycle management
- `proxy.rs` - Proxy connection handling
- `proxy_rotation.rs` - Automatic proxy switching
- `proxy_validator.rs` - Proxy health checking
- `fingerprint.rs` - Browser fingerprint generation
- `tab_isolation.rs` - Per-tab network/storage isolation
- `storage.rs` - Cookie, history, bookmark management
- `database.rs` - SQLite operations
- `free_ip_providers.rs` - Free proxy API integration

**2. Virtual IP (`crates/virtual-ip/`)** - IP generation, rotation strategies, validation

**3. Tauri Backend (`ui-tauri/src-tauri/`)** - Tauri commands, app init, auth, WebView lifecycle

**4. Svelte Frontend (`ui-tauri/src/`)** - Browser UI, settings, auth components

---

## 📅 Development Phases

### Status
**✅ COMPLETED**: Rust workspace, Tauri 2.0 foundation, Svelte UI, Database schema, Virtual IP models, Tab management framework, UI component structure

**🚧 IN PROGRESS**: Chromium engine integration, Proxy connection logic, Browser tab lifecycle, Free proxy providers

**❌ CRITICAL GAPS**: Network traffic interception, Actual browser rendering, WebView isolation, Proxy rotation engine, Cookie/storage isolation, PAC server, Local proxy server, Free proxy scraping, Fingerprint randomization, E2E testing

### Implementation Priorities

**PHASE 1: Core Browser Engine (CRITICAL)**
1. Complete Chromium Engine Integration (`chromium_engine.rs`)
   - Implement `ChromiumEngine::launch()` using chromiumoxide
   - Add navigation methods: `goto()`, `reload()`, `back()`, `forward()`
2. WebView Manager Enhancement (`webview_manager.rs` - TODO line 43)
   - Implement per-webview proxy configuration
   - Create isolated WebView instances per tab
3. Tab Lifecycle (`browser_tab_manager.rs` - TODO line 110)
   - Complete `create_tab()`, `close_tab()` with cleanup
4. Tauri Commands (`main.rs`)
   - Add: `navigate_tab`, `reload_tab`, `go_back`, `go_forward`

**Success**: Browser opens, navigates URLs, multiple tabs work, back/forward navigation, tabs create/close without crashes

**PHASE 2: Proxy & Virtual IP (CRITICAL)**
1. Local Proxy Server (`local_proxy.rs`) - SOCKS5 proxy using tokio
2. Proxy Connection (`proxy.rs`) - Complete `ProxyManager::connect()`, add SOCKS5/HTTP/HTTPS support
3. PAC Server (`pac_server.rs`) - HTTP server serving PAC files dynamically
4. Per-Tab Proxy Assignment - Integrate in `create_tab()`
5. Virtual IP Generator - Implement IP generation algorithms

**Success**: Traffic routes through proxy, each tab uses different proxy, IP verification works, no DNS/WebRTC leaks

**PHASE 3: Proxy Rotation & Health (HIGH)**
1. Proxy Health Checker (`proxy_validator.rs`) - Background service, test connectivity/latency
2. Proxy Rotation Engine (`proxy_rotation.rs`) - Round-robin, Random, Latency-based strategies
3. Proxy Pool Management - Health-based ranking, blacklist failed proxies
4. Tauri Commands - `rotate_proxy`, `get_proxy_health`, `add_proxy`, `remove_proxy`

**Success**: Auto-rotation works, failed proxies detected, health metrics visible, manual rotation works

**PHASE 4: Free Proxy Providers (HIGH)**
1. API Integration (`free_ip_providers.rs`) - ProxyScrape, GeoNode, ProxyList, PubProxy APIs
2. Scraper Enhancement - HTML scraping for proxy websites
3. Auto-Import Pipeline - Fetch on schedule, validate before adding, remove duplicates
4. Provider Commands - `fetch_free_proxies`, `get_provider_status`, `enable_provider`

**Success**: Free proxies fetched automatically, invalid filtered, pool populated, multiple providers supported

**PHASE 5: UI/UX (MEDIUM)** - Address bar, enhanced tab list, settings panel, proxy config UI, status bar

**PHASE 6: Advanced Features (MEDIUM)** - Cookie/storage isolation, fingerprint protection, downloads, bookmarks, sessions

**PHASE 7: Testing & Security (HIGH)**
- Unit tests (80%+ coverage target)
- Integration tests
- **Security**: Fix `auth.rs:127` - Implement proper password verification with hashing
- Performance testing (50+ tabs)
- Error recovery testing

**PHASE 8: Deployment (LOW)** - Build config, installers, auto-updates, documentation

---

## 🤖 Agent Execution Patterns

### Pattern 1: Single File Implementation
**When**: Adding feature to existing file

**Steps**:
1. Read file to understand current implementation
2. Identify insertion point (follow existing patterns)
3. Implement with proper error handling
4. Add imports and dependencies
5. Write tests
6. Verify with `cargo check`

**Example**: Adding Tauri command
```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(result)
}

// Register in main()
.invoke_handler(tauri::generate_handler![
    existing_commands,
    my_command  // Add here
])
```

### Pattern 2: Multi-File Feature
**When**: Feature spans multiple modules

**Steps**:
1. Plan - Identify all files needing changes
2. Backend First - Core logic in `crates/browser-core/`
3. Tauri Bridge - Commands in `ui-tauri/src-tauri/src/main.rs`
4. Type Definitions - Update `ui-tauri/src/lib/types.ts`
5. API Layer - Functions in `ui-tauri/src/lib/api.ts`
6. UI Component - Create/update Svelte components
7. Integration - Wire everything together
8. Testing - Test from UI to backend

### Pattern 3: Database Operations
**When**: Adding tables or queries

**Steps**:
1. Create Migration - `crates/browser-core/migrations/XXX_description.sql`
2. Write SQL - CREATE TABLE, indexes, constraints
3. Update Database Module - Add query functions to `database.rs`
4. Use sqlx Macros - `sqlx::query!` for type safety
5. Test - Verify migrations apply cleanly
6. Integrate - Use in business logic

**Example**:
```sql
-- migrations/005_proxy_sessions.sql
CREATE TABLE IF NOT EXISTS proxy_sessions (
    id TEXT PRIMARY KEY,
    tab_id TEXT NOT NULL,
    proxy_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    FOREIGN KEY (tab_id) REFERENCES tabs(id)
);
```

```rust
pub async fn create_proxy_session(&self, session: &ProxySession) -> Result<(), DatabaseError> {
    sqlx::query!("INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) VALUES (?, ?, ?, ?)",
        session.id, session.tab_id, session.proxy_id, session.started_at)
        .execute(&self.pool).await?;
    Ok(())
}
```

---

## 📝 Code Generation Guidelines

### Rust Backend

**Error Handling**:
```rust
// ❌ AVOID: Unwrapping
let result = some_operation().unwrap();

// ✅ PREFER: Returning Result
pub async fn my_function() -> Result<Data, MyError> {
    let result = some_operation()
        .map_err(|e| MyError::OperationFailed(e.to_string()))?;
    Ok(result)
}

// ✅ PREFER: Using anyhow
pub async fn my_function() -> anyhow::Result<Data> {
    let result = some_operation()
        .context("Failed to perform operation")?;
    Ok(result)
}
```

**Async/Await**:
```rust
// Async function with error handling
pub async fn fetch_proxies() -> Result<Vec<Proxy>, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/proxies")
        .await?
        .json::<Vec<Proxy>>()
        .await?;
    Ok(response)
}

// Spawning background tasks
tokio::spawn(async move {
    loop {
        if let Err(e) = health_check().await {
            eprintln!("Health check failed: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
});
```

**Database Operations**:
```rust
// Type-safe queries with compile-time verification
pub async fn get_proxy(&self, id: &str) -> Result<Option<Proxy>, DatabaseError> {
    let proxy = sqlx::query_as!(
        Proxy,
        "SELECT id, host, port, protocol FROM proxies WHERE id = ?", id
    )
    .fetch_optional(&self.pool)
    .await?;
    Ok(proxy)
}

// Transactions for multiple operations
pub async fn rotate_proxy(&self, tab_id: &str, new_proxy_id: &str) -> Result<(), DatabaseError> {
    let mut tx = self.pool.begin().await?;
    sqlx::query!("UPDATE proxy_sessions SET ended_at = ? WHERE tab_id = ? AND ended_at IS NULL", 
        chrono::Utc::now().timestamp(), tab_id)
        .execute(&mut *tx).await?;
    sqlx::query!("INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) VALUES (?, ?, ?, ?)",
        Uuid::new_v4().to_string(), tab_id, new_proxy_id, chrono::Utc::now().timestamp())
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
```

**Logging**:
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self), fields(tab_id = %tab_id))]
pub async fn create_tab(&self, tab_id: &str) -> Result<Tab, TabError> {
    info!("Creating new tab");
    let tab = Tab::new(tab_id);
    debug!("Assigning proxy to tab");
    if let Err(e) = self.assign_proxy(&tab).await {
        warn!("Failed to assign proxy: {}", e);
    }
    info!("Tab created successfully");
    Ok(tab)
}
```

### TypeScript/Svelte

**Type Safety**:
```typescript
// ui-tauri/src/lib/types.ts
export interface Tab {
    id: string;
    url: string;
    title: string;
    proxyId?: string;
    createdAt: number;
}

export interface ProxyStatus {
    connected: boolean;
    ipAddress?: string;
    latency?: number;
}

export type ProxyRotationStrategy = 'round-robin' | 'random' | 'latency-based';
```

**API Wrappers**:
```typescript
// ui-tauri/src/lib/api.ts
import { invoke } from '@tauri-apps/api/core';
import type { Tab, ProxyStatus, Proxy } from './types';

export async function createTab(url: string): Promise<Tab> {
    return await invoke<Tab>('create_tab', { url });
}

export async function rotateProxy(tabId: string): Promise<Proxy> {
    return await invoke<Proxy>('rotate_proxy', { tabId });
}
```

**Svelte Stores**:
```typescript
// ui-tauri/src/lib/stores.ts
import { writable, derived } from 'svelte/store';
import type { Tab, Proxy } from './types';

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);
export const activeTab = derived([tabs, activeTabId],
    ([$tabs, $activeTabId]) => $tabs.find(t => t.id === $activeTabId)
);
```

### Security Best Practices

**Input Validation**:
```rust
use validator::Validate;

#[derive(Validate)]
pub struct ProxyInput {
    #[validate(length(min = 7, max = 255))]
    host: String,
    #[validate(range(min = 1, max = 65535))]
    port: u16,
}
```

**SQL Injection Prevention**:
```rust
// ✅ SAFE: Parameterized query
sqlx::query!("SELECT * FROM tabs WHERE id = ?", tab_id).fetch_one(&pool).await?;

// ❌ DANGEROUS: String concatenation - NEVER DO THIS
// let query = format!("SELECT * FROM tabs WHERE id = '{}'", tab_id);
```

**Credential Storage**:
```rust
use keyring::Entry;

pub fn store_credentials(proxy_id: &str, username: &str, password: &str) -> Result<()> {
    let entry = Entry::new("virtual-ip-browser", proxy_id)?;
    entry.set_password(&format!("{}:{}", username, password))?;
    Ok(())
}
```

---

## 🧪 Testing Strategy

**Unit Testing**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_validation() {
        let validator = ProxyValidator::new();
        let proxy = Proxy {
            host: "127.0.0.1".to_string(),
            port: 8080,
            protocol: ProxyProtocol::Http,
        };
        let result = validator.validate(&proxy).await;
        assert!(result.is_ok());
    }
}
```

**Integration Testing**:
```rust
// crates/browser-core/tests/tab_lifecycle.rs
#[tokio::test]
async fn test_tab_with_proxy() {
    let db = Database::new(":memory:").await.unwrap();
    let proxy_manager = ProxyManager::new(db.clone());
    let tab_manager = BrowserTabManager::new(db.clone(), proxy_manager);
    
    let tab = tab_manager.create_tab("https://example.com").await.unwrap();
    assert!(tab.proxy_id.is_some());
    
    tab_manager.close_tab(&tab.id).await.unwrap();
}
```

---

## 🛠️ Common Tasks

### Task 1: Adding New Tauri Command
1. Implement Rust function in appropriate module
2. Create Tauri command in `ui-tauri/src-tauri/src/main.rs`
3. Register command in handler
4. Add TypeScript wrapper in `ui-tauri/src/lib/api.ts`
5. Update types in `ui-tauri/src/lib/types.ts`

**Example**:
```rust
#[tauri::command]
async fn get_active_proxy(tab_id: String, state: tauri::State<'_, AppState>) -> Result<Option<Proxy>, String> {
    state.proxy_manager.get_active_proxy(&tab_id).await.map_err(|e| e.to_string())
}

// Register
.invoke_handler(tauri::generate_handler![get_active_proxy])
```

```typescript
export async function getActiveProxy(tabId: string): Promise<Proxy | null> {
    return await invoke<Proxy | null>('get_active_proxy', { tabId });
}
```

### Task 2: Implementing Proxy Rotation
```rust
pub struct ProxyRotator {
    strategy: RotationStrategy,
    proxy_manager: Arc<ProxyManager>,
}

impl ProxyRotator {
    pub async fn rotate(&self, tab_id: &str) -> Result<Proxy> {
        let next_proxy = match self.strategy {
            RotationStrategy::RoundRobin => self.get_next_round_robin().await?,
            RotationStrategy::Random => self.get_random_proxy().await?,
            RotationStrategy::LatencyBased => self.get_fastest_proxy().await?,
        };
        self.proxy_manager.assign_proxy(tab_id, &next_proxy.id).await?;
        Ok(next_proxy)
    }
}
```

### Task 3: Integrating Free Proxy Provider
```rust
pub async fn fetch_from_proxyscrape() -> Result<Vec<Proxy>> {
    let url = "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http";
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    
    let proxies: Vec<Proxy> = text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                Some(Proxy {
                    id: Uuid::new_v4().to_string(),
                    host: parts[0].to_string(),
                    port: parts[1].parse().ok()?,
                    protocol: ProxyProtocol::Http,
                })
            } else { None }
        })
        .collect();
    Ok(proxies)
}
```

---

## 🔍 Troubleshooting

**Cargo Build Fails**:
```bash
cargo clean && cargo build
cargo update
rustup update
cd crates/browser-core && sqlx migrate run --database-url sqlite:test.db
```

**Tauri App Won't Start**:
- Check console logs
- Verify database initialization
- Test in dev mode: `cd ui-tauri && bun run tauri dev`

**Proxy Connection Fails**:
```bash
# Test proxy manually
curl -x http://proxy_host:port https://api.ipify.org
```
- Verify credentials
- Add debug logging
- Check firewall settings

---

## 📚 Reference

### Key Files
- `Cargo.toml` - Root workspace config
- `ui-tauri/src-tauri/Cargo.toml` - Tauri dependencies
- `ui-tauri/package.json` - Frontend dependencies
- `ui-tauri/src-tauri/tauri.conf.json` - Tauri config
- `crates/browser-core/migrations/` - SQL migrations

### Critical TODOs
1. `webview_manager.rs:43` - Implement per-webview proxy config (CRITICAL, Phase 2)
2. `auth.rs:127` - Implement password verification with hashing (HIGH, Security, Phase 7)
3. `tab_manager.rs:110` - Add cleanup logic for closed tabs (HIGH, Phase 1)
4. `proxy_rotation.rs` - Multiple TODOs for rotation strategies (HIGH, Phase 3)

### Commands Cheat Sheet
```bash
cargo build                      # Build workspace
cargo test                       # Run tests
cargo check                      # Check without building
cargo fmt                        # Format code
cargo clippy                     # Lint code
cd ui-tauri && bun run tauri dev # Run Tauri dev mode
sqlx migrate run                 # Run migrations
cargo add dependency_name        # Add dependency
```

### External Resources
- Tauri: https://tauri.app/v2/
- sqlx: https://docs.rs/sqlx/
- Svelte: https://svelte.dev/docs
- Tokio: https://tokio.rs/
- chromiumoxide: https://docs.rs/chromiumoxide/

---

## 🎯 Quick Start for Agent

### When Starting New Task:
1. Read this document to understand project structure
2. Identify which phase the task belongs to
3. Review relevant patterns
4. Check for TODOs in related files
5. Follow code generation guidelines
6. Write tests for new functionality

### Priority Order:
1. Phase 1: Core Browser Engine
2. Phase 2: Proxy Integration
3. Phase 3: Proxy Rotation
4. Phase 4: Free Proxy Providers
5. Phase 5: UI/UX
6. Phase 6: Advanced Features
7. Phase 7: Testing & Security
8. Phase 8: Deployment

### Before Committing:
- ✅ Code compiles without warnings
- ✅ Tests pass (`cargo test`)
- ✅ Code is formatted (`cargo fmt`)
- ✅ No clippy warnings (`cargo clippy`)
- ✅ Changes are documented
- ✅ TODOs are addressed or documented

---

**Last Updated**: 2024-01-20
**Agent Version**: 1.0.0
**Project Status**: In Development - Phase 1 & 2 Critical
