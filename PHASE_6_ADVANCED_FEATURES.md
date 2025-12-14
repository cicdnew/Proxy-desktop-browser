# Phase 6: Advanced Features

## 6.1 Cookie & Storage Isolation

**File**: `crates/browser-core/src/tab_isolation.rs`

**Claude Opus 4.5 Prompt:**
```
Implement complete isolation between tabs for cookies, localStorage, sessionStorage, and IndexedDB.

REQUIREMENTS:
1. Each tab has its own isolated storage context
2. Cookies are not shared between tabs (unless explicitly configured)
3. localStorage/sessionStorage per tab
4. IndexedDB isolation
5. Service worker isolation
6. Cache isolation
7. Optional: Allow tab grouping with shared storage

ARCHITECTURE:
```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationContext {
    pub id: String,
    pub cookies: HashMap<String, Cookie>,
    pub local_storage: HashMap<String, String>,
    pub session_storage: HashMap<String, String>,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<DateTime<Utc>>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

pub struct TabIsolationManager {
    contexts: Arc<Mutex<HashMap<TabId, IsolationContext>>>,
    base_storage_dir: PathBuf,
}

impl TabIsolationManager {
    pub fn new(base_dir: PathBuf) -> Self;
    
    pub async fn create_context(&self, tab_id: TabId) -> Result<IsolationContext>;
    pub async fn destroy_context(&self, tab_id: TabId) -> Result<()>;
    
    // Cookie management
    pub async fn set_cookie(&self, tab_id: TabId, cookie: Cookie) -> Result<()>;
    pub async fn get_cookies(&self, tab_id: TabId, domain: &str) -> Result<Vec<Cookie>>;
    pub async fn delete_cookie(&self, tab_id: TabId, name: &str, domain: &str) -> Result<()>;
    pub async fn clear_cookies(&self, tab_id: TabId) -> Result<()>;
    
    // Storage management
    pub async fn set_local_storage(&self, tab_id: TabId, key: &str, value: &str) -> Result<()>;
    pub async fn get_local_storage(&self, tab_id: TabId, key: &str) -> Result<Option<String>>;
    pub async fn remove_local_storage(&self, tab_id: TabId, key: &str) -> Result<()>;
    pub async fn clear_local_storage(&self, tab_id: TabId) -> Result<()>;
    
    // Session storage (cleared when tab closes)
    pub async fn set_session_storage(&self, tab_id: TabId, key: &str, value: &str) -> Result<()>;
    pub async fn get_session_storage(&self, tab_id: TabId, key: &str) -> Result<Option<String>>;
    
    // Cleanup
    pub async fn cleanup_expired_cookies(&self) -> Result<usize>;
    pub async fn get_storage_size(&self, tab_id: TabId) -> Result<u64>;
}
```

INTEGRATION WITH TAURI:
- Create custom protocol handler that checks isolation context
- Intercept cookie operations from WebView
- Inject JavaScript to override localStorage/sessionStorage
- Configure WebView with custom data directory per tab

PERSISTENCE:
- Save isolation contexts to disk for session restore
- Load contexts on application startup
- Export/import contexts for backup

Implement with comprehensive error handling and logging.
```

---

## 6.2 Fingerprint Protection

**File**: `crates/browser-core/src/fingerprint.rs`

**Claude Opus 4.5 Prompt:**
```
Implement browser fingerprinting protection to make each tab appear as a different browser/device.

FINGERPRINTING VECTORS TO MODIFY:
1. User Agent string
2. Screen resolution and color depth
3. Canvas fingerprinting
4. WebGL fingerprinting
5. Audio context fingerprinting
6. Font enumeration
7. Timezone
8. Language preferences
9. Hardware concurrency (CPU cores)
10. Device memory
11. Platform/OS
12. Battery status
13. Network information
14. Plugins list
15. WebRTC IP leak prevention

IMPLEMENTATION:
```rust
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    pub user_agent: String,
    pub screen_resolution: (u32, u32),
    pub color_depth: u32,
    pub timezone: String,
    pub language: String,
    pub platform: String,
    pub hardware_concurrency: u32,
    pub device_memory: u32,
    pub canvas_noise: Vec<u8>,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

pub struct FingerprintGenerator {
    user_agents: Vec<String>,
    resolutions: Vec<(u32, u32)>,
    timezones: Vec<String>,
    languages: Vec<String>,
}

impl FingerprintGenerator {
    pub fn new() -> Self;
    
    pub fn generate_random_fingerprint(&self) -> BrowserFingerprint;
    pub fn generate_consistent_fingerprint(&self, seed: &str) -> BrowserFingerprint;
    pub fn generate_fingerprint_for_country(&self, country: &str) -> BrowserFingerprint;
    
    fn generate_user_agent(&self, os: &str, browser: &str) -> String;
    fn generate_canvas_noise(&self) -> Vec<u8>;
    fn generate_webgl_info(&self) -> (String, String);
}

pub struct FingerprintProtection {
    generator: FingerprintGenerator,
    active_fingerprints: HashMap<TabId, BrowserFingerprint>,
}

impl FingerprintProtection {
    pub fn new() -> Self;
    
    pub fn assign_fingerprint(&mut self, tab_id: TabId) -> BrowserFingerprint;
    pub fn get_fingerprint(&self, tab_id: TabId) -> Option<&BrowserFingerprint>;
    pub fn remove_fingerprint(&mut self, tab_id: TabId);
    
    pub fn generate_injection_script(&self, fingerprint: &BrowserFingerprint) -> String;
}
```

JAVASCRIPT INJECTION:
Generate a script that overrides browser APIs:
```javascript
// Override navigator properties
Object.defineProperty(navigator, 'userAgent', {
    get: () => '{user_agent}'
});

Object.defineProperty(navigator, 'hardwareConcurrency', {
    get: () => {hardware_concurrency}
});

// Canvas fingerprinting protection
const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
HTMLCanvasElement.prototype.toDataURL = function() {
    const context = this.getContext('2d');
    // Add noise to canvas data
    const imageData = context.getImageData(0, 0, this.width, this.height);
    for (let i = 0; i < imageData.data.length; i += 4) {
        imageData.data[i] += Math.random() * 10 - 5;
    }
    context.putImageData(imageData, 0, 0);
    return originalToDataURL.apply(this, arguments);
};

// WebGL fingerprinting protection
const originalGetParameter = WebGLRenderingContext.prototype.getParameter;
WebGLRenderingContext.prototype.getParameter = function(parameter) {
    if (parameter === 0x1F00) return '{webgl_vendor}';
    if (parameter === 0x1F01) return '{webgl_renderer}';
    return originalGetParameter.apply(this, arguments);
};

// WebRTC IP leak prevention
const originalGetUserMedia = navigator.mediaDevices.getUserMedia;
navigator.mediaDevices.getUserMedia = function() {
    return Promise.reject(new Error('Permission denied'));
};
```

INTEGRATION:
- Inject fingerprint script before page loads
- Store fingerprint per tab
- Update fingerprint on proxy rotation (optional)
- Provide UI to view/edit fingerprint

Implement with comprehensive browser API coverage and testing.
```

---

## 6.3 Download Manager

**File**: `crates/browser-core/src/download_manager.rs`

**Claude Opus 4.5 Prompt:**
```
Create a comprehensive download manager with pause/resume, speed limiting, and download history.

FEATURES:
1. Intercept download requests from WebView
2. Show download progress in UI
3. Pause/resume downloads
4. Cancel downloads
5. Download queue management
6. Concurrent download limit
7. Speed limiting (bandwidth throttling)
8. Download history
9. File type associations
10. Virus scanning integration (optional)
11. Automatic retry on failure
12. Download through active proxy

IMPLEMENTATION:
```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: DownloadId,
    pub url: String,
    pub filename: String,
    pub destination: PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub speed_bps: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tab_id: TabId,
    pub proxy_id: Option<ProxyId>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

pub struct DownloadManager {
    downloads: Arc<Mutex<HashMap<DownloadId, Download>>>,
    active_downloads: Arc<Mutex<Vec<DownloadId>>>,
    download_dir: PathBuf,
    max_concurrent: usize,
    speed_limit_bps: Option<u64>,
    client: Client,
}

impl DownloadManager {
    pub fn new(download_dir: PathBuf, max_concurrent: usize) -> Self;
    
    pub async fn start_download(
        &self,
        url: &str,
        filename: Option<String>,
        tab_id: TabId,
        proxy: Option<ProxyConfig>
    ) -> Result<DownloadId>;
    
    pub async fn pause_download(&self, id: DownloadId) -> Result<()>;
    pub async fn resume_download(&self, id: DownloadId) -> Result<()>;
    pub async fn cancel_download(&self, id: DownloadId) -> Result<()>;
    pub async fn retry_download(&self, id: DownloadId) -> Result<()>;
    
    pub async fn get_download(&self, id: DownloadId) -> Option<Download>;
    pub async fn get_all_downloads(&self) -> Vec<Download>;
    pub async fn get_active_downloads(&self) -> Vec<Download>;
    
    pub async fn clear_completed(&self) -> Result<()>;
    pub async fn clear_all(&self) -> Result<()>;
    
    pub fn set_speed_limit(&mut self, bps: Option<u64>);
    
    async fn download_file(
        &self,
        download_id: DownloadId,
        url: &str,
        destination: PathBuf,
        proxy: Option<ProxyConfig>
    ) -> Result<()>;
    
    async fn process_queue(&self);
}
```

DOWNLOAD PROCESS:
1. Receive download request from WebView
2. Extract filename from Content-Disposition or URL
3. Check if file already exists (prompt user)
4. Add to download queue
5. Start download with configured proxy
6. Stream response to file with progress tracking
7. Calculate download speed
8. Emit progress events to UI
9. Handle errors and retries
10. Notify completion

UI INTEGRATION:
- Show download notification
- Download progress bar in status bar
- Download manager panel (list of downloads)
- Context menu actions (pause, cancel, open, show in folder)

Implement with proper error handling, progress tracking, and event emission.
```

---

## 6.4 Bookmark & History Manager

**File**: `crates/browser-core/src/bookmark_manager.rs` and `history_manager.rs`

**Claude Opus 4.5 Prompt:**
```
Implement bookmark and history management systems with full CRUD operations and search.

BOOKMARK MANAGER:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub parent_folder: Option<FolderId>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub id: FolderId,
    pub name: String,
    pub parent: Option<FolderId>,
    pub children: Vec<FolderId>,
}

pub struct BookmarkManager {
    bookmarks: HashMap<BookmarkId, Bookmark>,
    folders: HashMap<FolderId, BookmarkFolder>,
    db: Arc<Mutex<Connection>>,
}

impl BookmarkManager {
    pub async fn add_bookmark(&mut self, title: &str, url: &str, folder: Option<FolderId>) -> Result<BookmarkId>;
    pub async fn remove_bookmark(&mut self, id: BookmarkId) -> Result<()>;
    pub async fn update_bookmark(&mut self, id: BookmarkId, bookmark: Bookmark) -> Result<()>;
    pub async fn get_bookmark(&self, id: BookmarkId) -> Option<&Bookmark>;
    pub async fn search_bookmarks(&self, query: &str) -> Vec<Bookmark>;
    pub async fn get_bookmarks_in_folder(&self, folder_id: FolderId) -> Vec<Bookmark>;
    
    pub async fn create_folder(&mut self, name: &str, parent: Option<FolderId>) -> Result<FolderId>;
    pub async fn delete_folder(&mut self, id: FolderId) -> Result<()>;
    pub async fn move_bookmark(&mut self, bookmark_id: BookmarkId, to_folder: FolderId) -> Result<()>;
    
    pub async fn export_to_html(&self) -> Result<String>;
    pub async fn import_from_html(&mut self, html: &str) -> Result<usize>;
}
```

HISTORY MANAGER:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: HistoryId,
    pub url: String,
    pub title: String,
    pub visit_count: u32,
    pub last_visit: DateTime<Utc>,
    pub first_visit: DateTime<Utc>,
}

pub struct HistoryManager {
    db: Arc<Mutex<Connection>>,
    max_entries: usize,
}

impl HistoryManager {
    pub async fn add_visit(&self, url: &str, title: &str) -> Result<()>;
    pub async fn get_history(&self, limit: usize, offset: usize) -> Result<Vec<HistoryEntry>>;
    pub async fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>>;
    pub async fn delete_entry(&self, id: HistoryId) -> Result<()>;
    pub async fn clear_history(&self, older_than: Option<DateTime<Utc>>) -> Result<usize>;
    pub async fn get_most_visited(&self, limit: usize) -> Result<Vec<HistoryEntry>>;
    pub async fn get_suggestions(&self, query: &str) -> Result<Vec<String>>;
}
```

DATABASE SCHEMA:
```sql
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    favicon TEXT,
    folder_id TEXT,
    tags TEXT,
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE TABLE bookmark_folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    position INTEGER
);

CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    visit_count INTEGER DEFAULT 1,
    last_visit TEXT NOT NULL,
    first_visit TEXT NOT NULL
);

CREATE INDEX idx_history_url ON history(url);
CREATE INDEX idx_history_last_visit ON history(last_visit);
CREATE INDEX idx_bookmarks_folder ON bookmarks(folder_id);
```

Implement with full-text search support and efficient queries.
```

---

## 6.5 Session Management & Restore

**File**: `crates/browser-core/src/session_manager.rs`

**Claude Opus 4.5 Prompt:**
```
Implement session management to save and restore browser state (tabs, history, cookies, etc.).

FEATURES:
1. Auto-save session periodically
2. Save session on application close
3. Restore session on startup
4. Multiple saved sessions (named sessions)
5. Export/import sessions
6. Crash recovery

IMPLEMENTATION:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub tabs: Vec<TabSnapshot>,
    pub active_tab_index: usize,
    pub window_geometry: WindowGeometry,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub url: String,
    pub title: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub scroll_position: (f64, f64),
    pub zoom_level: f64,
    pub proxy_config: Option<ProxyConfig>,
    pub isolation_context: IsolationContext,
}

pub struct SessionManager {
    current_session: Option<Session>,
    saved_sessions: HashMap<SessionId, Session>,
    storage_dir: PathBuf,
    auto_save_interval: Duration,
}

impl SessionManager {
    pub fn new(storage_dir: PathBuf) -> Self;
    
    pub async fn capture_current_session(&self) -> Result<Session>;
    pub async fn restore_session(&self, session: &Session) -> Result<()>;
    
    pub async fn save_session(&self, name: &str) -> Result<SessionId>;
    pub async fn load_session(&self, id: SessionId) -> Result<Session>;
    pub async fn delete_session(&self, id: SessionId) -> Result<()>;
    pub async fn list_sessions(&self) -> Vec<Session>;
    
    pub async fn export_session(&self, id: SessionId) -> Result<String>;
    pub async fn import_session(&self, json: &str) -> Result<SessionId>;
    
    pub async fn start_auto_save(&self);
    pub async fn stop_auto_save(&self);
    
    async fn save_to_disk(&self, session: &Session) -> Result<()>;
    async fn load_from_disk(&self, id: SessionId) -> Result<Session>;
}
```

INTEGRATION:
- Hook into application lifecycle events
- Save session before application closes
- Restore last session on startup (if configured)
- Provide UI for session management

Implement with proper error handling and data validation.
```

