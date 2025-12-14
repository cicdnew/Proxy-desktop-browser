# Updated Prompts - Phase 3: Proxy Providers (Optimized Stack)
## Using sqlx, reqwest-middleware, and scraper

---

## 🎯 Overview

These prompts are updated for:
- **sqlx** (async database with compile-time checks)
- **reqwest-middleware** (auto-retry)
- **scraper** (HTML parsing)
- **validator** (input validation)
- **governor** (rate limiting)

---

## 3.1 ProxyScrape Provider (UPDATED)

**File**: `crates/browser-core/src/providers/proxyscrape.rs`

**Claude Opus 4.5 Prompt:**
```
Implement ProxyScrape API provider with reqwest-middleware for automatic retries.

REQUIREMENTS:

1. Use the HTTP client with middleware:
```rust
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct ProxyScrapeProvider {
    client: ClientWithMiddleware,
    base_url: String,
    rate_limiter: Arc<RateLimiter<DirectRateLimiter>>,
}

impl ProxyScrapeProvider {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self {
            client,
            base_url: "https://api.proxyscrape.com/v2/".to_string(),
            // 1 request per minute (free tier limit)
            rate_limiter: Arc::new(RateLimiter::direct(
                Quota::per_minute(NonZeroU32::new(1).unwrap())
            )),
        }
    }
}
```

2. Implement the trait:
```rust
#[async_trait]
impl ProxyProvider for ProxyScrapeProvider {
    fn name(&self) -> &str {
        "ProxyScrape"
    }
    
    async fn fetch_proxies_filtered(&self, filter: &ProxyFilter) -> Result<Vec<ProxyConfig>> {
        // Wait for rate limit
        self.rate_limiter.until_ready().await;
        
        let url = self.build_url(filter);
        
        tracing::info!("Fetching from ProxyScrape: {}", url);
        
        // Request automatically retries on failure (middleware)
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        let data: ProxyScrapeResponse = response.json().await?;
        
        let proxies = data.proxies
            .into_iter()
            .map(|p| self.convert_to_proxy_config(p))
            .collect();
        
        tracing::info!("Fetched {} proxies from ProxyScrape", proxies.len());
        
        Ok(proxies)
    }
    
    fn rate_limit(&self) -> Duration {
        Duration::from_secs(60)
    }
    
    fn requires_api_key(&self) -> bool {
        false
    }
}
```

3. Response types:
```rust
#[derive(Debug, Deserialize)]
struct ProxyScrapeResponse {
    proxies: Vec<ProxyScrapeProxy>,
}

#[derive(Debug, Deserialize)]
struct ProxyScrapeProxy {
    ip: String,
    port: u16,
    protocol: String,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    anonymity: Option<String>,
    #[serde(default)]
    ssl: Option<bool>,
}
```

4. URL building with validation:
```rust
use validator::{Validate, ValidationError};

fn build_url(&self, filter: &ProxyFilter) -> String {
    let mut params = vec![
        ("request", "displayproxies"),
        ("format", "json"),
        ("timeout", "10000"),
    ];
    
    // Add protocol
    if let Some(types) = &filter.proxy_types {
        let protocol = match types.first() {
            Some(ProxyType::HTTP) | Some(ProxyType::HTTPS) => "http",
            Some(ProxyType::SOCKS4) => "socks4",
            Some(ProxyType::SOCKS5) => "socks5",
            None => "all",
        };
        params.push(("protocol", protocol));
    }
    
    // Add country filter
    if let Some(countries) = &filter.countries {
        if !countries.is_empty() {
            let country = countries.join(",");
            params.push(("country", &country));
        }
    }
    
    // Build URL
    let query = params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    
    format!("{}?{}", self.base_url, query)
}
```

5. Error handling:
```rust
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP request failed")]
    RequestFailed(#[from] reqwest_middleware::Error),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid response format")]
    InvalidResponse(#[from] serde_json::Error),
    
    #[error("No proxies found")]
    NoProxiesFound,
}
```

6. Features:
   - Automatic retries via middleware (no manual retry logic!)
   - Rate limiting with governor
   - Type-safe JSON parsing
   - Input validation
   - Structured logging with tracing

Implement with proper async/await and comprehensive error handling.
```

---

## 3.2 HTML Scraper Provider (NEW)

**File**: `crates/browser-core/src/providers/free_proxy_list.rs`

**Claude Opus 4.5 Prompt:**
```
Implement HTML scraper for Free Proxy List using the scraper crate.

REQUIREMENTS:

1. Use scraper for HTML parsing:
```rust
use scraper::{Html, Selector};
use reqwest_middleware::ClientWithMiddleware;

pub struct FreeProxyListProvider {
    client: ClientWithMiddleware,
    base_url: String,
    rate_limiter: Arc<RateLimiter<DirectRateLimiter>>,
}

impl FreeProxyListProvider {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self {
            client,
            base_url: "https://free-proxy-list.net/".to_string(),
            // Be respectful: 1 request per 2 minutes
            rate_limiter: Arc::new(RateLimiter::direct(
                Quota::per_minute(NonZeroU32::new(1).unwrap())
                    .allow_burst(NonZeroU32::new(1).unwrap())
            )),
        }
    }
}
```

2. Implement scraping:
```rust
#[async_trait]
impl ProxyProvider for FreeProxyListProvider {
    fn name(&self) -> &str {
        "FreeProxyList"
    }
    
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>> {
        // Wait for rate limit
        self.rate_limiter.until_ready().await;
        
        tracing::info!("Scraping Free Proxy List");
        
        // Fetch HTML with retry
        let html = self.client
            .get(&self.base_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?
            .text()
            .await?;
        
        // Parse HTML
        let document = Html::parse_document(&html);
        
        // Select table rows
        let table_selector = Selector::parse("table.table tbody tr")
            .map_err(|_| ProviderError::ParseFailed)?;
        let cell_selector = Selector::parse("td")
            .map_err(|_| ProviderError::ParseFailed)?;
        
        let mut proxies = Vec::new();
        
        for row in document.select(&table_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            if cells.len() >= 8 {
                match self.parse_row(&cells) {
                    Ok(proxy) => proxies.push(proxy),
                    Err(e) => {
                        tracing::warn!("Failed to parse row: {}", e);
                        continue;
                    }
                }
            }
        }
        
        if proxies.is_empty() {
            return Err(ProviderError::NoProxiesFound);
        }
        
        tracing::info!("Scraped {} proxies from Free Proxy List", proxies.len());
        
        Ok(proxies)
    }
    
    fn rate_limit(&self) -> Duration {
        Duration::from_secs(120) // 2 minutes
    }
}
```

3. Row parsing with validation:
```rust
use validator::Validate;

#[derive(Validate)]
struct ProxyData {
    #[validate(ip)]
    ip: String,
    
    #[validate(range(min = 1, max = 65535))]
    port: u16,
    
    country: String,
}

fn parse_row(&self, cells: &[scraper::ElementRef]) -> Result<ProxyConfig> {
    // Extract text from cells
    let ip = cells[0].text().collect::<String>().trim().to_string();
    let port_str = cells[1].text().collect::<String>().trim();
    let country_code = cells[2].text().collect::<String>().trim().to_string();
    let anonymity = cells[4].text().collect::<String>().trim().to_lowercase();
    let https = cells[6].text().collect::<String>().trim().to_lowercase();
    
    // Parse port
    let port = port_str.parse::<u16>()
        .map_err(|_| ProviderError::InvalidPort)?;
    
    // Validate data
    let data = ProxyData {
        ip: ip.clone(),
        port,
        country: country_code.clone(),
    };
    
    data.validate()
        .map_err(|_| ProviderError::ValidationFailed)?;
    
    // Determine proxy type
    let proxy_type = if https == "yes" {
        ProxyType::HTTPS
    } else {
        ProxyType::HTTP
    };
    
    // Determine anonymity level
    let anonymity_level = match anonymity.as_str() {
        "elite proxy" => Some(AnonymityLevel::Elite),
        "anonymous" => Some(AnonymityLevel::Anonymous),
        _ => Some(AnonymityLevel::Transparent),
    };
    
    Ok(ProxyConfig {
        id: ProxyId::new(),
        proxy_type,
        host: ip,
        port,
        username: None,
        password: None,
        enabled: true,
        countries: vec![country_code],
        anonymity_level,
        speed_mbps: None,
        uptime_percent: None,
        last_checked: None,
    })
}
```

4. Error types:
```rust
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Failed to fetch page")]
    FetchFailed(#[from] reqwest_middleware::Error),
    
    #[error("Failed to parse HTML")]
    ParseFailed,
    
    #[error("Invalid port number")]
    InvalidPort,
    
    #[error("Validation failed")]
    ValidationFailed,
    
    #[error("No proxies found")]
    NoProxiesFound,
}
```

5. Best practices implemented:
   - Respectful rate limiting (1 req/2min)
   - User-Agent header
   - Robust error handling
   - Input validation with validator
   - Structured logging
   - Automatic retries via middleware

Implement with proper HTML parsing and validation.
```

---

## 3.3 Proxy Validator (UPDATED for async)

**File**: `crates/virtual-ip/src/validator.rs`

**Claude Opus 4.5 Prompt:**
```
Create an async proxy validator using tokio and reqwest-middleware.

REQUIREMENTS:

1. Validator structure:
```rust
pub struct ProxyValidator {
    client: ClientWithMiddleware,
    test_urls: Vec<String>,
    timeout: Duration,
    max_concurrent: usize,
}

impl ProxyValidator {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self {
            client,
            test_urls: vec![
                "http://httpbin.org/ip".to_string(),
                "https://api.ipify.org?format=json".to_string(),
            ],
            timeout: Duration::from_secs(10),
            max_concurrent: 10,
        }
    }
}
```

2. Validation result:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_working: bool,
    pub response_time_ms: u64,
    pub anonymity_level: Option<AnonymityLevel>,
    pub detected_country: Option<String>,
    pub detected_ip: Option<IpAddr>,
    pub supports_https: bool,
    pub has_ip_leak: bool,
    pub error: Option<String>,
    pub validated_at: DateTime<Utc>,
}
```

3. Single proxy validation:
```rust
pub async fn validate(&self, proxy: &ProxyConfig) -> Result<ValidationResult> {
    let start = Instant::now();
    
    // Build proxy URL
    let proxy_url = format!("{}://{}:{}", 
        proxy.scheme(), proxy.host, proxy.port);
    
    let mut req_proxy = reqwest::Proxy::all(&proxy_url)
        .map_err(|e| ValidatorError::ProxyConfigError(e.to_string()))?;
    
    if let (Some(user), Some(pass)) = (&proxy.username, &proxy.password) {
        req_proxy = req_proxy.basic_auth(user, pass);
    }
    
    // Create client with proxy
    let client = reqwest::Client::builder()
        .proxy(req_proxy)
        .timeout(self.timeout)
        .build()
        .map_err(|e| ValidatorError::ClientBuildError(e.to_string()))?;
    
    // Test connection
    match client.get(&self.test_urls[0]).send().await {
        Ok(response) => {
            let elapsed = start.elapsed();
            
            // Parse response
            let body = response.text().await?;
            let detected_ip = self.extract_ip_from_response(&body);
            
            Ok(ValidationResult {
                is_working: true,
                response_time_ms: elapsed.as_millis() as u64,
                anonymity_level: Some(AnonymityLevel::Anonymous),
                detected_country: None,
                detected_ip,
                supports_https: proxy.proxy_type == ProxyType::HTTPS,
                has_ip_leak: false,
                error: None,
                validated_at: Utc::now(),
            })
        }
        Err(e) => {
            Ok(ValidationResult {
                is_working: false,
                response_time_ms: start.elapsed().as_millis() as u64,
                anonymity_level: None,
                detected_country: None,
                detected_ip: None,
                supports_https: false,
                has_ip_leak: false,
                error: Some(e.to_string()),
                validated_at: Utc::now(),
            })
        }
    }
}
```

4. Batch validation with concurrency:
```rust
use futures::stream::{self, StreamExt};

pub async fn validate_batch(
    &self, 
    proxies: &[ProxyConfig]
) -> Result<Vec<(ProxyConfig, ValidationResult)>> {
    
    let results = stream::iter(proxies)
        .map(|proxy| async move {
            let result = self.validate(proxy).await
                .unwrap_or_else(|e| ValidationResult {
                    is_working: false,
                    response_time_ms: 0,
                    anonymity_level: None,
                    detected_country: None,
                    detected_ip: None,
                    supports_https: false,
                    has_ip_leak: false,
                    error: Some(e.to_string()),
                    validated_at: Utc::now(),
                });
            
            (proxy.clone(), result)
        })
        .buffer_unordered(self.max_concurrent)
        .collect::<Vec<_>>()
        .await;
    
    Ok(results)
}
```

5. Error handling:
```rust
#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Proxy configuration error: {0}")]
    ProxyConfigError(String),
    
    #[error("Client build error: {0}")]
    ClientBuildError(String),
    
    #[error("Request failed")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Timeout")]
    Timeout,
}
```

Implement with concurrent validation using tokio streams.
```

---

## 3.4 Proxy Database (UPDATED for sqlx)

**File**: `crates/browser-core/src/proxy_database.rs`

**Claude Opus 4.5 Prompt:**
```
Implement proxy database layer using sqlx with compile-time query verification.

REQUIREMENTS:

1. Database structure:
```rust
use sqlx::{SqlitePool, query, query_as};

pub struct ProxyDatabase {
    pool: SqlitePool,
}

impl ProxyDatabase {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        let db = Self { pool };
        db.ensure_tables().await?;
        Ok(db)
    }
    
    async fn ensure_tables(&self) -> Result<()> {
        // Tables are created via migrations
        // Just verify they exist
        Ok(())
    }
}
```

2. Insert with compile-time checks:
```rust
pub async fn insert_proxy(&self, proxy: &ProxyConfig) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO proxies (
            id, proxy_type, host, port, username, password,
            country, anonymity_level, source_provider,
            is_active, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(host, port) DO UPDATE SET
            last_validated = ?
        "#,
        proxy.id.0,
        proxy.proxy_type.to_string(),
        proxy.host,
        proxy.port,
        proxy.username,
        proxy.password,
        proxy.countries.first(),
        proxy.anonymity_level.map(|a| a.to_string()),
        proxy.source_provider,
        proxy.enabled,
        Utc::now().to_rfc3339(),
        Utc::now().to_rfc3339()
    )
    .execute(&self.pool)
    .await?;
    
    Ok(())
}
```

3. Query with compile-time verification:
```rust
pub async fn get_active_proxies(&self) -> Result<Vec<ProxyConfig>> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            id, proxy_type, host, port, username, password,
            country, anonymity_level, source_provider,
            is_active, created_at, last_validated
        FROM proxies
        WHERE is_active = TRUE
        ORDER BY last_validated DESC
        "#
    )
    .fetch_all(&self.pool)
    .await?;
    
    let proxies = rows.into_iter()
        .map(|row| ProxyConfig {
            id: ProxyId(row.id),
            proxy_type: ProxyType::from_str(&row.proxy_type).unwrap(),
            host: row.host,
            port: row.port as u16,
            username: row.username,
            password: row.password,
            countries: row.country.map(|c| vec![c]).unwrap_or_default(),
            anonymity_level: row.anonymity_level
                .and_then(|a| AnonymityLevel::from_str(&a).ok()),
            source_provider: Some(row.source_provider),
            enabled: row.is_active != 0,
            speed_mbps: None,
            uptime_percent: None,
            last_checked: row.last_validated
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
        .collect();
    
    Ok(proxies)
}
```

4. Record metrics:
```rust
pub async fn record_metric(
    &self,
    proxy_id: &ProxyId,
    metric: &ValidationResult
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO proxy_metrics (
            proxy_id, response_time_ms, success,
            error_message, checked_at
        ) VALUES (?, ?, ?, ?, ?)
        "#,
        proxy_id.0,
        metric.response_time_ms as i64,
        metric.is_working,
        metric.error,
        Utc::now().to_rfc3339()
    )
    .execute(&self.pool)
    .await?;
    
    Ok(())
}
```

5. Statistics with aggregation:
```rust
pub async fn get_proxy_statistics(
    &self,
    proxy_id: &ProxyId,
    since: DateTime<Utc>
) -> Result<ProxyStatistics> {
    let stats = sqlx::query!(
        r#"
        SELECT 
            AVG(response_time_ms) as avg_response_time,
            COUNT(CASE WHEN success = TRUE THEN 1 END) as success_count,
            COUNT(*) as total_count
        FROM proxy_metrics
        WHERE proxy_id = ? AND checked_at >= ?
        "#,
        proxy_id.0,
        since.to_rfc3339()
    )
    .fetch_one(&self.pool)
    .await?;
    
    let success_rate = if stats.total_count > 0 {
        (stats.success_count as f64 / stats.total_count as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(ProxyStatistics {
        avg_response_time_ms: stats.avg_response_time.unwrap_or(0.0),
        success_rate,
        total_checks: stats.total_count,
    })
}
```

6. Cleanup old data:
```rust
pub async fn cleanup_old_metrics(&self, days: u32) -> Result<usize> {
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);
    
    let result = sqlx::query!(
        r#"
        DELETE FROM proxy_metrics
        WHERE checked_at < ?
        "#,
        cutoff.to_rfc3339()
    )
    .execute(&self.pool)
    .await?;
    
    Ok(result.rows_affected() as usize)
}
```

7. Features:
   - Compile-time SQL verification (catches typos!)
   - Type-safe queries
   - Async operations (non-blocking)
   - Connection pooling
   - Prepared statements (auto)
   - Transaction support

Implement with full sqlx features and error handling.
```

---

These updated prompts leverage the modern stack for better performance and developer experience! 🚀

