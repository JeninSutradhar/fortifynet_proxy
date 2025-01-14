//! # FortifyNet Proxy
//!
//! A flexible and asynchronous proxy server library built with Rust.
//!
//! This crate provides a robust foundation for building various proxy servers,
//! including HTTP, HTTPS, and SOCKS5 proxies. It also features built-in
//! metrics, caching, and a simple dashboard for monitoring.
//!
//! ## Features
//!
//! *   **Asynchronous I/O:** Built with `tokio` for efficient handling of concurrent connections.
//! *   **HTTP/HTTPS Proxying:** Handles both HTTP and HTTPS traffic using `hyper` and `tokio-rustls`.
//! *   **SOCKS5 Proxy Support:** Supports proxying through SOCKS5 servers using `tokio-socks`.
//! *   **Request Caching:** Implements a simple in-memory cache for responses.
//! *   **Built-in Metrics:** Provides real-time traffic statistics, error tracking, and response time analysis.
//! *   **Basic Dashboard:** Includes a simple web-based dashboard using `warp` for live metrics.
//! *   **Configurable:** Highly configurable through the `ProxyConfig` struct.
//!
//! ## Getting Started
//!
//! To use this library, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fortifynet_proxy = "1.1.9"  # Or the latest version
//! tokio = { version = "1", features = ["full"] }
//! hyper = { version = "0.14", features = ["client","http1","server","tcp"] }
//! log = "0.4"
//! env_logger = "0.10"
//! thiserror = "1"
//! anyhow = "1"
//! rustls = "0.21"
//! tokio-rustls = "0.24"
//! tokio-socks = "0.3"
//! url = "2.5"
//! warp = "0.3"
//! rustls-pemfile = "1.1"
//! ```
//!
//! Then, in your `main.rs` or library code, use the `start_proxy_server` function to start a proxy server.
//!
//! ```rust
//! use fortifynet_proxy::{start_proxy_server, ProxyConfig};
//! use log::info;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!
//!     // Create a proxy configuration with default values
//!     let config = ProxyConfig {
//!         ip_address: "127.0.0.1".to_string(),
//!         port: 8080,
//!         authentication: false,
//!         username: "admin".to_string(),
//!         password: "password".to_string(),
//!         cache_enabled: true,
//!         socks5_address: None,
//!         https_enabled: false,
//!         certificate_path: None,
//!         private_key_path: None,
//!          target_address: Some("http://www.example.com".to_string()),
//!     };
//!      info!("Starting Proxy server with configuration: {:?}", config);
//!     // Start the proxy server with the provided configuration
//!     start_proxy_server(config).await?;
//!     Ok(())
//! }
//! ```
//!
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use hyper::{
    body::{Bytes, to_bytes},
    client::{Client, HttpConnector},
    header::{HeaderValue, HOST},
    service::service_fn,
    Body, Method, Request, Response, StatusCode,
};
use log::{debug, error, info, warn};
use std::str::FromStr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tokio_socks::tcp::Socks5Stream;
use url::Url;
use warp::http::Response as WarpResponse;
use warp::Filter;

// Constants for metrics
const METRICS_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

/// Configuration for the proxy server.
#[derive(Clone, Debug)]
pub struct ProxyConfig {
    /// IP address to bind the server to. Defaults to `127.0.0.1`.
    pub ip_address: String,
    /// Port number to bind the server to. Defaults to `8080`.
    pub port: u16,
    /// Flag indicating whether authentication is required. Defaults to `false`.
    pub authentication: bool,
    /// Username for authentication. Only used if `authentication` is `true`.
    pub username: String,
    /// Password for authentication. Only used if `authentication` is `true`.
    pub password: String,
    /// Flag indicating whether caching is enabled. Defaults to `true`.
    pub cache_enabled: bool,
    /// SOCKS5 proxy address (optional). If provided, all traffic is routed through this SOCKS5 proxy server.
    pub socks5_address: Option<String>,
    /// Flag indicating whether HTTPS support is enabled. Defaults to `false`.
    pub https_enabled: bool,
    /// Path to SSL certificate file for HTTPS. Only used if `https_enabled` is `true`.
    pub certificate_path: Option<String>,
    /// Path to SSL private key file for HTTPS. Only used if `https_enabled` is `true`.
    pub private_key_path: Option<String>,
     /// Target address to send requests when not using socks5
    pub target_address: Option<String>,
}

// Implementing Default Method for ProxyConfig
impl Default for ProxyConfig {
    /// Returns a new `ProxyConfig` instance with default values
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            port: 8080,
            authentication: false,
            username: "".to_string(),
            password: "".to_string(),
            cache_enabled: true,
            socks5_address: None,
            https_enabled: false,
            certificate_path: None,
            private_key_path: None,
            target_address: None,
        }
    }
}

/// Struct to hold and manage metrics
#[derive(Default, Clone, Debug)]
pub struct Metrics {
    /// Total number of requests handled by the proxy.
    pub total_requests: u64,
    /// A vector of durations, representing the response times for each request.
    pub response_times: Vec<Duration>,
    /// Total number of cache hits.
    pub cache_hits: u64,
    /// Total number of cache misses.
    pub cache_misses: u64,
    /// A hashmap of error counts, with the keys representing status codes of errors.
    pub error_counts: HashMap<u16, u64>,
}

impl Metrics {
    /// Records a new request, updating `total_requests` and `response_times`.
    pub fn record_request(&mut self, duration: Duration) {
        self.total_requests += 1;
        self.response_times.push(duration);
    }

    /// Records a cache hit, incrementing `cache_hits`.
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Records a cache miss, incrementing `cache_misses`.
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Records an error, incrementing the corresponding entry in `error_counts`.
    pub fn record_error(&mut self, status_code: u16) {
        *self.error_counts.entry(status_code).or_insert(0) += 1;
    }

    /// Gets the average response time of all the requests.
    pub fn get_average_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::from_secs(0);
        }
        let sum: Duration = self.response_times.iter().sum();
        sum / (self.response_times.len() as u32)
    }
}

/// Structure for the global state of the proxy server
pub struct ProxyState {
    /// The proxy configuration
    pub config: ProxyConfig,
    /// Cache for storing responses
    pub cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Metrics for collecting proxy stats
    pub metrics: Arc<Mutex<Metrics>>,
    /// HTTP client to be used for making requests
    pub http_client: Client<HttpConnector, Body>,
}

impl ProxyState {
    /// Creates a new proxy state with the given configuration.
    pub fn new(config: ProxyConfig) -> Self {
        ProxyState {
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Metrics::default())),
            http_client: Client::new(), //create a new client
        }
    }
}

/// Handles an incoming client connection, authenticates the user if needed, and forwards the request to be handled further.
async fn handle_client_connection(
    mut stream: TcpStream,
    state: Arc<ProxyState>,
    addr: SocketAddr,
) -> Result<()> {
    debug!("Handling connection from: {}", addr);
    // Check if authentication is required and handle authentication
    if state.config.authentication && !handle_authentication(&mut stream, &state.config).await? {
        return Ok(());
    }

    if state.config.https_enabled {
        handle_https_connection(stream, state, addr).await
    } else {
        handle_http_connection(stream, state, addr).await
    }
}

/// Handles authentication for incoming client connections
async fn handle_authentication(stream: &mut TcpStream, config: &ProxyConfig) -> Result<bool> {
    let mut login_buffer = [0; 1024];

    // Read login data from the client
    let bytes_read = stream.peek(&mut login_buffer).await?;
    let login_data = String::from_utf8_lossy(&login_buffer[..bytes_read]);
    debug!("Received login data: {}", login_data);

    // Check if the login data matches the configured username and password
    if login_data.contains(&format!("{}:{}", config.username, config.password)) {
        //consume the login data and return true
        stream.read(&mut login_buffer[..bytes_read]).await?;
        info!("Successful login");
        Ok(true)
    } else {
        // If authentication fails, send a 401 Unauthorized response to the client
        let response = b"HTTP/1.1 401 Unauthorized\r\n\r\n";
        stream.write_all(response).await?;
        warn!("Failed login attempt");
        Ok(false)
    }
}

/// Handles HTTP requests
async fn handle_http_connection(
    stream: TcpStream,
    state: Arc<ProxyState>,
    addr: SocketAddr,
) -> Result<()> {
    debug!("Handling HTTP connection from: {}", addr);
    let service = service_fn(move |req| {
        let state = state.clone();
        async move { handle_http_request(req, state).await }
    });
    let http = hyper::server::conn::Http::new().serve_connection(stream, service);

    if let Err(err) = http.await {
        error!("Error serving HTTP connection from {}: {}", addr, err);
        return Err(err.into());
    }
    Ok(())
}
/// Handles HTTPS connections
async fn handle_https_connection(
    stream: TcpStream,
    state: Arc<ProxyState>,
    addr: SocketAddr,
) -> Result<()> {
    debug!("Handling HTTPS connection from: {}", addr);
    let tls_acceptor = create_tls_acceptor(&state.config)?;

    match tls_acceptor.accept(stream).await {
        Ok(tls_stream) => {
            let service = service_fn(move |req: hyper::Request<Body>| {
                let state = state.clone();
                async move { handle_http_request(req, state).await }
            });

            let http = hyper::server::conn::Http::new().serve_connection(tls_stream, service);

            if let Err(err) = http.await {
                error!("Error serving HTTPS connection from {}: {}", addr, err);
                return Err(err.into());
            }
            Ok(())
        }
        Err(e) => {
            error!("TLS handshake failed with {}: {}", addr, e);
            Err(e.into())
        }
    }
}

/// Creates a TLS acceptor for HTTPS
fn create_tls_acceptor(config: &ProxyConfig) -> Result<TlsAcceptor> {
    let cert_path = config
        .certificate_path
        .as_ref()
        .context("Certificate path required for HTTPS")?;
    let key_path = config
        .private_key_path
        .as_ref()
        .context("Private key path required for HTTPS")?;

    let cert_file = std::fs::File::open(cert_path).context("Failed to open cert file")?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_reader)
        .context("Failed to read certificate")?
        .into_iter()
        .map(Certificate)
        .collect();

    let key_file = std::fs::File::open(key_path).context("Failed to open key file")?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let keys: Vec<PrivateKey> = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .context("Failed to read private key")?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        anyhow::bail!("No private keys found in key file");
    }

    let mut server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.first().unwrap().clone())
        .map_err(|err| anyhow::anyhow!("Invalid certificate or private key: {}", err))?;

    server_config.alpn_protocols.push(b"http/1.1".to_vec());

    Ok(TlsAcceptor::from(Arc::new(server_config)))
}

/// Handles an HTTP request, checks cache, forwards the request to the target server, and updates the metrics and cache accordingly
async fn handle_http_request(req: Request<Body>, state: Arc<ProxyState>) -> Result<Response<Body>> {
    let start = std::time::Instant::now();
    let (parts, body) = req.into_parts();
    let uri = parts.uri.clone();
    let method = parts.method.clone();
    let url_string = uri.to_string();
    debug!("Incoming request: {} {}", method, url_string);
    let mut response_to_client = Response::new(Body::empty());

    // Check cache
    if state.config.cache_enabled && method == Method::GET {
        let cache = state.cache.lock().unwrap();
        if let Some(response_body) = cache.get(&url_string) {
            let duration = start.elapsed();
            state.metrics.lock().unwrap().record_cache_hit();
            info!("Cache hit for: {}, took: {:?}", url_string, duration);
            *response_to_client.status_mut() = StatusCode::OK;
            *response_to_client.body_mut() = Body::from(Bytes::copy_from_slice(response_body));
            return Ok(response_to_client);
        } else {
            state.metrics.lock().unwrap().record_cache_miss();
            debug!("Cache miss for: {}", url_string);
        }
    }

    // Forward the request to the target server
    let mut forward_response = forward_request(parts, body, state.clone()).await?;
    let status = forward_response.status();
    let duration = start.elapsed();

    //Update Metrics
    {
        let mut metrics = state.metrics.lock().unwrap();
        metrics.record_request(duration);
        if !status.is_success() {
            metrics.record_error(status.as_u16());
        }
    }
    debug!("Forwarded request to server, took: {:?}", duration);

    // Cache response
    if state.config.cache_enabled && method == Method::GET && status.is_success() {
        match to_bytes(forward_response.body_mut()).await {
            Ok(full_response) => {
                let mut cache = state.cache.lock().unwrap();
                cache.insert(url_string.clone(), full_response.to_vec());
                info!(
                    "Cache insert for: {}, took: {:?} and response status: {}",
                    url_string, duration, status
                );
                response_to_client = forward_response;
            }
            Err(e) => {
                error!(
                    "Error reading response body for caching {}: {}",
                    url_string, e
                );
                // If caching fails, still return the original response
                response_to_client = forward_response;
            }
        }
    } else {
        response_to_client = forward_response;
    }
    info!(
        "Request for: {}, took: {:?} and response status: {}",
        url_string, duration, status
    );
    Ok(response_to_client)
}

/// Forwards a request to the upstream server
async fn forward_request(
    parts: hyper::http::request::Parts,
    body: Body,
    state: Arc<ProxyState>,
) -> Result<Response<Body>> {
    let uri_to_use = parts.uri.clone();
    debug!("Forwarding request to: {}", uri_to_use.to_string());
    debug!("Request headers: {:?}", parts.headers);

    let response = if let Some(socks5_addr) = &state.config.socks5_address {
        debug!("Using SOCKS5 proxy: {}", socks5_addr);
        let mut uri_string = parts.uri.to_string();
        if uri_string.starts_with("http://") {
            uri_string = uri_string.replace("http://", "");
        } else if uri_string.starts_with("https://") {
            uri_string = uri_string.replace("https://", "");
        }
        let url = Url::from_str(&format!("http://{}", uri_string))?;
        let proxy_addr = SocketAddr::from_str(socks5_addr)
            .map_err(|e| anyhow::anyhow!("Failed to parse SOCKS5 address: {}", e))?;

        let stream = Socks5Stream::connect(
            proxy_addr,
            (url.host_str().unwrap(), url.port().unwrap_or(80)),
        )
        .await?;
        let (mut sender, conn) = hyper::client::conn::handshake(stream).await?;
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection error on SOCKS5 connection: {}", err);
            }
        });
        let mut req = Request::from_parts(parts, body);
        req.headers_mut()
            .insert(HOST, HeaderValue::from_str(url.host_str().unwrap())?);

        debug!("Sending request through SOCKS5 proxy");
        sender
            .send_request(req)
            .await
            .context("Failed to make request through socks5 proxy")
    } else {
        debug!(
            "Attempting direct connection for: {}",
            uri_to_use.to_string()
        );
        let target_host = state
            .config
            .target_address
            .as_ref()
            .map_or(
                "http://localhost".to_string(), //set default target to localhost if target address is not present
                |url| url.clone(),
            );
        let target_url = format!("{}{}", target_host, uri_to_use);
         let client = state.http_client.clone();
        let mut req = Request::from_parts(parts, body);
          let url = Url::from_str(target_url.as_str())
            .map_err(|e| anyhow::anyhow!("Failed to parse URI: {}", e))?;

        req.headers_mut()
           .insert(
                HOST,
              HeaderValue::from_str(url.host_str().unwrap())
                .map_err(|e| anyhow::anyhow!("Failed to make Host Header: {}", e))?
           );
        *req.uri_mut() = url.to_string().parse().unwrap();
         debug!("Direct connection request: {:?}", req);
          client
            .request(req)
            .await
            .context("Failed to make request through direct connection")
    };

    match response {
        Ok(response) => {
            debug!(
                "Received response from {}: status {}",
                uri_to_use,
                response.status()
            );
            Ok(response)
        }
        Err(err) => {
            error!("Error forwarding request to {}: {}", uri_to_use, err);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Failed to forward request to {}: {}",
                    uri_to_use, err
                )))
                .unwrap())
        }
    }
}

/// Starts the proxy server
pub async fn start_proxy_server(config: ProxyConfig) -> Result<()> {
    let state = Arc::new(ProxyState::new(config));
    let state_clone = state.clone();
    let config_clone = state.config.clone();
    let metrics_clone = state.metrics.clone();

    // Initialize the logger
    env_logger::init();

    // Start metrics update task in background
    tokio::spawn(async move {
        info!("Starting metrics update task");
        metrics_update_task(metrics_clone).await;
    });

    // Start the dashboard server
    tokio::spawn(async move {
        info!("Starting metrics dashboard");
        start_metrics_dashboard(config_clone, state_clone).await;
    });

    let bind_address = format!("{}:{}", state.config.ip_address, state.config.port);
    let listener = TcpListener::bind(&bind_address)
        .await
        .context(format!("Failed to bind to address: {}", bind_address))?;
    info!("Proxy server listening on: {}", bind_address);
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    info!("New connection from {}", addr);
                    if let Err(err) = handle_client_connection(stream, state_clone, addr).await {
                        error!("Error handling client connection from {}: {}", addr, err);
                    } else {
                        info!("Connection from {} handled successfully", addr);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}

/// Starts a simple metrics dashboard with warp crate
///
/// This function starts a simple web server with warp crate that exposes two routes:
/// - /metrics: Displays the current metrics of the proxy server
/// - /: Displays a simple HTML page with a link to the metrics route
///
/// The metrics route displays the following metrics:
/// - Total requests: The total number of requests handled by the proxy server
/// - Average response time: The average response time of all the requests
/// - Cache hits: The number of cache hits
/// - Cache misses: The number of cache misses
/// - Error counts: The number of errors for each status code
async fn start_metrics_dashboard(config: ProxyConfig, state: Arc<ProxyState>) {
    info!("Starting metrics dashboard...");
    // Define metrics route
    let metrics_route = warp::path!("metrics").map(move || {
        info!("Metrics route hit");
        let metrics = state.metrics.lock().unwrap();
        let body = format!(
            "<h1>Metrics</h1>\
            <ul>\
                <li><strong>Total requests:</strong> {}</li>\
                <li><strong>Average response time:</strong> {:?}</li>\
                <li><strong>Cache hits:</strong> {}</li>\
                <li><strong>Cache misses:</strong> {}</li>\
                <li><strong>Error counts:</strong> {:?}</li>\
            </ul>",
            metrics.total_requests,
            metrics.get_average_response_time(),
            metrics.cache_hits,
            metrics.cache_misses,
            metrics.error_counts,
        );
        // Return an HTML response with the metrics
        WarpResponse::builder()
            .header("Content-Type", "text/html")
            .body(body)
    });
    // Define index route
    let index_route = warp::path::end().map(move || {
        info!("Index route hit");
        let body = format!(
            "<h1>FortifyNet Proxy Server</h1>\
            <p>Welcome to FortifyNet proxy server dashboard.</p>\
            <a href='/metrics' style='font-size: 18px; color: blue;'>View Metrics</a>"
        );
        // Return an HTML response with a link to the metrics route
        WarpResponse::builder()
            .header("Content-Type", "text/html")
            .body(body)
    });

    // Combine routes
    let routes = metrics_route.or(index_route);

    // Bind the metrics dashboard to an address
    let dashboard_address = SocketAddr::from(([127, 0, 0, 1], config.port + 1000));
    info!(
        "Binding metrics dashboard to address: {}",
        dashboard_address
    );
    // Start the metrics dashboard
    warp::serve(routes).bind(dashboard_address).await;

    info!("Metrics Dashboard Started at http://{}", dashboard_address);
}

//Periodically prints Metrics every 5 secs
async fn metrics_update_task(metrics: Arc<Mutex<Metrics>>) {
    let mut interval = tokio::time::interval(METRICS_UPDATE_INTERVAL);
    loop {
        interval.tick().await;
        let metrics = metrics.lock().unwrap();
        info!("Current metrics: {:?}", metrics);
    }
}

/// Shuts down the proxy server
pub fn shutdown_proxy_server() {
    info!("Shutting down proxy server...");
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::process::exit(0);
    });
}