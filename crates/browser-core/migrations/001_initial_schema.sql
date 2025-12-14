-- Proxies table
CREATE TABLE IF NOT EXISTS proxies (
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

CREATE INDEX idx_proxies_active ON proxies(is_active);
CREATE INDEX idx_proxies_country ON proxies(country);
CREATE INDEX idx_proxies_provider ON proxies(source_provider);

-- Proxy metrics table
CREATE TABLE IF NOT EXISTS proxy_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proxy_id TEXT NOT NULL,
    response_time_ms REAL NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    checked_at TEXT NOT NULL,
    FOREIGN KEY (proxy_id) REFERENCES proxies(id) ON DELETE CASCADE
);

CREATE INDEX idx_metrics_proxy ON proxy_metrics(proxy_id);
CREATE INDEX idx_metrics_time ON proxy_metrics(checked_at);

-- Tabs table
CREATE TABLE IF NOT EXISTS tabs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    favicon TEXT,
    proxy_id TEXT,
    virtual_ip TEXT,
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL,
    is_pinned BOOLEAN DEFAULT 0,
    is_suspended BOOLEAN DEFAULT 0,
    FOREIGN KEY (proxy_id) REFERENCES proxies(id) ON DELETE SET NULL
);

CREATE INDEX idx_tabs_active ON tabs(is_pinned);
CREATE INDEX idx_tabs_proxy ON tabs(proxy_id);
CREATE INDEX idx_tabs_last_active ON tabs(last_active);

-- Bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    favicon TEXT,
    folder_id TEXT,
    tags TEXT,
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE INDEX idx_bookmarks_folder ON bookmarks(folder_id);
CREATE INDEX idx_bookmarks_created ON bookmarks(created_at);

-- History table
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    visit_count INTEGER DEFAULT 1,
    last_visit TEXT NOT NULL,
    first_visit TEXT NOT NULL
);

CREATE INDEX idx_history_url ON history(url);
CREATE INDEX idx_history_last_visit ON history(last_visit);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Downloads table
CREATE TABLE IF NOT EXISTS downloads (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    status TEXT NOT NULL,
    progress INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    completed_at TEXT
);

CREATE INDEX idx_downloads_status ON downloads(status);
CREATE INDEX idx_downloads_created ON downloads(created_at);
