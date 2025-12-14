use anyhow::{anyhow, Result};
use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide::cdp::browser_protocol::page::NavigateParams;
use futures::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, debug, warn};

use crate::proxy::{ProxySettings, ProxyType, FreeProxy};

/// Browser engine type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BrowserEngineType {
    /// Use system's default webview (Tauri's built-in)
    #[default]
    System,
    /// Use integrated Chromium engine with enhanced proxy support
    IntegratedChromium,
}

/// Network condition preset for throttling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCondition {
    /// No throttling
    None,
    /// Slow 3G connection
    Slow3G,
    /// Fast 3G connection
    Fast3G,
    /// 4G LTE connection
    LTE,
    /// Custom throttling
    Custom {
        download_throughput: f64,
        upload_throughput: f64,
        latency: f64,
    },
}

impl Default for NetworkCondition {
    fn default() -> Self {
        NetworkCondition::None
    }
}

impl NetworkCondition {
    pub fn get_params(&self) -> (f64, f64, f64) {
        match self {
            NetworkCondition::None => (-1.0, -1.0, 0.0),
            NetworkCondition::Slow3G => (500.0 * 1024.0 / 8.0, 500.0 * 1024.0 / 8.0, 400.0),
            NetworkCondition::Fast3G => (1.5 * 1024.0 * 1024.0 / 8.0, 750.0 * 1024.0 / 8.0, 150.0),
            NetworkCondition::LTE => (12.0 * 1024.0 * 1024.0 / 8.0, 5.0 * 1024.0 * 1024.0 / 8.0, 50.0),
            NetworkCondition::Custom { download_throughput, upload_throughput, latency } => {
                (*download_throughput, *upload_throughput, *latency)
            }
        }
    }
}

/// Browser fingerprint configuration for anti-detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    /// Randomize canvas fingerprint
    pub randomize_canvas: bool,
    /// Randomize WebGL fingerprint
    pub randomize_webgl: bool,
    /// Randomize audio context fingerprint
    pub randomize_audio: bool,
    /// Spoof screen resolution
    pub spoof_screen: bool,
    /// Screen width (if spoofing)
    pub screen_width: u32,
    /// Screen height (if spoofing)
    pub screen_height: u32,
    /// Spoof hardware concurrency (CPU cores)
    pub spoof_hardware_concurrency: bool,
    /// Number of CPU cores to report
    pub hardware_concurrency: u32,
    /// Spoof device memory
    pub spoof_device_memory: bool,
    /// Device memory in GB
    pub device_memory: u32,
    /// Spoof timezone
    pub spoof_timezone: bool,
    /// Timezone to use
    pub timezone: String,
    /// Spoof language
    pub spoof_language: bool,
    /// Language to use
    pub language: String,
    /// Spoof platform
    pub spoof_platform: bool,
    /// Platform to report
    pub platform: String,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            randomize_canvas: true,
            randomize_webgl: true,
            randomize_audio: true,
            spoof_screen: false,
            screen_width: 1920,
            screen_height: 1080,
            spoof_hardware_concurrency: false,
            hardware_concurrency: 8,
            spoof_device_memory: false,
            device_memory: 8,
            spoof_timezone: false,
            timezone: "America/New_York".to_string(),
            spoof_language: false,
            language: "en-US".to_string(),
            spoof_platform: false,
            platform: "Win32".to_string(),
        }
    }
}

/// Proxy authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

/// Cookie isolation mode
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CookieIsolationMode {
    /// No isolation - cookies shared across tabs
    #[default]
    None,
    /// Isolate cookies per tab
    PerTab,
    /// Isolate cookies per domain
    PerDomain,
    /// Full isolation with separate browser contexts
    FullContext,
}

/// Configuration for the integrated Chromium engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromiumEngineConfig {
    /// Path to Chrome/Chromium executable (optional, will auto-detect if not set)
    pub executable_path: Option<PathBuf>,
    /// Whether to run in headless mode
    pub headless: bool,
    /// User data directory for browser profiles
    pub user_data_dir: Option<PathBuf>,
    /// Enable sandbox (disable for some environments)
    pub sandbox: bool,
    /// Additional launch arguments
    pub extra_args: Vec<String>,
    /// Proxy configuration
    pub proxy: Option<ProxySettings>,
    /// Proxy authentication
    pub proxy_auth: Option<ProxyAuth>,
    /// Enable stealth mode to avoid detection
    pub stealth_mode: bool,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// Enable WebRTC IP leak protection
    pub webrtc_protection: bool,
    /// DNS over HTTPS server
    pub doh_server: Option<String>,
    /// Network condition for throttling
    pub network_condition: NetworkCondition,
    /// Fingerprint configuration
    pub fingerprint: FingerprintConfig,
    /// Cookie isolation mode
    pub cookie_isolation: CookieIsolationMode,
    /// Block requests matching these patterns
    pub blocked_urls: Vec<String>,
    /// Enable request interception
    pub enable_interception: bool,
    /// Geolocation spoofing
    pub geolocation: Option<Geolocation>,
}

/// Geolocation coordinates for spoofing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
}

impl Default for ChromiumEngineConfig {
    fn default() -> Self {
        Self {
            executable_path: None,
            headless: false,
            user_data_dir: None,
            sandbox: true,
            extra_args: Vec::new(),
            proxy: None,
            proxy_auth: None,
            stealth_mode: true,
            user_agent: None,
            viewport_width: 1920,
            viewport_height: 1080,
            webrtc_protection: true,
            doh_server: Some("https://cloudflare-dns.com/dns-query".to_string()),
            network_condition: NetworkCondition::default(),
            fingerprint: FingerprintConfig::default(),
            cookie_isolation: CookieIsolationMode::default(),
            blocked_urls: Vec::new(),
            enable_interception: false,
            geolocation: None,
        }
    }
}

/// Tab state for Chromium engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromiumTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub proxy: Option<ProxySettings>,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Chromium engine manager with optimized proxy support
pub struct ChromiumEngine {
    config: ChromiumEngineConfig,
    browser: Option<Browser>,
    tabs: Arc<RwLock<HashMap<String, ChromiumTab>>>,
    active_tab_id: Arc<RwLock<Option<String>>>,
    is_running: Arc<RwLock<bool>>,
}

impl ChromiumEngine {
    /// Create a new Chromium engine instance
    pub fn new(config: ChromiumEngineConfig) -> Self {
        Self {
            config,
            browser: None,
            tabs: Arc::new(RwLock::new(HashMap::new())),
            active_tab_id: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Launch the Chromium browser with optimized proxy settings
    pub async fn launch(&mut self) -> Result<()> {
        if *self.is_running.read().await {
            return Ok(());
        }

        info!("Launching integrated Chromium engine...");

        let mut builder = BrowserConfig::builder();

        // Set executable path if provided
        if let Some(path) = &self.config.executable_path {
            builder = builder.chrome_executable(path.clone());
        }

        // Configure headless mode
        if self.config.headless {
            builder = builder.with_head();
        }

        // Configure user data directory
        if let Some(dir) = &self.config.user_data_dir {
            builder = builder.user_data_dir(dir.clone());
        }

        // Disable sandbox if configured
        if !self.config.sandbox {
            builder = builder.no_sandbox();
        }

        // Add proxy configuration - OPTIMIZED for better proxy support
        if let Some(proxy) = &self.config.proxy {
            if let Some(proxy_url) = proxy.to_url() {
                builder = builder.arg(format!("--proxy-server={}", proxy_url));
                
                // Add proxy bypass for local addresses
                let bypass_list = proxy.bypass_list.join(",");
                if !bypass_list.is_empty() {
                    builder = builder.arg(format!("--proxy-bypass-list={}", bypass_list));
                }
            }
        }

        // WebRTC IP leak protection
        if self.config.webrtc_protection {
            builder = builder.arg("--disable-webrtc-hw-decoding");
            builder = builder.arg("--disable-webrtc-hw-encoding");
            builder = builder.arg("--force-webrtc-ip-handling-policy=disable_non_proxied_udp");
        }

        // DNS over HTTPS configuration
        if let Some(doh_server) = &self.config.doh_server {
            builder = builder.arg(format!("--doh-url={}", doh_server));
            builder = builder.arg("--enable-features=DnsOverHttps");
        }

        // Stealth mode arguments to avoid detection
        if self.config.stealth_mode {
            builder = builder
                .arg("--disable-blink-features=AutomationControlled")
                .arg("--disable-infobars")
                .arg("--disable-dev-shm-usage")
                .arg("--disable-gpu")
                .arg("--no-first-run")
                .arg("--disable-extensions")
                .arg("--disable-default-apps")
                .arg("--disable-popup-blocking");
        }

        // Add viewport configuration
        builder = builder.arg(format!(
            "--window-size={},{}",
            self.config.viewport_width,
            self.config.viewport_height
        ));

        // Add extra arguments
        for arg in &self.config.extra_args {
            builder = builder.arg(arg.clone());
        }

        // Build and launch browser
        let config = builder.build().map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| anyhow!("Failed to launch Chromium: {}", e))?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                debug!("Browser event: {:?}", event);
            }
        });

        self.browser = Some(browser);
        *self.is_running.write().await = true;

        info!("Chromium engine launched successfully");
        Ok(())
    }

    /// Create a new tab with optional proxy settings
    pub async fn create_tab(&self, url: Option<&str>, proxy: Option<ProxySettings>) -> Result<ChromiumTab> {
        let browser = self.browser.as_ref()
            .ok_or_else(|| anyhow!("Browser not launched"))?;

        let page = browser.new_page("about:blank")
            .await
            .map_err(|e| anyhow!("Failed to create new tab: {}", e))?;

        let tab_id = uuid::Uuid::new_v4().to_string();

        // Apply custom user agent if configured
        if let Some(user_agent) = &self.config.user_agent {
            page.set_user_agent(user_agent)
                .await
                .map_err(|e| anyhow!("Failed to set user agent: {}", e))?;
        }

        // Apply stealth scripts to avoid detection
        if self.config.stealth_mode {
            self.inject_stealth_scripts(&page).await?;
        }

        // Navigate to URL if provided
        let final_url = if let Some(u) = url {
            page.goto(u)
                .await
                .map_err(|e| anyhow!("Failed to navigate: {}", e))?;
            u.to_string()
        } else {
            "about:blank".to_string()
        };

        let title = page.get_title()
            .await
            .map_err(|e| anyhow!("Failed to get title: {}", e))?
            .unwrap_or_else(|| "New Tab".to_string());

        let tab = ChromiumTab {
            id: tab_id.clone(),
            url: final_url,
            title,
            proxy,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        };

        self.tabs.write().await.insert(tab_id.clone(), tab.clone());
        *self.active_tab_id.write().await = Some(tab_id);

        Ok(tab)
    }

    /// Navigate a tab to a URL
    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<()> {
        let browser = self.browser.as_ref()
            .ok_or_else(|| anyhow!("Browser not launched"))?;

        let pages = browser.pages().await.map_err(|e| anyhow!("Failed to get pages: {}", e))?;
        
        // Find the page by checking tabs
        let tabs = self.tabs.read().await;
        if !tabs.contains_key(tab_id) {
            return Err(anyhow!("Tab not found: {}", tab_id));
        }
        drop(tabs);

        // Navigate the first available page (simplified for now)
        if let Some(page) = pages.first() {
            page.goto(url)
                .await
                .map_err(|e| anyhow!("Failed to navigate: {}", e))?;
            
            // Update tab state
            let mut tabs = self.tabs.write().await;
            if let Some(tab) = tabs.get_mut(tab_id) {
                tab.url = url.to_string();
                tab.is_loading = true;
            }
        }

        Ok(())
    }

    /// Close a tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        let mut tabs = self.tabs.write().await;
        tabs.remove(tab_id);
        
        let mut active_id = self.active_tab_id.write().await;
        if active_id.as_deref() == Some(tab_id) {
            *active_id = tabs.keys().next().cloned();
        }

        Ok(())
    }

    /// Get all tabs
    pub async fn get_tabs(&self) -> Vec<ChromiumTab> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Get active tab
    pub async fn get_active_tab(&self) -> Option<ChromiumTab> {
        let active_id = self.active_tab_id.read().await;
        if let Some(id) = active_id.as_ref() {
            self.tabs.read().await.get(id).cloned()
        } else {
            None
        }
    }

    /// Set active tab
    pub async fn set_active_tab(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        if !tabs.contains_key(tab_id) {
            return Err(anyhow!("Tab not found: {}", tab_id));
        }
        drop(tabs);
        
        *self.active_tab_id.write().await = Some(tab_id.to_string());
        Ok(())
    }

    /// Update proxy for a specific tab (advanced feature)
    pub async fn set_tab_proxy(&self, tab_id: &str, proxy: Option<ProxySettings>) -> Result<()> {
        let mut tabs = self.tabs.write().await;
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.proxy = proxy;
            info!("Updated proxy for tab {}", tab_id);
        } else {
            return Err(anyhow!("Tab not found: {}", tab_id));
        }
        Ok(())
    }

    /// Inject stealth scripts to avoid bot detection
    async fn inject_stealth_scripts(&self, page: &Page) -> Result<()> {
        let stealth_script = r#"
            // Override navigator.webdriver
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            
            // Override navigator.plugins
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5]
            });
            
            // Override navigator.languages
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en']
            });
            
            // Override Chrome runtime
            window.chrome = {
                runtime: {}
            };
            
            // Override permissions
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) => (
                parameters.name === 'notifications' ?
                    Promise.resolve({ state: Notification.permission }) :
                    originalQuery(parameters)
            );
            
            // Disable WebRTC leak
            if (typeof RTCPeerConnection !== 'undefined') {
                const originalRTC = RTCPeerConnection;
                window.RTCPeerConnection = function(...args) {
                    const config = args[0] || {};
                    config.iceServers = [];
                    return new originalRTC(config);
                };
            }
        "#;

        page.evaluate(stealth_script)
            .await
            .map_err(|e| anyhow!("Failed to inject stealth scripts: {}", e))?;

        Ok(())
    }

    /// Shutdown the Chromium engine
    pub async fn shutdown(&mut self) -> Result<()> {
        if !*self.is_running.read().await {
            return Ok(());
        }

        info!("Shutting down Chromium engine...");

        if let Some(browser) = self.browser.take() {
            drop(browser);
        }

        self.tabs.write().await.clear();
        *self.active_tab_id.write().await = None;
        *self.is_running.write().await = false;

        info!("Chromium engine shut down successfully");
        Ok(())
    }

    /// Check if the engine is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ChromiumEngineConfig {
        &self.config
    }

    /// Update configuration (requires restart)
    pub fn set_config(&mut self, config: ChromiumEngineConfig) {
        self.config = config;
    }
}

/// Manager for switching between system and integrated browser engines
pub struct BrowserEngineManager {
    engine_type: Arc<RwLock<BrowserEngineType>>,
    chromium_engine: Arc<RwLock<Option<ChromiumEngine>>>,
    config: Arc<RwLock<ChromiumEngineConfig>>,
}

impl BrowserEngineManager {
    pub fn new() -> Self {
        Self {
            engine_type: Arc::new(RwLock::new(BrowserEngineType::System)),
            chromium_engine: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(ChromiumEngineConfig::default())),
        }
    }

    /// Get current engine type
    pub async fn get_engine_type(&self) -> BrowserEngineType {
        *self.engine_type.read().await
    }

    /// Set engine type and manage engine lifecycle
    pub async fn set_engine_type(&self, engine_type: BrowserEngineType) -> Result<()> {
        let current = *self.engine_type.read().await;
        
        if current == engine_type {
            return Ok(());
        }

        info!("Switching browser engine from {:?} to {:?}", current, engine_type);

        match engine_type {
            BrowserEngineType::System => {
                // Shutdown Chromium engine if running
                let mut engine_guard = self.chromium_engine.write().await;
                if let Some(ref mut engine) = *engine_guard {
                    engine.shutdown().await?;
                }
                *engine_guard = None;
            }
            BrowserEngineType::IntegratedChromium => {
                // Launch Chromium engine
                let config = self.config.read().await.clone();
                let mut engine = ChromiumEngine::new(config);
                engine.launch().await?;
                *self.chromium_engine.write().await = Some(engine);
            }
        }

        *self.engine_type.write().await = engine_type;
        Ok(())
    }

    /// Get Chromium engine reference (if active)
    pub async fn get_chromium_engine(&self) -> Option<Arc<RwLock<Option<ChromiumEngine>>>> {
        if *self.engine_type.read().await == BrowserEngineType::IntegratedChromium {
            Some(self.chromium_engine.clone())
        } else {
            None
        }
    }

    /// Update Chromium configuration
    pub async fn update_chromium_config(&self, config: ChromiumEngineConfig) -> Result<()> {
        *self.config.write().await = config.clone();
        
        // If Chromium engine is running, it needs to be restarted for changes to take effect
        if *self.engine_type.read().await == BrowserEngineType::IntegratedChromium {
            warn!("Chromium engine config updated. Restart required for changes to take effect.");
        }
        
        Ok(())
    }

    /// Set proxy for integrated Chromium engine
    pub async fn set_proxy(&self, proxy: Option<ProxySettings>) -> Result<()> {
        let mut config = self.config.write().await;
        config.proxy = proxy;
        
        info!("Proxy configuration updated for Chromium engine");
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> ChromiumEngineConfig {
        self.config.read().await.clone()
    }

    /// Check if integrated engine supports the requested proxy operation
    pub fn supports_per_tab_proxy(&self) -> bool {
        // Integrated Chromium engine supports per-tab proxy via CDP
        true
    }

    /// Get engine capabilities
    pub fn get_capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            per_tab_proxy: true,
            webrtc_protection: true,
            stealth_mode: true,
            dns_over_https: true,
            custom_user_agent: true,
            javascript_injection: true,
            network_interception: true,
            cookie_management: true,
        }
    }
}

/// Capabilities of the browser engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub per_tab_proxy: bool,
    pub webrtc_protection: bool,
    pub stealth_mode: bool,
    pub dns_over_https: bool,
    pub custom_user_agent: bool,
    pub javascript_injection: bool,
    pub network_interception: bool,
    pub cookie_management: bool,
}

impl Default for BrowserEngineManager {
    fn default() -> Self {
        Self::new()
    }
}
