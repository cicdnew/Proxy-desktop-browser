use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::{SinkExt, StreamExt};
use tracing::{info, error, debug, warn};
use uuid::Uuid;
use std::net::SocketAddr;
use base64::engine::Engine;

use crate::proxy::{ProxySettings, ProxyType};

/// Local proxy server for routing tab traffic through upstream proxies
pub struct LocalProxyServer {
    /// Local address to bind to
    bind_addr: SocketAddr,
    /// Upstream proxy configuration
    upstream_proxy: Option<ProxySettings>,
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ProxyConnection>>>,
    /// Running state
    is_running: Arc<RwLock<bool>>,
}

/// Represents an active proxy connection
#[derive(Debug, Clone)]
pub struct ProxyConnection {
    pub id: String,
    pub client_addr: String,
    pub target_host: String,
    pub target_port: u16,
    pub upstream_proxy: Option<ProxySettings>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl LocalProxyServer {
    /// Create a new local proxy server
    pub fn new(bind_port: u16, upstream_proxy: Option<ProxySettings>) -> Result<Self> {
        let bind_addr = format!("127.0.0.1:{}", bind_port)
            .parse()
            .map_err(|e| anyhow!("Invalid bind address: {}", e))?;

        Ok(Self {
            bind_addr,
            upstream_proxy,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the local proxy server
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(anyhow!("Proxy server is already running"));
        }

        let listener = TcpListener::bind(&self.bind_addr).await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", self.bind_addr, e))?;

        info!("Local proxy server listening on {}", self.bind_addr);
        *is_running = true;
        drop(is_running);

        let connections = self.connections.clone();
        let upstream_proxy = self.upstream_proxy.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("New connection from {}", addr);
                        let conn_id = Uuid::new_v4().to_string();
                        let connections_clone = connections.clone();
                        let upstream_proxy_clone = upstream_proxy.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                addr.to_string(),
                                conn_id.clone(),
                                upstream_proxy_clone,
                                connections_clone,
                            ).await {
                                error!("Error handling connection {}: {}", conn_id, e);
                            }
                        });
                    }
                    Err(e) => {
                        if *is_running.read().await {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the local proxy server
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        // Clear all connections
        let mut connections = self.connections.write().await;
        connections.clear();
        
        info!("Local proxy server stopped");
        Ok(())
    }

    /// Get the proxy URL for configuring WebView
    pub fn get_proxy_url(&self) -> String {
        format!("http://{}", self.bind_addr)
    }

    /// Handle an incoming proxy connection
    async fn handle_connection(
        mut client_stream: TcpStream,
        client_addr: String,
        conn_id: String,
        upstream_proxy: Option<ProxySettings>,
        connections: Arc<RwLock<HashMap<String, ProxyConnection>>>,
    ) -> Result<()> {
        // Read HTTP CONNECT request
        let mut buffer = vec![0u8; 4096];
        let n = client_stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Parse CONNECT request
        let (target_host, target_port) = Self::parse_connect_request(&request)?;

        // Record the connection
        {
            let mut conns = connections.write().await;
            conns.insert(conn_id.clone(), ProxyConnection {
                id: conn_id.clone(),
                client_addr: client_addr.clone(),
                target_host: target_host.clone(),
                target_port,
                upstream_proxy: upstream_proxy.clone(),
                created_at: chrono::Utc::now(),
            });
        }

        // Send 200 Connection established response
        let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
        client_stream.write_all(response.as_bytes()).await?;

        // Connect to upstream proxy if configured, otherwise direct connection
        let target_stream = if let Some(ref proxy) = upstream_proxy {
            Self::connect_through_upstream(proxy, &target_host, target_port).await?
        } else {
            Self::connect_direct(&target_host, target_port).await?
        };

        // Start proxying data between client and target
        let (mut client_read, mut client_write) = client_stream.into_split();
        let (mut target_read, mut target_write) = target_stream.into_split();

        // Proxy client to target
        let client_to_target = tokio::spawn(async move {
            let mut buffer = vec![0u8; 8192];
            loop {
                match client_read.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if target_write.write_all(&buffer[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Proxy target to client
        let target_to_client = tokio::spawn(async move {
            let mut buffer = vec![0u8; 8192];
            loop {
                match target_read.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if client_write.write_all(&buffer[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Wait for either direction to finish
        futures::future::select(client_to_target, target_to_client).await;

        // Remove connection from active list
        {
            let mut conns = connections.write().await;
            conns.remove(&conn_id);
        }

        debug!("Connection {} closed", conn_id);
        Ok(())
    }

    /// Parse HTTP CONNECT request to extract target host and port
    fn parse_connect_request(request: &str) -> Result<(String, u16)> {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return Err(anyhow!("Empty request"));
        }

        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 || parts[0] != "CONNECT" {
            return Err(anyhow!("Invalid CONNECT request"));
        }

        let target = parts[1];
        let target_parts: Vec<&str> = target.split(':').collect();
        if target_parts.len() != 2 {
            return Err(anyhow!("Invalid target format"));
        }

        let host = target_parts[0].to_string();
        let port = target_parts[1].parse::<u16>()
            .map_err(|_| anyhow!("Invalid port"))?;

        Ok((host, port))
    }

    /// Connect directly to target host
    async fn connect_direct(host: &str, port: u16) -> Result<TcpStream> {
        let target_addr = format!("{}:{}", host, port);
        TcpStream::connect(target_addr).await
            .map_err(|e| anyhow!("Failed to connect to {}:{} - {}", host, port, e))
    }

    /// Connect through upstream proxy
    async fn connect_through_upstream(
        proxy: &ProxySettings,
        target_host: &str,
        target_port: u16,
    ) -> Result<TcpStream> {
        let proxy_addr = format!("{}:{}", 
            proxy.host.as_ref().ok_or_else(|| anyhow!("Proxy host not set"))?,
            proxy.port.ok_or_else(|| anyhow!("Proxy port not set"))?
        );

        let mut proxy_stream = TcpStream::connect(&proxy_addr).await
            .map_err(|e| anyhow!("Failed to connect to proxy {} - {}", proxy_addr, e))?;

        // Send CONNECT request to upstream proxy
        let connect_req = format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n",
            target_host, target_port, target_host, target_port
        );

        // Add proxy authentication if configured
        let auth_header = if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
            let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password).as_bytes());
            format!("Proxy-Authorization: Basic {}\r\n", auth)
        } else {
            String::new()
        };

        let full_request = format!("{}{}\r\n", connect_req, auth_header);
        proxy_stream.write_all(full_request.as_bytes()).await?;

        // Read proxy response
        let mut response_buf = vec![0u8; 1024];
        let n = proxy_stream.read(&mut response_buf).await?;
        let response = String::from_utf8_lossy(&response_buf[..n]);

        // Check if connection was established
        if !response.starts_with("HTTP/1.1 200") {
            return Err(anyhow!("Proxy failed to establish connection: {}", response));
        }

        Ok(proxy_stream)
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> Vec<ProxyConnection> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Check if the proxy server is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

/// Manager for multiple local proxy servers (one per tab)
pub struct LocalProxyManager {
    /// Active proxy servers by tab ID
    proxy_servers: Arc<RwLock<HashMap<String, Arc<LocalProxyServer>>>>,
    /// Port range for local proxies
    port_range: std::ops::Range<u16>,
    /// Currently used ports
    used_ports: Arc<RwLock<std::collections::HashSet<u16>>>,
}

impl LocalProxyManager {
    /// Create a new local proxy manager
    pub fn new(port_range: std::ops::Range<u16>) -> Self {
        Self {
            proxy_servers: Arc::new(RwLock::new(HashMap::new())),
            port_range,
            used_ports: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Create a proxy server for a specific tab
    pub async fn create_proxy_for_tab(
        &self,
        tab_id: &str,
        upstream_proxy: Option<ProxySettings>,
    ) -> Result<String> {
        // Find an available port
        let port = self.find_available_port().await?;

        // Create and start the proxy server
        let proxy_server = Arc::new(LocalProxyServer::new(port, upstream_proxy)?);
        proxy_server.start().await?;

        // Register the proxy server
        {
            let mut servers = self.proxy_servers.write().await;
            servers.insert(tab_id.to_string(), proxy_server.clone());
        }

        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.insert(port);
        }

        let proxy_url = proxy_server.get_proxy_url();
        info!("Created proxy for tab {} on {}", tab_id, proxy_url);

        Ok(proxy_url)
    }

    /// Remove proxy server for a tab
    pub async fn remove_proxy_for_tab(&self, tab_id: &str) -> Result<()> {
        let proxy_server = {
            let mut servers = self.proxy_servers.write().await;
            servers.remove(tab_id)
        };

        if let Some(server) = proxy_server {
            server.stop().await?;
            
            // Extract port from bind address and mark as available
            let addr_str = server.get_proxy_url();
            if let Some(port_str) = addr_str.split(':').nth(1) {
                if let Ok(port) = port_str.parse::<u16>() {
                    let mut used_ports = self.used_ports.write().await;
                    used_ports.remove(&port);
                }
            }

            info!("Removed proxy for tab {}", tab_id);
        }

        Ok(())
    }

    /// Get proxy URL for a tab
    pub async fn get_proxy_url_for_tab(&self, tab_id: &str) -> Option<String> {
        let servers = self.proxy_servers.read().await;
        servers.get(tab_id).map(|server| server.get_proxy_url())
    }

    /// Find an available port in the configured range
    async fn find_available_port(&self) -> Result<u16> {
        let used_ports = self.used_ports.read().await;
        
        for port in self.port_range.clone() {
            if !used_ports.contains(&port) {
                return Ok(port);
            }
        }

        Err(anyhow!("No available ports in range {:?}", self.port_range))
    }

    /// Get all active proxy servers
    pub async fn get_active_proxies(&self) -> HashMap<String, String> {
        let servers = self.proxy_servers.read().await;
        servers.iter()
            .map(|(tab_id, server)| (tab_id.clone(), server.get_proxy_url()))
            .collect()
    }

    /// Stop all proxy servers
    pub async fn stop_all(&self) -> Result<()> {
        let servers: Vec<_> = {
            let servers = self.proxy_servers.read().await;
            servers.values().cloned().collect()
        };

        for server in servers {
            server.stop().await?;
        }

        {
            let mut servers = self.proxy_servers.write().await;
            servers.clear();
        }

        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.clear();
        }

        info!("Stopped all proxy servers");
        Ok(())
    }
}
