use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visit_count: i32,
    pub last_visit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub folder: Option<String>,
    pub created_at: i64,
}

pub struct StorageEngine {
    db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl StorageEngine {
    pub fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("browser_data.db");
        let conn = Connection::open(&db_path)?;
        
        // Initialize tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cookies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                path TEXT DEFAULT '/',
                expires INTEGER,
                http_only INTEGER DEFAULT 0,
                secure INTEGER DEFAULT 0,
                same_site TEXT DEFAULT 'Lax',
                UNIQUE(domain, name, path)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                visit_count INTEGER DEFAULT 1,
                last_visit INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                folder TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS local_storage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                origin TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                UNIQUE(origin, key)
            )",
            [],
        )?;

        Ok(Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // Cookie operations
    pub async fn set_cookie(&self, cookie: &Cookie) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO cookies (domain, name, value, path, expires, http_only, secure, same_site)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cookie.domain,
                cookie.name,
                cookie.value,
                cookie.path,
                cookie.expires,
                cookie.http_only as i32,
                cookie.secure as i32,
                cookie.same_site
            ],
        )?;
        Ok(())
    }

    pub async fn get_cookies(&self, domain: &str) -> Result<Vec<Cookie>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT domain, name, value, path, expires, http_only, secure, same_site 
             FROM cookies WHERE domain LIKE ?1"
        )?;
        
        let pattern = format!("%{}", domain);
        let cookies = stmt.query_map([pattern], |row| {
            Ok(Cookie {
                domain: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                path: row.get(3)?,
                expires: row.get(4)?,
                http_only: row.get::<_, i32>(5)? != 0,
                secure: row.get::<_, i32>(6)? != 0,
                same_site: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(cookies)
    }

    pub async fn get_all_cookies(&self) -> Result<Vec<Cookie>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT domain, name, value, path, expires, http_only, secure, same_site FROM cookies"
        )?;
        
        let cookies = stmt.query_map([], |row| {
            Ok(Cookie {
                domain: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                path: row.get(3)?,
                expires: row.get(4)?,
                http_only: row.get::<_, i32>(5)? != 0,
                secure: row.get::<_, i32>(6)? != 0,
                same_site: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(cookies)
    }

    pub async fn clear_cookies(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM cookies", [])?;
        Ok(())
    }

    // History operations
    pub async fn add_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO history (url, title, visit_count, last_visit) 
             VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(url) DO UPDATE SET 
                title = COALESCE(?2, title),
                visit_count = visit_count + 1,
                last_visit = ?3",
            params![url, title, now],
        )?;
        Ok(())
    }

    pub async fn get_history(&self, limit: i64) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_count, last_visit 
             FROM history ORDER BY last_visit DESC LIMIT ?1"
        )?;
        
        let entries = stmt.query_map([limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_count: row.get(3)?,
                last_visit: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(entries)
    }

    pub async fn search_history(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_count, last_visit 
             FROM history WHERE url LIKE ?1 OR title LIKE ?1
             ORDER BY last_visit DESC LIMIT 100"
        )?;
        
        let pattern = format!("%{}%", query);
        let entries = stmt.query_map([pattern], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_count: row.get(3)?,
                last_visit: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(entries)
    }

    pub async fn clear_history(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    // Bookmark operations
    pub async fn add_bookmark(&self, url: &str, title: &str, folder: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO bookmarks (url, title, folder, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![url, title, folder, now],
        )?;
        
        Ok(conn.last_insert_rowid())
    }

    pub async fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, folder, created_at FROM bookmarks ORDER BY created_at DESC"
        )?;
        
        let bookmarks = stmt.query_map([], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                folder: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(bookmarks)
    }

    pub async fn delete_bookmark(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", [id])?;
        Ok(())
    }

    // Local storage operations
    pub async fn set_local_storage(&self, origin: &str, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO local_storage (origin, key, value) VALUES (?1, ?2, ?3)",
            params![origin, key, value],
        )?;
        Ok(())
    }

    pub async fn get_local_storage(&self, origin: &str, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT value FROM local_storage WHERE origin = ?1 AND key = ?2"
        )?;
        
        let value = stmt.query_row([origin, key], |row| row.get(0)).ok();
        Ok(value)
    }

    pub async fn get_all_local_storage(&self, origin: &str) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT key, value FROM local_storage WHERE origin = ?1"
        )?;
        
        let items = stmt.query_map([origin], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}
