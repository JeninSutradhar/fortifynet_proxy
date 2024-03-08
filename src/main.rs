use fortifynet_proxy::{ProxyConfig, start_proxy_server};

fn main() {
    // Define proxy server configuration
    let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: true,
        username: "admin".to_string(),
        password: "password123".to_string(),
        cache_enabled: true,
    };

    // Start the proxy server with the provided configuration
    start_proxy_server(config);
}