use crate::fingerprint::BrowserFingerprint;
use crate::tab_isolation::{NetworkConfig, TabProfile, TabStatus};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use uuid::Uuid;
use virtual_ip::{IPGenerator, VirtualIP};
use sqlx::{SqlitePool, Row};
use tracing::{info, error, debug};

pub struct TabIPManager {
    tabs: RwLock<HashMap<String, TabProfile>>,
    ip_generator: IPGenerator,
    db_pool: SqlitePool,
}

impl TabIPManager {
    /// Create new TabIPManager with database persistence
    pub async fn new(ip_generator: IPGenerator, db_pool: SqlitePool) -> Result<Self> {
        let manager = Self {
            tabs: RwLock::new(HashMap::new()),
            ip_generator,
            db_pool,
        };
        
        // Restore tabs from database on startup
        manager.restore_tabs_from_db().await?;
        
        Ok(manager)
    }

    /// Create new tab with IP from specific country
    pub async fn create_tab(&self, country_code: &str) -> Result<TabProfile> {
        let tab_id = Uuid::new_v4().to_string();
        let virtual_ip = self.ip_generator.generate_for_country(country_code)?;
        let fingerprint = self.generate_matching_fingerprint(&virtual_ip)?;
        let network_config = self.create_network_config(&virtual_ip)?;

        let tab_profile = TabProfile {
            tab_id: tab_id.clone(),
            virtual_ip,
            fingerprint,
            network_config,
            storage_path: format!("./data/tabs/{}", tab_id),
            process_id: None,
            created_at: SystemTime::now(),
            last_active: SystemTime::now(),
            status: TabStatus::Creating,
        };

        // Save to in-memory cache
        self.tabs
            .write()
            .await
            .insert(tab_id.clone(), tab_profile.clone());

        // Persist to database
        self.save_tab_to_db(&tab_profile).await?;

        Ok(tab_profile)
    }

    /// Create tab with random country
    pub async fn create_tab_random(&self) -> Result<TabProfile> {
        let virtual_ip = self.ip_generator.generate_random()?;
        self.create_tab(&virtual_ip.country_code).await
    }

    /// Rotate IP for existing tab
    pub async fn rotate_ip(&self, tab_id: &str, new_country_code: Option<&str>) -> Result<VirtualIP> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs
            .get_mut(tab_id)
            .ok_or_else(|| anyhow!("Tab not found"))?;

        let new_ip = if let Some(country) = new_country_code {
            self.ip_generator.generate_for_country(country)?
        } else {
            self.ip_generator
                .generate_for_country(&tab.virtual_ip.country_code)?
        };

        tab.virtual_ip = new_ip.clone();
        tab.fingerprint = self.generate_matching_fingerprint(&new_ip)?;
        tab.network_config = self.create_network_config(&new_ip)?;
        tab.last_active = SystemTime::now();

        Ok(new_ip)
    }

    /// Get tab by ID
    pub async fn get_tab(&self, tab_id: &str) -> Option<TabProfile> {
        self.tabs.read().await.get(tab_id).cloned()
    }

    /// List all active tabs
    pub async fn list_tabs(&self) -> Vec<TabProfile> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Close tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        // Remove from in-memory cache
        self.tabs.write().await.remove(tab_id);
        
        // Remove from database
        self.delete_tab_from_db(tab_id).await?;
        
        // TODO: cleanup storage/process when integrated.
        Ok(())
    }

    /// Navigate an existing tab (stub; integrate with engine later)
    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<()> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs
            .get_mut(tab_id)
            .ok_or_else(|| anyhow!("Tab not found"))?;
        tab.last_active = SystemTime::now();
        
        // Update in database with URL
        drop(tabs); // Release lock before database operation
        self.update_tab_in_db(tab_id, None, Some(url)).await?;
        
        Ok(())
    }

    fn generate_matching_fingerprint(&self, ip: &VirtualIP) -> Result<BrowserFingerprint> {
        Ok(BrowserFingerprint {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            accept_language: ip.language.clone(),
            timezone: ip.timezone.clone(),
            screen_resolution: (1920, 1080),
            color_depth: 24,
            hardware_concurrency: 8,
            device_memory: 8,
            platform: "Win32".to_string(),
            webgl_vendor: "Google Inc.".to_string(),
            webgl_renderer: "ANGLE (NVIDIA GeForce GTX 1060)".to_string(),
            canvas_hash: self.generate_canvas_hash(),
            audio_hash: self.generate_audio_hash(),
        })
    }

    fn create_network_config(&self, ip: &VirtualIP) -> Result<NetworkConfig> {
        Ok(NetworkConfig {
            proxy_url: ip.proxy_url.clone(),
            dns_servers: vec!["1.1.1.1".into(), "8.8.8.8".into()],
            tls_profile: crate::tab_isolation::TLSProfile {
                version: "TLS 1.3".into(),
                cipher_suites: vec![
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_AES_256_GCM_SHA384".into(),
                ],
                extensions: vec!["server_name".into(), "supported_groups".into()],
                ja3_hash: "771,4865-4866-4867,0-23-65281,29-23-24,0".into(),
            },
            http2_settings: crate::tab_isolation::HTTP2Settings {
                settings_frame: vec![(1, 65536), (2, 0), (3, 1000)],
                window_update: 15663105,
                priority: vec![],
            },
            tcp_fingerprint: crate::tab_isolation::TCPFingerprint {
                ttl: 64,
                window_size: 65535,
                options: vec!["mss".into(), "nop".into(), "ws".into()],
            },
        })
    }

    fn generate_canvas_hash(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}", rng.gen::<u64>())
    }

    fn generate_audio_hash(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}", rng.gen::<u64>())
    }

    /// Save tab profile to database
    async fn save_tab_to_db(&self, tab: &TabProfile) -> Result<()> {
        let created_at = tab.created_at.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        let last_active = tab.last_active.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO tabs (
                id, title, url, favicon, proxy_id, virtual_ip, 
                created_at, last_active, is_pinned, is_suspended
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&tab.tab_id)
        .bind("") // title - will be updated when navigating
        .bind("") // url - will be updated when navigating
        .bind(None::<String>) // favicon
        .bind(None::<String>) // proxy_id
        .bind(serde_json::to_string(&tab.virtual_ip)?)
        .bind(created_at as i64)
        .bind(last_active as i64)
        .bind(false) // is_pinned
        .bind(false) // is_suspended
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to save tab to database: {}", e);
            anyhow!("Database error: {}", e)
        })?;

        debug!("Saved tab {} to database", tab.tab_id);
        Ok(())
    }

    /// Restore all tabs from database
    async fn restore_tabs_from_db(&self) -> Result<()> {
        let rows = sqlx::query(
            r#"
            SELECT id, virtual_ip, created_at, last_active, is_pinned, is_suspended
            FROM tabs
            ORDER BY last_active DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to restore tabs from database: {}", e);
            anyhow!("Database error: {}", e)
        })?;

        let mut tabs = self.tabs.write().await;
        
        for row in rows {
            // Get tab ID and virtual IP
            let tab_id: String = row.get(0);
            let virtual_ip_json: String = row.get(1);
            
            // Deserialize virtual IP
            let virtual_ip: VirtualIP = serde_json::from_str(&virtual_ip_json)
                .map_err(|e| anyhow!("Failed to deserialize virtual IP: {}", e))?;
            
            // Convert timestamps
            let created_at = SystemTime::UNIX_EPOCH + Duration::from_secs(row.get::<i64, usize>(2) as u64);
            let last_active = SystemTime::UNIX_EPOCH + Duration::from_secs(row.get::<i64, usize>(3) as u64);
            
            // Recreate tab profile (note: fingerprint and network_config will be regenerated)
            let fingerprint = self.generate_matching_fingerprint(&virtual_ip)?;
            let network_config = self.create_network_config(&virtual_ip)?;
            
            let tab_profile = TabProfile {
                tab_id: tab_id.clone(),
                virtual_ip,
                fingerprint,
                network_config,
                storage_path: format!("./data/tabs/{}", tab_id),
                process_id: None,
                created_at,
                last_active,
                status: TabStatus::Active, // Assume restored tabs are active
            };
            
            tabs.insert(tab_id, tab_profile);
        }
        
        info!("Restored {} tabs from database", tabs.len());
        Ok(())
    }

    /// Update tab in database
    async fn update_tab_in_db(&self, tab_id: &str, title: Option<&str>, url: Option<&str>) -> Result<()> {
        let last_active = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        sqlx::query(
            r#"
            UPDATE tabs 
            SET title = COALESCE(?, title),
                url = COALESCE(?, url),
                last_active = ?
            WHERE id = ?
            "#
        )
        .bind(title)
        .bind(url)
        .bind(last_active as i64)
        .bind(tab_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to update tab in database: {}", e);
            anyhow!("Database error: {}", e)
        })?;

        Ok(())
    }

    /// Remove tab from database
    async fn delete_tab_from_db(&self, tab_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM tabs WHERE id = ?")
            .bind(tab_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| {
                error!("Failed to delete tab from database: {}", e);
                anyhow!("Database error: {}", e)
            })?;

        debug!("Deleted tab {} from database", tab_id);
        Ok(())
    }
}
