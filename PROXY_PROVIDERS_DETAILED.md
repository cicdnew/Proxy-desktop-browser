# Detailed Proxy Provider Implementations
## Real-World Free Proxy Provider Integrations

---

## Overview

This document provides detailed implementation guides for integrating 8+ free proxy providers with actual API endpoints, parsing logic, and rate limiting.

---

## Provider 1: ProxyScrape

**API**: https://api.proxyscrape.com/v2/

**Claude Opus 4.5 Prompt:**
```
Implement a ProxyScrape provider for fetching free proxies.

API Documentation:
- Endpoint: GET https://api.proxyscrape.com/v2/
- Parameters:
  - request: "displayproxies" or "getproxies"
  - protocol: "http", "socks4", "socks5", or "all"
  - timeout: timeout in milliseconds (1000-10000)
  - country: country code (e.g., "US", "UK", "all")
  - ssl: "yes" or "no" or "all"
  - anonymity: "elite", "anonymous", "transparent", or "all"
  - format: "textplain" or "json"

Example Request:
https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all&format=json

Response Format (JSON):
{
  "proxies": [
    {
      "ip": "1.2.3.4",
      "port": 8080,
      "ip_address": "1.2.3.4",
      "protocol": "http",
      "country_code": "US",
      "anonymity": "elite",
      "ssl": true
    }
  ]
}

IMPLEMENTATION:
```rust
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ProxyScrapeResponse {
    proxies: Vec<ProxyScrapeProxy>,
}

#[derive(Debug, Deserialize)]
struct ProxyScrapeProxy {
    ip: String,
    port: u16,
    protocol: String,
    country_code: Option<String>,
    anonymity: Option<String>,
    ssl: Option<bool>,
}

pub struct ProxyScrapeProvider {
    client: Client,
    base_url: String,
    rate_limit: Duration,
}

impl ProxyScrapeProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: "https://api.proxyscrape.com/v2/".to_string(),
            rate_limit: Duration::from_secs(60), // 1 request per minute
        }
    }
    
    fn build_url(&self, filter: &ProxyFilter) -> String {
        let mut url = format!("{}?request=displayproxies&format=json", self.base_url);
        
        // Protocol
        if let Some(types) = &filter.proxy_types {
            let protocol = if types.len() == 1 {
                match types[0] {
                    ProxyType::HTTP | ProxyType::HTTPS => "http",
                    ProxyType::SOCKS4 => "socks4",
                    ProxyType::SOCKS5 => "socks5",
                }
            } else {
                "all"
            };
            url.push_str(&format!("&protocol={}", protocol));
        } else {
            url.push_str("&protocol=all");
        }
        
        // Country
        if let Some(countries) = &filter.countries {
            if !countries.is_empty() {
                url.push_str(&format!("&country={}", countries.join(",")));
            }
        } else {
            url.push_str("&country=all");
        }
        
        // SSL
        if let Some(https) = filter.https_support {
            url.push_str(&format!("&ssl={}", if https { "yes" } else { "no" }));
        } else {
            url.push_str("&ssl=all");
        }
        
        // Anonymity
        if let Some(levels) = &filter.anonymity_levels {
            let anonymity = if levels.len() == 1 {
                match levels[0] {
                    AnonymityLevel::Elite => "elite",
                    AnonymityLevel::Anonymous => "anonymous",
                    AnonymityLevel::Transparent => "transparent",
                }
            } else {
                "all"
            };
            url.push_str(&format!("&anonymity={}", anonymity));
        } else {
            url.push_str("&anonymity=all");
        }
        
        // Timeout
        if let Some(max_time) = filter.max_response_time_ms {
            url.push_str(&format!("&timeout={}", max_time));
        } else {
            url.push_str("&timeout=10000");
        }
        
        url
    }
}

#[async_trait]
impl ProxyProvider for ProxyScrapeProvider {
    fn name(&self) -> &str {
        "ProxyScrape"
    }
    
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>> {
        self.fetch_proxies_filtered(&ProxyFilter::default()).await
    }
    
    async fn fetch_proxies_filtered(&self, filter: &ProxyFilter) -> Result<Vec<ProxyConfig>> {
        let url = self.build_url(filter);
        
        info!("Fetching proxies from ProxyScrape: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;
        
        let data: ProxyScrapeResponse = response.json().await?;
        
        let proxies = data.proxies
            .into_iter()
            .map(|p| {
                let proxy_type = match p.protocol.as_str() {
                    "http" | "https" => if p.ssl.unwrap_or(false) {
                        ProxyType::HTTPS
                    } else {
                        ProxyType::HTTP
                    },
                    "socks4" => ProxyType::SOCKS4,
                    "socks5" => ProxyType::SOCKS5,
                    _ => ProxyType::HTTP,
                };
                
                ProxyConfig {
                    id: format!("proxyscrape_{}_{}", p.ip, p.port).into(),
                    proxy_type,
                    host: p.ip,
                    port: p.port,
                    username: None,
                    password: None,
                    enabled: true,
                    countries: p.country_code.into_iter().collect(),
                    speed_mbps: None,
                    uptime_percent: None,
                    last_checked: None,
                }
            })
            .collect();
        
        info!("Fetched {} proxies from ProxyScrape", proxies.len());
        
        Ok(proxies)
    }
    
    fn rate_limit(&self) -> Duration {
        self.rate_limit
    }
    
    fn requires_api_key(&self) -> bool {
        false
    }
}
```

Implement with proper error handling and logging.
```

---

## Provider 2: PubProxy

**API**: http://pubproxy.com/api/proxy

**Claude Opus 4.5 Prompt:**
```
Implement a PubProxy provider for fetching free proxies.

API Documentation:
- Endpoint: GET http://pubproxy.com/api/proxy
- Parameters:
  - limit: number of proxies (1-20)
  - format: "json" or "txt"
  - type: "http", "socks4", "socks5"
  - level: "anonymous" or "elite"
  - country: country code (e.g., "US", "UK")
  - last_check: max minutes since last check (1-1000)
  - speed: max speed in seconds (1-60)
  - not_country: exclude country code

Example Request:
http://pubproxy.com/api/proxy?limit=20&format=json&type=http&level=elite&last_check=50

Response Format (JSON):
{
  "data": [
    {
      "ipPort": "1.2.3.4:8080",
      "ip": "1.2.3.4",
      "port": "8080",
      "country": "US",
      "type": "http",
      "speed": "2.5",
      "support": {
        "https": 1,
        "get": 1,
        "post": 1,
        "cookies": 1,
        "referer": 1,
        "user_agent": 1
      }
    }
  ],
  "count": 1
}

IMPLEMENTATION:
```rust
#[derive(Debug, Deserialize)]
struct PubProxyResponse {
    data: Vec<PubProxyItem>,
    count: u32,
}

#[derive(Debug, Deserialize)]
struct PubProxyItem {
    ip: String,
    port: String,
    country: Option<String>,
    #[serde(rename = "type")]
    proxy_type: String,
    speed: Option<String>,
    support: Option<PubProxySupport>,
}

#[derive(Debug, Deserialize)]
struct PubProxySupport {
    https: Option<u8>,
}

pub struct PubProxyProvider {
    client: Client,
    base_url: String,
    rate_limit: Duration,
}

impl PubProxyProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: "http://pubproxy.com/api/proxy".to_string(),
            rate_limit: Duration::from_secs(5), // 1 request per 5 seconds
        }
    }
    
    fn build_url(&self, filter: &ProxyFilter) -> String {
        let mut url = format!("{}?format=json&limit=20", self.base_url);
        
        // Proxy type
        if let Some(types) = &filter.proxy_types {
            if types.len() == 1 {
                let type_str = match types[0] {
                    ProxyType::HTTP | ProxyType::HTTPS => "http",
                    ProxyType::SOCKS4 => "socks4",
                    ProxyType::SOCKS5 => "socks5",
                };
                url.push_str(&format!("&type={}", type_str));
            }
        }
        
        // Anonymity level
        if let Some(levels) = &filter.anonymity_levels {
            if levels.contains(&AnonymityLevel::Elite) {
                url.push_str("&level=elite");
            } else if levels.contains(&AnonymityLevel::Anonymous) {
                url.push_str("&level=anonymous");
            }
        }
        
        // Country
        if let Some(countries) = &filter.countries {
            if countries.len() == 1 {
                url.push_str(&format!("&country={}", countries[0]));
            }
        }
        
        // Speed filter
        if let Some(max_time) = filter.max_response_time_ms {
            let speed_seconds = max_time / 1000;
            url.push_str(&format!("&speed={}", speed_seconds.max(1).min(60)));
        }
        
        // Last check (only recently validated proxies)
        url.push_str("&last_check=50");
        
        url
    }
}

#[async_trait]
impl ProxyProvider for PubProxyProvider {
    fn name(&self) -> &str {
        "PubProxy"
    }
    
    async fn fetch_proxies_filtered(&self, filter: &ProxyFilter) -> Result<Vec<ProxyConfig>> {
        let url = self.build_url(filter);
        
        info!("Fetching proxies from PubProxy: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;
        
        let data: PubProxyResponse = response.json().await?;
        
        let proxies = data.data
            .into_iter()
            .filter_map(|p| {
                let port = p.port.parse::<u16>().ok()?;
                
                let proxy_type = match p.proxy_type.as_str() {
                    "http" => {
                        if p.support.as_ref().and_then(|s| s.https) == Some(1) {
                            ProxyType::HTTPS
                        } else {
                            ProxyType::HTTP
                        }
                    }
                    "socks4" => ProxyType::SOCKS4,
                    "socks5" => ProxyType::SOCKS5,
                    _ => return None,
                };
                
                Some(ProxyConfig {
                    id: format!("pubproxy_{}_{}", p.ip, port).into(),
                    proxy_type,
                    host: p.ip,
                    port,
                    username: None,
                    password: None,
                    enabled: true,
                    countries: p.country.into_iter().collect(),
                    speed_mbps: None,
                    uptime_percent: None,
                    last_checked: None,
                })
            })
            .collect();
        
        info!("Fetched {} proxies from PubProxy", proxies.len());
        
        Ok(proxies)
    }
    
    fn rate_limit(&self) -> Duration {
        self.rate_limit
    }
    
    fn requires_api_key(&self) -> bool {
        false
    }
}
```

Implement with proper error handling and retry logic.
```

---

## Provider 3: Free Proxy List (Web Scraping)

**Website**: https://free-proxy-list.net/

**Claude Opus 4.5 Prompt:**
```
Implement a Free Proxy List provider using web scraping (since they don't have an API).

IMPORTANT: Web scraping should be done respectfully:
- Add User-Agent header
- Respect rate limits (1 request per minute)
- Cache results
- Handle HTML parsing errors gracefully

Website Structure:
- Proxies are in a table with class "table table-striped table-bordered"
- Columns: IP Address, Port, Code (country), Country, Anonymity, Google, Https, Last Checked

IMPLEMENTATION:
```rust
use scraper::{Html, Selector};

pub struct FreeProxyListProvider {
    client: Client,
    base_url: String,
    rate_limit: Duration,
}

impl FreeProxyListProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .unwrap(),
            base_url: "https://free-proxy-list.net/".to_string(),
            rate_limit: Duration::from_secs(60), // Be respectful
        }
    }
    
    async fn scrape_proxies(&self) -> Result<Vec<ProxyConfig>> {
        info!("Scraping proxies from Free Proxy List");
        
        let html = self.client
            .get(&self.base_url)
            .send()
            .await?
            .text()
            .await?;
        
        let document = Html::parse_document(&html);
        let table_selector = Selector::parse("table.table").unwrap();
        let row_selector = Selector::parse("tbody tr").unwrap();
        let cell_selector = Selector::parse("td").unwrap();
        
        let mut proxies = Vec::new();
        
        if let Some(table) = document.select(&table_selector).next() {
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();
                
                if cells.len() >= 7 {
                    let ip = cells[0].text().collect::<String>().trim().to_string();
                    let port = cells[1].text().collect::<String>().trim();
                    let country_code = cells[2].text().collect::<String>().trim().to_string();
                    let anonymity = cells[4].text().collect::<String>().trim().to_lowercase();
                    let https = cells[6].text().collect::<String>().trim().to_lowercase();
                    
                    if let Ok(port) = port.parse::<u16>() {
                        let anonymity_level = match anonymity.as_str() {
                            "elite proxy" => AnonymityLevel::Elite,
                            "anonymous" => AnonymityLevel::Anonymous,
                            _ => AnonymityLevel::Transparent,
                        };
                        
                        let proxy_type = if https == "yes" {
                            ProxyType::HTTPS
                        } else {
                            ProxyType::HTTP
                        };
                        
                        proxies.push(ProxyConfig {
                            id: format!("freeproxylist_{}_{}", ip, port).into(),
                            proxy_type,
                            host: ip,
                            port,
                            username: None,
                            password: None,
                            enabled: true,
                            countries: vec![country_code],
                            speed_mbps: None,
                            uptime_percent: None,
                            last_checked: None,
                        });
                    }
                }
            }
        }
        
        info!("Scraped {} proxies from Free Proxy List", proxies.len());
        
        Ok(proxies)
    }
}

#[async_trait]
impl ProxyProvider for FreeProxyListProvider {
    fn name(&self) -> &str {
        "FreeProxyList"
    }
    
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>> {
        self.scrape_proxies().await
    }
    
    async fn fetch_proxies_filtered(&self, filter: &ProxyFilter) -> Result<Vec<ProxyConfig>> {
        let all_proxies = self.scrape_proxies().await?;
        
        // Filter locally since this provider doesn't support API filtering
        Ok(all_proxies
            .into_iter()
            .filter(|p| {
                // Filter by country
                if let Some(countries) = &filter.countries {
                    if !countries.is_empty() && !p.countries.iter().any(|c| countries.contains(c)) {
                        return false;
                    }
                }
                
                // Filter by proxy type
                if let Some(types) = &filter.proxy_types {
                    if !types.contains(&p.proxy_type) {
                        return false;
                    }
                }
                
                // Filter by HTTPS support
                if let Some(https) = filter.https_support {
                    if https && p.proxy_type != ProxyType::HTTPS {
                        return false;
                    }
                }
                
                true
            })
            .collect())
    }
    
    fn rate_limit(&self) -> Duration {
        self.rate_limit
    }
    
    fn requires_api_key(&self) -> bool {
        false
    }
}
```

Add to Cargo.toml:
```toml
[dependencies]
scraper = "0.17"
```

Implement with robust HTML parsing and error handling.
```

---

## Provider Manager Integration

**Claude Opus 4.5 Prompt:**
```
Now integrate all providers into a unified ProxyProviderManager that:
1. Manages multiple providers
2. Handles rate limiting per provider
3. Deduplicates proxies across providers
4. Schedules automatic updates
5. Validates fetched proxies before adding to pool

IMPLEMENTATION:
```rust
use std::collections::{HashMap, HashSet};
use tokio::time::{interval, Duration};

pub struct ProxyProviderManager {
    providers: Vec<Box<dyn ProxyProvider>>,
    last_fetch: HashMap<String, DateTime<Utc>>,
    proxy_pool: Arc<Mutex<Vec<ProxyConfig>>>,
    update_interval: Duration,
    validator: ProxyValidator,
    database: Arc<ProxyDatabase>,
}

impl ProxyProviderManager {
    pub fn new(database: Arc<ProxyDatabase>) -> Self {
        let mut providers: Vec<Box<dyn ProxyProvider>> = Vec::new();
        
        // Add all providers
        providers.push(Box::new(ProxyScrapeProvider::new()));
        providers.push(Box::new(PubProxyProvider::new()));
        providers.push(Box::new(FreeProxyListProvider::new()));
        // Add more providers here
        
        Self {
            providers,
            last_fetch: HashMap::new(),
            proxy_pool: Arc::new(Mutex::new(Vec::new())),
            update_interval: Duration::from_secs(1800), // 30 minutes
            validator: ProxyValidator::new(),
            database,
        }
    }
    
    pub async fn fetch_all_proxies(&mut self, filter: Option<ProxyFilter>) -> Result<Vec<ProxyConfig>> {
        let mut all_proxies = Vec::new();
        let mut seen_hosts = HashSet::new();
        
        for provider in &self.providers {
            let provider_name = provider.name().to_string();
            
            // Check rate limit
            if let Some(last_fetch) = self.last_fetch.get(&provider_name) {
                let elapsed = Utc::now().signed_duration_since(*last_fetch);
                let rate_limit = provider.rate_limit();
                
                if elapsed < chrono::Duration::from_std(rate_limit).unwrap() {
                    info!("Skipping {} due to rate limit", provider_name);
                    continue;
                }
            }
            
            // Fetch proxies
            info!("Fetching proxies from {}", provider_name);
            
            match if let Some(ref f) = filter {
                provider.fetch_proxies_filtered(f).await
            } else {
                provider.fetch_proxies().await
            } {
                Ok(proxies) => {
                    info!("Fetched {} proxies from {}", proxies.len(), provider_name);
                    
                    // Deduplicate
                    for proxy in proxies {
                        let key = format!("{}:{}", proxy.host, proxy.port);
                        if !seen_hosts.contains(&key) {
                            seen_hosts.insert(key);
                            all_proxies.push(proxy);
                        }
                    }
                    
                    self.last_fetch.insert(provider_name, Utc::now());
                }
                Err(e) => {
                    error!("Failed to fetch from {}: {}", provider_name, e);
                }
            }
            
            // Small delay between providers
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        
        info!("Fetched total of {} unique proxies", all_proxies.len());
        
        Ok(all_proxies)
    }
    
    pub async fn fetch_and_validate(&mut self, filter: Option<ProxyFilter>) -> Result<Vec<ProxyConfig>> {
        let proxies = self.fetch_all_proxies(filter).await?;
        
        info!("Validating {} proxies...", proxies.len());
        
        // Validate in batches
        let validated = self.validator.validate_batch(&proxies, 10).await?;
        
        let working_proxies: Vec<_> = validated
            .into_iter()
            .filter(|(_, result)| result.is_working)
            .map(|(proxy, _)| proxy)
            .collect();
        
        info!("{} proxies are working", working_proxies.len());
        
        // Save to database
        for proxy in &working_proxies {
            if let Err(e) = self.database.insert_proxy(proxy).await {
                error!("Failed to save proxy to database: {}", e);
            }
        }
        
        // Update pool
        *self.proxy_pool.lock().unwrap() = working_proxies.clone();
        
        Ok(working_proxies)
    }
    
    pub async fn start_auto_update(&self) {
        let mut interval = interval(self.update_interval);
        let manager = self.clone(); // Implement Clone or use Arc
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                info!("Auto-updating proxy pool...");
                
                if let Err(e) = manager.fetch_and_validate(None).await {
                    error!("Failed to auto-update proxies: {}", e);
                }
            }
        });
    }
    
    pub fn get_proxy_pool(&self) -> Vec<ProxyConfig> {
        self.proxy_pool.lock().unwrap().clone()
    }
}
```

Implement with comprehensive error handling and logging.
```

---

## Testing the Providers

**Claude Opus 4.5 Prompt:**
```
Create comprehensive tests for the proxy provider system.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_proxyscrape_provider() {
        let provider = ProxyScrapeProvider::new();
        let proxies = provider.fetch_proxies().await;
        
        assert!(proxies.is_ok());
        let proxies = proxies.unwrap();
        assert!(!proxies.is_empty());
        
        // Verify structure
        for proxy in proxies.iter().take(5) {
            assert!(!proxy.host.is_empty());
            assert!(proxy.port > 0);
        }
    }
    
    #[tokio::test]
    async fn test_pubproxy_provider() {
        let provider = PubProxyProvider::new();
        let proxies = provider.fetch_proxies().await;
        
        assert!(proxies.is_ok());
        // PubProxy returns 1-20 proxies per request
        assert!(proxies.unwrap().len() <= 20);
    }
    
    #[tokio::test]
    async fn test_provider_filtering() {
        let provider = ProxyScrapeProvider::new();
        
        let filter = ProxyFilter {
            countries: Some(vec!["US".to_string()]),
            proxy_types: Some(vec![ProxyType::HTTP]),
            anonymity_levels: Some(vec![AnonymityLevel::Elite]),
            https_support: Some(true),
            ..Default::default()
        };
        
        let proxies = provider.fetch_proxies_filtered(&filter).await.unwrap();
        
        for proxy in &proxies {
            assert_eq!(proxy.proxy_type, ProxyType::HTTPS);
            if !proxy.countries.is_empty() {
                assert_eq!(proxy.countries[0], "US");
            }
        }
    }
    
    #[tokio::test]
    async fn test_provider_manager_deduplication() {
        let db = Arc::new(ProxyDatabase::new(":memory:").await.unwrap());
        let mut manager = ProxyProviderManager::new(db);
        
        let proxies = manager.fetch_all_proxies(None).await.unwrap();
        
        // Check for duplicates
        let mut seen = HashSet::new();
        for proxy in &proxies {
            let key = format!("{}:{}", proxy.host, proxy.port);
            assert!(!seen.contains(&key), "Duplicate proxy found: {}", key);
            seen.insert(key);
        }
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let provider = PubProxyProvider::new();
        
        // First request should succeed
        let result1 = provider.fetch_proxies().await;
        assert!(result1.is_ok());
        
        // Immediate second request should be rate limited
        // (In real implementation, this would be handled by the manager)
    }
}
```

Run tests:
```bash
cargo test --package browser-core -- test_proxy --nocapture
```
```

---

## Additional Providers (Quick Reference)

### Provider 4: Geonode
- API: https://proxylist.geonode.com/api/proxy-list
- Rate Limit: 100 requests/day (free tier)
- Response: JSON with detailed proxy info

### Provider 5: ProxyNova (Scraping Required)
- Website: https://www.proxynova.com/proxy-server-list/
- Rate Limit: Be respectful
- Structure: HTML table

### Provider 6: Spys.one (Scraping Required)
- Website: http://spys.one/en/
- Complex JavaScript-based obfuscation
- Consider using headless browser

### Provider 7: HideMyName
- Website: https://hidemy.name/en/proxy-list/
- API: Premium only
- Scraping: Possible but rate-limited

### Provider 8: ProxyDB
- Website: http://proxydb.net/
- API: Available with free tier
- Rate Limit: 100 requests/hour

---

## Best Practices

1. **Always respect rate limits**
2. **Cache proxy lists locally**
3. **Validate before adding to pool**
4. **Handle provider failures gracefully**
5. **Log all fetch operations**
6. **Deduplicate across providers**
7. **Schedule updates during off-peak hours**
8. **Monitor provider uptime**

