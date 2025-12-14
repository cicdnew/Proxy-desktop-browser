use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::proxy::{FreeProxy, ProxyType};
use crate::free_ip_providers::FreeIpProviderManager;

/// Manages proxy rotation strategies for browser tabs.
pub struct ProxyRotationManager {
    provider_manager: Arc<RwLock<FreeIpProviderManager>>,
    active_proxies: Arc<RwLock<HashMap<String, ProxySession>>>,
    strategy: ProxyRotationStrategy,
    performance_metrics: Arc<RwLock<HashMap<String, ProxyMetrics>>>,
}

#[derive(Clone)]
pub struct ProxySession {
    pub proxy: FreeProxy,
    pub assigned_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub request_count: usize,
    pub tab_id: String,
    pub domain_proxy_map: HashMap<String, String>, // domain -> proxy_id
}

#[derive(Debug, Clone)]
pub enum ProxyRotationStrategy {
    /// Rotate proxy after N requests
    PerRequest(usize),
    /// Rotate proxy after time duration
    PerDuration(Duration),
    /// Rotate proxy per session (never during session)
    PerSession,
    /// Random rotation (probabilistic)
    Random { probability: f64 },
    /// Sticky proxy (same proxy for same target domain)
    Sticky { duration: Duration },
    /// Geographic distribution (rotate within country/region)
    Geographic { country_codes: Vec<String> },
    /// Performance-based (use fastest proxies)
    PerformanceBased,
    /// Round-robin rotation
    RoundRobin,
    /// Domain-based (different proxy per domain)
    DomainBased,
    /// Manual rotation (user-triggered)
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMetrics {
    pub response_time_ms: f64,
    pub success_rate: f64,
    pub last_success: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
    pub total_requests: u32,
    pub failed_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySessionStats {
    pub tab_id: String,
    pub current_proxy_ip: String,
    pub proxy_country: String,
    pub assigned_at: DateTime<Utc>,
    pub request_count: usize,
    pub duration_seconds: i64,
}

impl ProxyRotationManager {
    pub fn new(
        provider_manager: Arc<RwLock<FreeIpProviderManager>>,
        strategy: ProxyRotationStrategy,
    ) -> Self {
        Self {
            provider_manager,
            active_proxies: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or rotate proxy for tab
    pub async fn get_proxy_for_tab(&self, tab_id: &str, domain: Option<&str>) -> Result<FreeProxy> {
        let mut sessions = self.active_proxies.write().await;

        if let Some(session) = sessions.get_mut(tab_id) {
            // Check if we need domain-based proxy
            if let (Some(domain), ProxyRotationStrategy::DomainBased) = (domain, &self.strategy) {
                let proxy_key = format!("{}:{}", tab_id, domain);
                if let Some(proxy_id) = session.domain_proxy_map.get(domain) {
                    // Use the same proxy for this domain
                    if let Some(proxy) = self.get_proxy_by_id(proxy_id).await? {
                        session.last_used = Utc::now();
                        session.request_count += 1;
                        return Ok(proxy);
                    }
                }
                // Get a new proxy for this domain
                let new_proxy = self.get_next_proxy(session).await?;
                session.domain_proxy_map.insert(domain.to_string(), new_proxy.ip.clone());
                session.last_used = Utc::now();
                session.request_count += 1;
                return Ok(new_proxy);
            }

            if self.should_rotate(session).await {
                let new_proxy = self.get_next_proxy(session).await?;
                session.proxy = new_proxy.clone();
                session.assigned_at = Utc::now();
                session.last_used = Utc::now();
                session.request_count = 1;
                
                // Clear domain map when rotating
                if matches!(self.strategy, ProxyRotationStrategy::DomainBased) {
                    session.domain_proxy_map.clear();
                }
                
                debug!("Rotated proxy for tab {}: {} -> {}", tab_id, session.proxy.ip, new_proxy.ip);
                return Ok(new_proxy);
            } else {
                session.last_used = Utc::now();
                session.request_count += 1;
                return Ok(session.proxy.clone());
            }
        }

        // Create new session with new proxy
        let proxy = self.get_initial_proxy().await?;
        let session = ProxySession {
            proxy: proxy.clone(),
            assigned_at: Utc::now(),
            last_used: Utc::now(),
            request_count: 1,
            tab_id: tab_id.to_string(),
            domain_proxy_map: HashMap::new(),
        };

        sessions.insert(tab_id.to_string(), session);
        info!("Assigned initial proxy {} to tab {}", proxy.ip, tab_id);
        Ok(proxy)
    }

    /// Manually rotate proxy for tab
    pub async fn force_rotate(&self, tab_id: &str) -> Result<FreeProxy> {
        let mut sessions = self.active_proxies.write().await;
        if let Some(session) = sessions.get_mut(tab_id) {
            let new_proxy = self.get_next_proxy(session).await?;
            session.proxy = new_proxy.clone();
            session.assigned_at = Utc::now();
            session.last_used = Utc::now();
            session.request_count = 0;
            session.domain_proxy_map.clear();
            
            info!("Force rotated proxy for tab {}: {} -> {}", tab_id, session.proxy.ip, new_proxy.ip);
            Ok(new_proxy)
        } else {
            Err(anyhow!("Tab session not found"))
        }
    }

    /// Record proxy performance metrics
    pub async fn record_performance(&self, proxy_id: &str, success: bool, response_time_ms: Option<f64>) {
        let mut metrics = self.performance_metrics.write().await;
        let metric = metrics.entry(proxy_id.to_string()).or_insert(ProxyMetrics {
            response_time_ms: 0.0,
            success_rate: 0.0,
            last_success: None,
            consecutive_failures: 0,
            total_requests: 0,
            failed_requests: 0,
        });

        metric.total_requests += 1;
        if success {
            metric.failed_requests = 0;
            metric.consecutive_failures = 0;
            metric.last_success = Some(Utc::now());
            if let Some(rt) = response_time_ms {
                metric.response_time_ms = (metric.response_time_ms * 0.9) + (rt * 0.1); // EMA
            }
        } else {
            metric.failed_requests += 1;
            metric.consecutive_failures += 1;
        }

        metric.success_rate = (metric.total_requests - metric.failed_requests) as f64 / metric.total_requests as f64;
    }

    /// Get current proxy for tab
    pub async fn get_current_proxy(&self, tab_id: &str) -> Option<FreeProxy> {
        let sessions = self.active_proxies.read().await;
        sessions.get(tab_id).map(|s| s.proxy.clone())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, tab_id: &str) -> Option<ProxySessionStats> {
        let sessions = self.active_proxies.read().await;
        sessions.get(tab_id).map(|s| ProxySessionStats {
            tab_id: s.tab_id.clone(),
            current_proxy_ip: s.proxy.ip.clone(),
            proxy_country: s.proxy.country.clone(),
            assigned_at: s.assigned_at,
            request_count: s.request_count,
            duration_seconds: (Utc::now() - s.assigned_at).num_seconds(),
        })
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self, max_age: Duration) {
        let mut sessions = self.active_proxies.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();
        sessions.retain(|_, session| now - session.last_used < max_age);
        let removed = initial_count - sessions.len();
        
        if removed > 0 {
            info!("Cleaned up {} expired proxy sessions", removed);
        }
    }

    /// Update rotation strategy
    pub async fn update_strategy(&mut self, strategy: ProxyRotationStrategy) {
        info!("Updating proxy rotation strategy to {:?}", strategy);
        self.strategy = strategy;
    }

    async fn should_rotate(&self, session: &ProxySession) -> bool {
        match &self.strategy {
            ProxyRotationStrategy::PerRequest(count) => session.request_count >= *count,
            ProxyRotationStrategy::PerDuration(duration) => {
                let elapsed = Utc::now() - session.assigned_at;
                elapsed > *duration
            }
            ProxyRotationStrategy::PerSession => false,
            ProxyRotationStrategy::Random { probability } => rand::thread_rng().gen::<f64>() < *probability,
            ProxyRotationStrategy::Sticky { duration } => {
                let elapsed = Utc::now() - session.last_used;
                elapsed > *duration
            }
            ProxyRotationStrategy::Geographic { .. } => false, // Handled in get_next_proxy
            ProxyRotationStrategy::PerformanceBased => {
                // Check if current proxy is underperforming
                if let Some(metric) = self.performance_metrics.read().await.get(&session.proxy.ip) {
                    metric.success_rate < 0.8 || metric.consecutive_failures > 3
                } else {
                    false
                }
            }
            ProxyRotationStrategy::RoundRobin => session.request_count >= 100, // Rotate every 100 requests
            ProxyRotationStrategy::DomainBased => false, // Handled per domain
            ProxyRotationStrategy::Manual => false,
        }
    }

    async fn get_initial_proxy(&self) -> Result<FreeProxy> {
        let provider = self.provider_manager.read().await;
        
        match &self.strategy {
            ProxyRotationStrategy::Geographic { country_codes } => {
                if country_codes.is_empty() {
                    return provider.get_random_working_proxy()
                        .ok_or_else(|| anyhow!("No working proxies available"))
                        .map(|p| p.clone());
                }
                
                let working_proxies = provider.get_working_proxies();
                let country_proxies: Vec<_> = working_proxies
                    .iter()
                    .filter(|p| country_codes.contains(&p.country) || country_codes.contains(&p.country_code))
                    .collect();
                
                if country_proxies.is_empty() {
                    warn!("No proxies found for specified countries, using random working proxy");
                    provider.get_random_working_proxy()
                        .ok_or_else(|| anyhow!("No working proxies available"))
                        .map(|p| p.clone())
                } else {
                    let mut rng = rand::thread_rng();
                    Ok((*country_proxies[rng.gen_range(0..country_proxies.len())]).clone())
                }
            }
            ProxyRotationStrategy::PerformanceBased => {
                let metrics = self.performance_metrics.read().await;
                let working_proxies = provider.get_working_proxies();
                
                if working_proxies.is_empty() {
                    return Err(anyhow!("No working proxies available"));
                }
                
                // Sort by success rate and response time
                let mut sorted_proxies: Vec<_> = working_proxies.iter().collect();
                sorted_proxies.sort_by(|a, b| {
                    let metric_a = metrics.get(&a.ip);
                    let metric_b = metrics.get(&b.ip);
                    
                    match (metric_a, metric_b) {
                        (Some(ma), Some(mb)) => {
                            mb.success_rate.partial_cmp(&ma.success_rate)
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| ma.response_time_ms.partial_cmp(&mb.response_time_ms).unwrap_or(std::cmp::Ordering::Equal))
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
                
                Ok((*sorted_proxies[0]).clone())
            }
            _ => {
                provider.get_random_working_proxy()
                    .ok_or_else(|| anyhow!("No working proxies available"))
                    .map(|p| p.clone())
            }
        }
    }

    async fn get_next_proxy(&self, _current_session: &ProxySession) -> Result<FreeProxy> {
        self.get_initial_proxy().await
    }

    async fn get_proxy_by_id(&self, proxy_id: &str) -> Result<Option<FreeProxy>> {
        let provider = self.provider_manager.read().await;
        Ok(provider.get_proxy_pool()
            .iter()
            .find(|p| p.ip == proxy_id && p.is_working)
            .cloned())
    }
}
