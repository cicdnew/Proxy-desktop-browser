use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn, error};
use std::sync::Arc;

use crate::proxy::{FreeProxy, ProxySettings};
use crate::http_client::HttpClient;

// Internal struct for test results
#[derive(Debug, Clone)]
struct InternalTestResult {
    is_working: bool,
    detected_ip: Option<String>,
    detected_country: Option<String>,
    supports_https: bool,
    has_ip_leak: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_working: bool,
    pub response_time_ms: u64,
    pub detected_country: Option<String>,
    pub detected_ip: Option<String>,
    pub supports_https: bool,
    pub has_ip_leak: bool,
    pub error: Option<String>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProxyValidatorConfig {
    pub timeout: Duration,
    pub concurrent_checks: usize,
    pub test_urls: Vec<String>,
    pub max_retries: u32,
}

impl Default for ProxyValidatorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            concurrent_checks: 20,
            test_urls: vec![
                "https://api.ipify.org?format=json".to_string(),
                "https://httpbin.org/ip".to_string(),
                "https://jsonip.com".to_string(),
            ],
            max_retries: 3,
        }
    }
}

pub struct ProxyValidator {
    config: ProxyValidatorConfig,
    semaphore: Arc<Semaphore>,
}

impl ProxyValidator {
    pub fn new(config: ProxyValidatorConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.concurrent_checks)),
            config,
        }
    }

    pub async fn validate_proxy(&self, proxy: &FreeProxy) -> Result<ValidationResult> {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| anyhow!("Failed to acquire semaphore: {}", e))?;

        let settings = proxy.to_proxy_settings();
        
        info!("Validating proxy {}:{}", proxy.ip, proxy.port);
        
        let mut last_error = None;
        let mut best_result = None;
        
        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {} for proxy {}:{}", attempt + 1, proxy.ip, proxy.port);
            }
            
            match self.validate_single_attempt(&settings, &proxy).await {
                Ok(result) => {
                    if result.is_working {
                        best_result = Some(result);
                        break;
                    } else {
                        last_error = result.error.clone();
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
            
            // Wait before retry
            if attempt < self.config.max_retries - 1 {
                tokio::time::sleep(Duration::from_millis(500 * (attempt + 1) as u64)).await;
            }
        }
        
        let result = best_result.unwrap_or_else(|| ValidationResult {
            is_working: false,
            response_time_ms: u64::MAX,
            detected_country: None,
            detected_ip: None,
            supports_https: false,
            has_ip_leak: false,
            error: last_error,
            validated_at: Utc::now(),
        });
        
        if result.is_working {
            info!("Proxy {}:{} is working ({}ms)", proxy.ip, proxy.port, result.response_time_ms);
        } else {
            warn!("Proxy {}:{} failed validation: {:?}", proxy.ip, proxy.port, result.error);
        }
        
        Ok(result)
    }

    async fn validate_single_attempt(&self, settings: &ProxySettings, proxy: &FreeProxy) -> Result<ValidationResult> {
        let start = std::time::Instant::now();
        
        // Create HTTP client with proxy
        let client = HttpClient::with_proxy(settings)?;
        
        // Test basic connectivity
        let test_result = self.test_connectivity(&client, proxy).await?;
        
        let elapsed = start.elapsed();
        
        Ok(ValidationResult {
            is_working: test_result.is_working,
            response_time_ms: elapsed.as_millis() as u64,
            detected_country: test_result.detected_country,
            detected_ip: test_result.detected_ip,
            supports_https: test_result.supports_https,
            has_ip_leak: test_result.has_ip_leak,
            error: test_result.error,
            validated_at: Utc::now(),
        })
    }

    async fn test_connectivity(&self, client: &HttpClient, proxy: &FreeProxy) -> Result<InternalTestResult> {
        // Try each test URL until one succeeds
        for url in &self.config.test_urls {
            match self.test_url(client, url).await {
                Ok(result) => {
                    if result.is_working {
                        return Ok(result);
                    }
                }
                Err(e) => {
                    debug!("Failed to test {} with proxy {}: {}: {}", url, proxy.ip, proxy.port, e);
                }
            }
        }
        
        // All URLs failed
        Ok(InternalTestResult {
            is_working: false,
            detected_ip: None,
            detected_country: None,
            supports_https: false,
            has_ip_leak: false,
            error: Some("All test URLs failed".to_string()),
        })
    }

    async fn test_url(&self, client: &HttpClient, url: &str) -> Result<InternalTestResult> {
        // Make request through proxy
        let response = client.get(url).await?;
        
        // Parse response to detect IP
        #[derive(Deserialize)]
        struct IpResponse {
            ip: String,
            country: Option<String>,
        }
        
        let ip_response: IpResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        
        // Check if IP is different from local IP (simple leak detection)
        let has_ip_leak = self.check_ip_leak(&ip_response.ip).await.unwrap_or(false);
        
        Ok(InternalTestResult {
            is_working: true,
            detected_ip: Some(ip_response.ip.clone()),
            detected_country: ip_response.country,
            supports_https: url.starts_with("https://"),
            has_ip_leak,
            error: None,
        })
    }

    async fn check_ip_leak(&self, detected_ip: &str) -> Result<bool> {
        // Get local IP without proxy
        let local_client = HttpClient::new()?;
        let local_response = local_client.get("https://api.ipify.org?format=json").await?;
        
        #[derive(Deserialize)]
        struct LocalIpResponse {
            ip: String,
        }
        
        let local_ip: LocalIpResponse = serde_json::from_str(&local_response)?;
        
        Ok(local_ip.ip == detected_ip)
    }

    pub async fn validate_batch(&self, proxies: &[FreeProxy]) -> Vec<(FreeProxy, ValidationResult)> {
        let mut results = Vec::new();
        
        // Process proxies in parallel with semaphore limiting
        let mut tasks = Vec::new();
        
        for proxy in proxies {
            let proxy = proxy.clone();
            let validator = ProxyValidator::new(self.config.clone());
            
            let task = tokio::spawn(async move {
                let result = validator.validate_proxy(&proxy).await;
                (proxy, result)
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok((proxy, Ok(result))) => {
                    results.push((proxy, result));
                }
                Ok((proxy, Err(e))) => {
                    error!("Failed to validate proxy: {}", e);
                    results.push((proxy, ValidationResult {
                        is_working: false,
                        response_time_ms: u64::MAX,
                        detected_country: None,
                        detected_ip: None,
                        supports_https: false,
                        has_ip_leak: false,
                        error: Some(e.to_string()),
                        validated_at: Utc::now(),
                    }));
                }
                Err(e) => {
                    error!("Task failed: {}", e);
                }
            }
        }
        
        results
    }
}

#[allow(dead_code)]
pub struct ProxyHealthChecker {
    validator: ProxyValidator,
    check_interval: Duration,
    max_consecutive_failures: u32,
    quarantine_duration: Duration,
}

impl ProxyHealthChecker {
    pub fn new(
        validator: ProxyValidator,
        check_interval: Duration,
        max_consecutive_failures: u32,
        quarantine_duration: Duration,
    ) -> Self {
        Self {
            validator,
            check_interval,
            max_consecutive_failures,
            quarantine_duration,
        }
    }

    pub async fn start_health_monitoring(&self, proxies: Arc<tokio::sync::RwLock<Vec<FreeProxy>>>) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            let proxies_to_check = {
                let proxies = proxies.read().await;
                proxies.clone()
            };
            
            if proxies_to_check.is_empty() {
                continue;
            }
            
            info!("Starting health check for {} proxies", proxies_to_check.len());
            
            let results = self.validator.validate_batch(&proxies_to_check).await;
            
            // Update proxy status based on results
            let mut proxies = proxies.write().await;
            for (proxy, result) in results {
                if let Some(p) = proxies.iter_mut().find(|p| p.ip == proxy.ip && p.port == proxy.port) {
                    p.is_working = result.is_working;
                    p.last_checked = Utc::now().to_rfc3339();
                    
                    if !result.is_working {
                        warn!("Proxy {}:{} marked as unhealthy: {:?}", proxy.ip, proxy.port, result.error);
                    }
                }
            }
            
            let working_count = proxies.iter().filter(|p| p.is_working).count();
            info!("Health check completed: {}/{} proxies working", working_count, proxies.len());
        }
    }
}
