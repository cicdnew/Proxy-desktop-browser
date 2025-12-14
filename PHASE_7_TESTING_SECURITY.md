# Phase 7: Testing & Security

## 7.1 Comprehensive Testing Suite

**Claude Opus 4.5 Prompt:**
```
Create a comprehensive testing suite for the browser application covering unit tests, integration tests, and end-to-end tests.

TESTING STRUCTURE:

### Unit Tests

**File**: `crates/browser-core/tests/proxy_tests.rs`
```rust
#[cfg(test)]
mod proxy_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_proxy_connection() {
        let proxy = ProxyConfig {
            id: "test-proxy".into(),
            proxy_type: ProxyType::HTTP,
            host: "proxy.example.com".into(),
            port: 8080,
            username: None,
            password: None,
            enabled: true,
            countries: vec!["US".into()],
            speed_mbps: None,
            uptime_percent: None,
            last_checked: None,
        };
        
        let manager = ProxyManager::new();
        let result = manager.connect(&proxy).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_proxy_rotation() {
        let rotator = ProxyRotator::new(RotationStrategy::RoundRobin);
        let proxies = vec![/* test proxies */];
        
        rotator.set_proxy_pool(proxies.clone());
        
        let first = rotator.get_next_proxy().await.unwrap();
        let second = rotator.get_next_proxy().await.unwrap();
        
        assert_ne!(first.id, second.id);
    }
    
    #[tokio::test]
    async fn test_proxy_validation() {
        let validator = ProxyValidator::new();
        let proxy = /* test proxy */;
        
        let result = validator.validate(&proxy).await;
        assert!(result.is_working);
        assert!(result.response_time_ms < 5000);
    }
}
```

**File**: `crates/virtual-ip/tests/ip_generation_tests.rs`
```rust
#[cfg(test)]
mod ip_generation_tests {
    #[test]
    fn test_valid_ip_generation() {
        let generator = IpGenerator::new();
        let ip = generator.generate_random_ip();
        assert!(ip.is_ipv4() || ip.is_ipv6());
    }
    
    #[test]
    fn test_ip_uniqueness() {
        let generator = IpGenerator::new();
        let mut ips = HashSet::new();
        
        for _ in 0..1000 {
            let ip = generator.generate_random_ip();
            ips.insert(ip);
        }
        
        assert_eq!(ips.len(), 1000); // All unique
    }
}
```

### Integration Tests

**File**: `crates/browser-core/tests/integration_tests.rs`
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_request_flow() {
        // Setup
        let proxy_manager = ProxyManager::new();
        let http_client = HttpClient::new(proxy_manager);
        let proxy = /* test proxy */;
        
        // Execute
        let request = Request::get("https://httpbin.org/ip").build();
        let response = http_client.send(request, proxy).await;
        
        // Verify
        assert!(response.is_ok());
        let body = response.unwrap().text().await.unwrap();
        assert!(body.contains("origin"));
    }
    
    #[tokio::test]
    async fn test_tab_isolation() {
        let isolation_manager = TabIsolationManager::new(PathBuf::from("/tmp/test"));
        
        let tab1 = TabId::new();
        let tab2 = TabId::new();
        
        isolation_manager.create_context(tab1).await.unwrap();
        isolation_manager.create_context(tab2).await.unwrap();
        
        // Set cookie in tab1
        isolation_manager.set_cookie(tab1, Cookie::new("test", "value1")).await.unwrap();
        
        // Verify tab2 doesn't have the cookie
        let cookies = isolation_manager.get_cookies(tab2, "example.com").await.unwrap();
        assert!(cookies.is_empty());
    }
    
    #[tokio::test]
    async fn test_proxy_failover() {
        let mut rotator = ProxyRotator::new(RotationStrategy::OnFailure);
        let proxies = vec![
            /* dead proxy */,
            /* working proxy */,
        ];
        
        rotator.set_proxy_pool(proxies);
        
        // First request should fail, second should succeed
        let proxy1 = rotator.get_next_proxy().await.unwrap();
        rotator.mark_failure(&proxy1.id).await;
        
        let proxy2 = rotator.get_next_proxy().await.unwrap();
        assert_ne!(proxy1.id, proxy2.id);
    }
}
```

### End-to-End Tests

**File**: `tests/e2e/browser_tests.rs`
```rust
use tauri::test::{mock_builder, MockRuntime};

#[tokio::test]
async fn test_complete_browsing_session() {
    let app = mock_builder().build().unwrap();
    
    // Create new tab
    let tab_id: TabId = app.invoke("create_tab", json!({
        "url": "https://example.com"
    })).await.unwrap();
    
    // Wait for page load
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Get tab info
    let tab: Tab = app.invoke("get_tab", json!({
        "tab_id": tab_id
    })).await.unwrap();
    
    assert_eq!(tab.url, "https://example.com");
    assert!(!tab.loading);
    
    // Navigate to new URL
    app.invoke("navigate", json!({
        "tab_id": tab_id,
        "url": "https://httpbin.org/ip"
    })).await.unwrap();
    
    // Close tab
    app.invoke("close_tab", json!({
        "tab_id": tab_id
    })).await.unwrap();
}

#[tokio::test]
async fn test_proxy_switching() {
    let app = mock_builder().build().unwrap();
    
    // Enable proxy
    app.invoke("enable_proxy", json!({
        "rotation_strategy": "manual"
    })).await.unwrap();
    
    // Get current proxy
    let proxy: Option<ProxyConfig> = app.invoke("get_current_proxy", json!({})).await.unwrap();
    assert!(proxy.is_some());
    
    // Rotate proxy
    app.invoke("rotate_proxy", json!({})).await.unwrap();
    
    // Verify new proxy is different
    let new_proxy: Option<ProxyConfig> = app.invoke("get_current_proxy", json!({})).await.unwrap();
    assert!(new_proxy.is_some());
    assert_ne!(proxy.unwrap().id, new_proxy.unwrap().id);
}
```

### Performance Tests

**File**: `tests/performance/load_tests.rs`
```rust
#[tokio::test]
async fn test_multiple_tabs_performance() {
    let start = Instant::now();
    
    let tab_manager = TabManager::new();
    let mut tasks = vec![];
    
    // Create 50 tabs simultaneously
    for i in 0..50 {
        let task = tokio::spawn(async move {
            tab_manager.create_tab(&format!("https://example.com/{}", i)).await
        });
        tasks.push(task);
    }
    
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    assert!(duration.as_secs() < 5, "Creating 50 tabs took too long: {:?}", duration);
}

#[tokio::test]
async fn test_proxy_validation_performance() {
    let validator = ProxyValidator::new();
    let proxies = vec![/* 100 test proxies */];
    
    let start = Instant::now();
    let results = validator.validate_batch(&proxies).await.unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_secs() < 30, "Validating 100 proxies took too long");
    assert_eq!(results.len(), 100);
}
```

### Test Coverage Goals:
- Unit tests: 80%+ code coverage
- Integration tests: All critical paths
- E2E tests: Main user flows
- Performance tests: All async operations
```

---

## 7.2 Security Auditing

**Claude Opus 4.5 Prompt:**
```
Implement security measures and auditing for the browser application.

SECURITY CHECKLIST:

### 1. Input Validation
```rust
pub fn validate_url(url: &str) -> Result<Url, ValidationError> {
    let parsed = Url::parse(url)?;
    
    // Block dangerous protocols
    let allowed_protocols = vec!["http", "https", "file"];
    if !allowed_protocols.contains(&parsed.scheme()) {
        return Err(ValidationError::UnsafeProtocol(parsed.scheme().to_string()));
    }
    
    // Block local network IPs in production
    if let Some(host) = parsed.host_str() {
        if is_private_ip(host) {
            return Err(ValidationError::PrivateIp);
        }
    }
    
    Ok(parsed)
}
```

### 2. SQL Injection Prevention
```rust
// Always use parameterized queries
pub async fn add_bookmark(&self, title: &str, url: &str) -> Result<BookmarkId> {
    let id = Uuid::new_v4().to_string();
    
    self.db.execute(
        "INSERT INTO bookmarks (id, title, url, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, url, Utc::now().to_rfc3339()]
    )?;
    
    Ok(id.into())
}
```

### 3. XSS Prevention
```rust
pub fn sanitize_html(input: &str) -> String {
    use ammonia::clean;
    clean(input)
}

// In UI rendering
pub fn render_tab_title(title: &str) -> String {
    html_escape::encode_text(title).to_string()
}
```

### 4. Path Traversal Prevention
```rust
pub fn validate_download_path(base_dir: &Path, requested_path: &Path) -> Result<PathBuf> {
    let canonical = requested_path.canonicalize()?;
    
    if !canonical.starts_with(base_dir) {
        return Err(SecurityError::PathTraversal);
    }
    
    Ok(canonical)
}
```

### 5. Credential Storage
```rust
use keyring::Entry;

pub struct SecureStorage {
    service_name: String,
}

impl SecureStorage {
    pub fn store_credential(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.set_password(value)?;
        Ok(())
    }
    
    pub fn retrieve_credential(&self, key: &str) -> Result<String> {
        let entry = Entry::new(&self.service_name, key)?;
        Ok(entry.get_password()?)
    }
}
```

### 6. Certificate Validation
```rust
use reqwest::Certificate;

pub fn build_secure_client() -> Result<Client> {
    let client = Client::builder()
        .tls_built_in_root_certs(true)
        .danger_accept_invalid_certs(false) // Never disable in production!
        .min_tls_version(Version::TLS_1_2)
        .build()?;
    
    Ok(client)
}
```

### 7. Rate Limiting
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct ApiRateLimiter {
    limiter: RateLimiter<DirectRateLimiter>,
}

impl ApiRateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }
    
    pub async fn check(&self) -> Result<()> {
        self.limiter.check().map_err(|_| Error::RateLimitExceeded)?;
        Ok(())
    }
}
```

### 8. Memory Safety
```rust
// Use Arc<Mutex<>> for shared state
pub struct SharedState {
    proxy_manager: Arc<Mutex<ProxyManager>>,
    tab_manager: Arc<Mutex<TabManager>>,
}

// Implement proper Drop for cleanup
impl Drop for TabIsolationManager {
    fn drop(&mut self) {
        // Clean up temporary files
        for context in self.contexts.lock().unwrap().values() {
            let _ = std::fs::remove_dir_all(&context.cache_dir);
        }
    }
}
```

### 9. Security Headers
```rust
pub fn add_security_headers(response: &mut Response) {
    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'".parse().unwrap()
    );
    response.headers_mut().insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap()
    );
    response.headers_mut().insert(
        "X-Frame-Options",
        "DENY".parse().unwrap()
    );
    response.headers_mut().insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap()
    );
}
```

### 10. Logging & Auditing
```rust
use tracing::{info, warn, error};

pub async fn handle_sensitive_operation(user: &str, action: &str) -> Result<()> {
    info!(
        user = user,
        action = action,
        timestamp = %Utc::now(),
        "Security audit: sensitive operation"
    );
    
    // Perform operation
    let result = perform_operation().await;
    
    if let Err(e) = &result {
        error!(
            user = user,
            action = action,
            error = %e,
            "Security audit: operation failed"
        );
    }
    
    result
}
```

### Security Test Suite
```rust
#[cfg(test)]
mod security_tests {
    #[test]
    fn test_sql_injection_prevention() {
        let malicious_input = "'; DROP TABLE bookmarks; --";
        let result = add_bookmark(malicious_input, "http://example.com");
        assert!(result.is_ok());
        
        // Verify table still exists
        let bookmarks = list_bookmarks();
        assert!(bookmarks.is_ok());
    }
    
    #[test]
    fn test_path_traversal_prevention() {
        let base = PathBuf::from("/home/user/downloads");
        let malicious = PathBuf::from("../../../etc/passwd");
        
        let result = validate_download_path(&base, &malicious);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_xss_prevention() {
        let malicious = "<script>alert('XSS')</script>";
        let sanitized = sanitize_html(malicious);
        assert!(!sanitized.contains("<script>"));
    }
}
```

Implement all security measures with comprehensive testing and documentation.
```

---

## 7.3 Error Handling & Recovery

**File**: `crates/browser-core/src/error.rs`

**Claude Opus 4.5 Prompt:**
```
Implement comprehensive error handling and recovery mechanisms for the browser application.

ERROR TYPES:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Proxy connection failed: {0}")]
    ProxyConnectionFailed(String),
    
    #[error("Tab not found: {0}")]
    TabNotFound(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("WebView error: {0}")]
    WebViewError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Timeout error: operation took too long")]
    TimeoutError,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, BrowserError>;
```

ERROR RECOVERY STRATEGIES:
```rust
pub struct ErrorRecovery {
    max_retries: u32,
    retry_delay: Duration,
}

impl ErrorRecovery {
    pub async fn with_retry<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> std::result::Result<T, E>,
        E: std::error::Error,
    {
        let mut attempts = 0;
        
        loop {
            match operation() {
                Ok(value) => return Ok(value),
                Err(e) if attempts < self.max_retries => {
                    attempts += 1;
                    warn!("Operation failed (attempt {}/{}): {}", attempts, self.max_retries, e);
                    tokio::time::sleep(self.retry_delay).await;
                }
                Err(e) => {
                    error!("Operation failed after {} attempts: {}", attempts, e);
                    return Err(BrowserError::from(format!("Max retries exceeded: {}", e)));
                }
            }
        }
    }
    
    pub async fn with_timeout<F, T>(&self, duration: Duration, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        tokio::time::timeout(duration, operation)
            .await
            .map_err(|_| BrowserError::TimeoutError)?
    }
    
    pub async fn with_fallback<F1, F2, T>(&self, primary: F1, fallback: F2) -> Result<T>
    where
        F1: Future<Output = Result<T>>,
        F2: Future<Output = Result<T>>,
    {
        match primary.await {
            Ok(value) => Ok(value),
            Err(e) => {
                warn!("Primary operation failed, trying fallback: {}", e);
                fallback.await
            }
        }
    }
}
```

CRASH RECOVERY:
```rust
pub struct CrashRecovery {
    session_manager: Arc<SessionManager>,
    state_dir: PathBuf,
}

impl CrashRecovery {
    pub async fn save_crash_dump(&self) -> Result<()> {
        let state = self.capture_application_state().await?;
        
        let dump_path = self.state_dir.join(format!("crash_{}.json", Utc::now().timestamp()));
        
        let json = serde_json::to_string_pretty(&state)?;
        tokio::fs::write(dump_path, json).await?;
        
        Ok(())
    }
    
    pub async fn recover_from_crash(&self) -> Result<Option<Session>> {
        let entries = std::fs::read_dir(&self.state_dir)?;
        
        let mut crash_dumps: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().to_string_lossy().contains("crash_"))
            .collect();
        
        if crash_dumps.is_empty() {
            return Ok(None);
        }
        
        // Get most recent crash dump
        crash_dumps.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());
        let latest = crash_dumps.last().unwrap();
        
        let json = tokio::fs::read_to_string(latest.path()).await?;
        let session: Session = serde_json::from_str(&json)?;
        
        // Delete crash dump after successful recovery
        tokio::fs::remove_file(latest.path()).await?;
        
        Ok(Some(session))
    }
}
```

ERROR NOTIFICATION TO UI:
```rust
use tauri::Manager;

pub fn notify_error(app: &AppHandle, error: &BrowserError) {
    let message = match error {
        BrowserError::ProxyConnectionFailed(msg) => {
            format!("Proxy connection failed: {}. Trying next proxy...", msg)
        }
        BrowserError::NetworkError(e) => {
            format!("Network error: {}. Please check your connection.", e)
        }
        BrowserError::TimeoutError => {
            "Operation timed out. Please try again.".to_string()
        }
        _ => error.to_string(),
    };
    
    app.emit_all("error-notification", json!({
        "message": message,
        "severity": "error",
        "timestamp": Utc::now().to_rfc3339(),
    })).ok();
}
```

Implement with comprehensive error handling, logging, and user-friendly error messages.
```

