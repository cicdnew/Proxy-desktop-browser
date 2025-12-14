use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::chromium_engine::BrowserEngineType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub tab_id: String,
    pub current_url: String,
    pub title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub history: Vec<HistoryItem>,
    pub history_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub url: String,
    pub title: String,
    pub timestamp: i64,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            tab_id: String::new(),
            current_url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            history: Vec::new(),
            history_index: -1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub user_agent: String,
    pub language: String,
    pub timezone: String,
    pub webrtc_policy: WebRtcPolicy,
    pub dns_over_https: bool,
    pub block_trackers: bool,
    pub block_ads: bool,
    pub javascript_enabled: bool,
    pub cookies_enabled: bool,
    /// Browser engine type: System (Tauri webview) or IntegratedChromium
    pub engine_type: BrowserEngineType,
    /// Enable stealth mode for integrated Chromium engine
    pub stealth_mode: bool,
    /// Run integrated Chromium in headless mode
    pub headless_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebRtcPolicy {
    Default,
    DisableNonProxiedUdp,
    Disabled,
}

impl Default for WebRtcPolicy {
    fn default() -> Self {
        WebRtcPolicy::DisableNonProxiedUdp
    }
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            language: "en-US".to_string(),
            timezone: "America/New_York".to_string(),
            webrtc_policy: WebRtcPolicy::DisableNonProxiedUdp,
            dns_over_https: true,
            block_trackers: true,
            block_ads: false,
            javascript_enabled: true,
            cookies_enabled: true,
            engine_type: BrowserEngineType::System,
            stealth_mode: true,
            headless_mode: false,
        }
    }
}

pub struct BrowserController {
    states: Arc<RwLock<HashMap<String, BrowserState>>>,
    settings: Arc<RwLock<BrowserSettings>>,
}

impl BrowserController {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(BrowserSettings::default())),
        }
    }

    pub async fn create_browser_state(&self, tab_id: &str) -> BrowserState {
        let state = BrowserState {
            tab_id: tab_id.to_string(),
            ..Default::default()
        };
        self.states.write().await.insert(tab_id.to_string(), state.clone());
        state
    }

    pub async fn get_state(&self, tab_id: &str) -> Option<BrowserState> {
        self.states.read().await.get(tab_id).cloned()
    }

    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<BrowserState> {
        let mut states = self.states.write().await;
        let state = states.entry(tab_id.to_string()).or_insert_with(|| BrowserState {
            tab_id: tab_id.to_string(),
            ..Default::default()
        });

        // Trim history after current index
        if state.history_index >= 0 {
            state.history.truncate((state.history_index + 1) as usize);
        }

        // Add to history
        state.history.push(HistoryItem {
            url: url.to_string(),
            title: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        state.history_index = (state.history.len() - 1) as i32;

        state.current_url = url.to_string();
        state.can_go_back = state.history_index > 0;
        state.can_go_forward = false;
        state.is_loading = true;

        Ok(state.clone())
    }

    pub async fn go_back(&self, tab_id: &str) -> Result<Option<String>> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tab_id) {
            if state.history_index > 0 {
                state.history_index -= 1;
                let url = state.history[state.history_index as usize].url.clone();
                state.current_url = url.clone();
                state.can_go_back = state.history_index > 0;
                state.can_go_forward = true;
                return Ok(Some(url));
            }
        }
        Ok(None)
    }

    pub async fn go_forward(&self, tab_id: &str) -> Result<Option<String>> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tab_id) {
            if (state.history_index as usize) < state.history.len() - 1 {
                state.history_index += 1;
                let url = state.history[state.history_index as usize].url.clone();
                state.current_url = url.clone();
                state.can_go_back = true;
                state.can_go_forward = (state.history_index as usize) < state.history.len() - 1;
                return Ok(Some(url));
            }
        }
        Ok(None)
    }

    pub async fn reload(&self, tab_id: &str) -> Result<Option<String>> {
        let states = self.states.read().await;
        if let Some(state) = states.get(tab_id) {
            return Ok(Some(state.current_url.clone()));
        }
        Ok(None)
    }

    pub async fn stop_loading(&self, tab_id: &str) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tab_id) {
            state.is_loading = false;
        }
    }

    pub async fn update_title(&self, tab_id: &str, title: &str) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tab_id) {
            state.title = title.to_string();
            // Update history item title
            if state.history_index >= 0 && (state.history_index as usize) < state.history.len() {
                state.history[state.history_index as usize].title = title.to_string();
            }
        }
    }

    pub async fn set_loading(&self, tab_id: &str, loading: bool) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tab_id) {
            state.is_loading = loading;
        }
    }

    pub async fn close_tab(&self, tab_id: &str) {
        self.states.write().await.remove(tab_id);
    }

    pub async fn get_settings(&self) -> BrowserSettings {
        self.settings.read().await.clone()
    }

    pub async fn set_settings(&self, settings: BrowserSettings) {
        *self.settings.write().await = settings;
    }

    pub async fn get_all_states(&self) -> Vec<BrowserState> {
        self.states.read().await.values().cloned().collect()
    }
}

impl Default for BrowserController {
    fn default() -> Self {
        Self::new()
    }
}
