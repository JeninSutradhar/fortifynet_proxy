// BASIC USAGE 
// Note this is the main file[if you are using the source code directly]
// ! adjust USE statement if you are using API

// mod lib;
use fortifynet_proxy::{start_proxy_server, ProxyConfig};

fn main() {
    
    // Create a proxy configuration with default values
    let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: false,
        username: "admin".to_string(),
        password: "password".to_string(),
        cache_enabled: true, // Disable for Faster Execution
    };

    // Start the proxy server with the provided configuration
    start_proxy_server(&config);
}
