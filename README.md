# FortifyNet Proxy

FortifyNet Proxy is a lightweight Rust proxy server designed to provide secure and efficient handling of HTTP requests with basic authentication and resource caching capabilities.

## Features

- **Proxy Authentication:** Securely authenticate users before allowing access to resources.
- **HTTP Request Handling:** Efficiently handle HTTP requests and generate appropriate responses.
- **Activity Logging:** Log proxy server activities for monitoring and troubleshooting.
- **Resource Caching:** Cache frequently accessed resources to optimize performance.
- **Graceful Shutdown:** Gracefully shutdown the proxy server to ensure data integrity and user experience.

## Installation

To use FortifyNet Proxy in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
fortifynet_proxy = "1.1.5"
```

# Usage

## Simple Use Case
- To quickly set up the FortifyNet Proxy Server with default settings:

1. Import the start_proxy_server function from the fortifynet_proxy crate.
2. Call the start_proxy_server function without providing any configuration parameters.

```rust
use fortifynet_proxy::start_proxy_server;

fn main() {
    // Start the proxy server with default settings
    start_proxy_server();
}
```
In this usage scenario, the proxy server starts with default settings, including:
- IP address: "127.0.0.1"
- Port: 8080
- No authentication required
- No resource caching enabled


## Basic Usage
To use the FortifyNet Proxy Server, follow these simple steps:

- Define a ProxyConfig struct to specify the server configuration parameters.
- Start the proxy server using the start_proxy_server function with the provided configuration.

```rust
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
```

# Advanced Usage

## Customized Authentication
Configure custom authentication settings to enforce user access control.

```rust
use fortifynet_proxy::{ProxyConfig, start_proxy_server};

fn main() {
    // Configure custom authentication
    let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: true,
        username: "admin".to_string(),
        password: "password123".to_string(),
        cache_enabled: true,
    };

    // Start the proxy server with custom authentication
    start_proxy_server(config);
}
```

## Resource Caching Strategies
Implement resource caching strategies to optimize network performance.

```rust
use fortifynet_proxy::{ProxyConfig, start_proxy_server};

fn main() {
    // Configure resource caching
    let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: false,
        username: "".to_string(),
        password: "".to_string(),
        cache_enabled: true,
    };

    // Start the proxy server with resource caching enabled
    start_proxy_server(config);
}
```

## Graceful Shutdown Handling
Gracefully shutdown the proxy server to ensure ongoing connections are completed.

```rust
use fortifynet_proxy::{ProxyConfig, start_proxy_server};

fn main() {
    // Configure proxy server
    let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: false,
        username: "".to_string(),
        password: "".to_string(),
        cache_enabled: true,
    };

    // Start the proxy server
    let server_thread = start_proxy_server(config);

    // Gracefully shutdown the server after a specified time
    std::thread::sleep(std::time::Duration::from_secs(10));
    server_thread.join().expect("Failed to join server thread");
}
```