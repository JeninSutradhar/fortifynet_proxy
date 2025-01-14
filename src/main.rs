use fortifynet_proxy::{start_proxy_server, ProxyConfig};
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a proxy configuration with default values
    let config = ProxyConfig {
        // The IP address the proxy server will bind to
        ip_address: "127.0.0.1".to_string(),
        // The port the proxy server will listen on
        port: 1234,
        // Whether to enable basic authentication
        authentication: false,
        // The username and password to be used for authentication
        username: "admin".to_string(),
        password: "password".to_string(),
        // Whether to enable response caching
        cache_enabled: false,
        // The SOCKS5 server to use for proxying
        socks5_address: None,
        // Whether to enable HTTPS
        https_enabled: false,
        // The path to the SSL/TLS certificate file
        certificate_path: Some("cert.pem".to_string()),
        // The path to the SSL/TLS private key file
        private_key_path: Some("key.pem".to_string()),
        // The target address to proxy requests to
        target_address: Some("http://www.google.com".to_string()), // Set the target address
    };
    info!("Starting Proxy server with configuration: {:?}", config);
    // Start the proxy server with the provided configuration
    start_proxy_server(config).await?;
    Ok(())
}
