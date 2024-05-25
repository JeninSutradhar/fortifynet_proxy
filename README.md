# FortifyNet Proxy: Secure and Efficient Rust Proxy Server

FortifyNet Proxy is a lightweight Rust proxy server designed to provide secure and efficient handling of HTTP requests with basic authentication and resource caching capabilities.

[![Crates.io](https://img.shields.io/crates/v/fortifynet_proxy.svg)](https://crates.io/crates/fortifynet_proxy)
[![Docs.rs](https://docs.rs/fortifynet_proxy/badge.svg)](https://docs.rs/fortifynet_proxy)
- Github Clone : `git clone https://github.com/JeninSutradhar/fortifynet_proxy`

## Features

1. **Proxy Authentication:** Securely authenticate users before allowing access to resources.
2. **HTTP Request Forwarding**: Forwards incoming HTTP requests to target servers and relays the responses back to the clients.
3. **Activity Logging:** Log proxy server activities for monitoring and troubleshooting.
4. **Caching**: Caches responses for repeated requests to reduce load on target servers and improve response times.
5. **Graceful Shutdown:** Gracefully shutdown the proxy server to ensure data integrity and user experience.


## Installation

To use FortifyNet Proxy in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
fortifynet_proxy = "1.1.9"
```

# Usage

## Basic Usage
To use the FortifyNet Proxy Server, follow these simple steps:

- Define a ProxyConfig struct to specify the server configuration parameters.
- Start the proxy server using the start_proxy_server function with the provided configuration.

```rust
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
```
## Customization:
FortifyNet Proxy offers extensive configuration options:

- **IP Address and Port:** Specify the desired IP address and port for the server.
- **Authentication:** Enable user authentication with custom usernames and passwords.
- **Resource Caching:** Implement caching strategies to store frequently accessed resources and improve performance.


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

## Logging Implementation:
Modify the handle_http_request function to add logging:

```rust
pub fn handle_http_request(stream: &mut TcpStream, config: &ProxyConfig, cache: Arc<Mutex<HashMap<String, Vec<u8>>>>) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request_str = String::from_utf8_lossy(&buffer);
    log_activity(&format!("Incoming request: {}", request_str));

    let request_lines: Vec<&str> = request_str.lines().collect();
    if request_lines.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid HTTP request"));
    }

    let first_line = request_lines[0];
    let request_parts: Vec<&str> = first_line.split_whitespace().collect();
    if request_parts.len() != 3 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid HTTP request line"));
    }

    let method = request_parts[0];
    let url = request_parts[1];
    let version = request_parts[2];

    let host = if let Some(host_line) = request_lines.iter().find(|&&line| line.starts_with("Host:")) {
        host_line.split_whitespace().nth(1).unwrap_or("")
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No Host header found"));
    };

    if config.cache_enabled {
        let cache = cache.lock().unwrap();
        if let Some(response) = cache.get(url) {
            log_activity(&format!("Serving from cache: {}", url));
            stream.write_all(response)?;
            stream.flush()?;
            return Ok(());
        }
    }

    let target_address = format!("{}:80", host);
    let mut target_stream = TcpStream::connect(target_address)?;

    target_stream.write_all(buffer)?;
    target_stream.flush()?;

    let mut response_buffer = Vec::new();
    target_stream.read_to_end(&mut response_buffer)?;

    if config.cache_enabled {
        let mut cache = cache.lock().unwrap();
        cache.insert(url.to_string(), response_buffer.clone());
    }

    log_activity(&format!("Outgoing response: {:?}", response_buffer));
    stream.write_all(&response_buffer)?;
    stream.flush()?;
    Ok(())
}

pub fn log_activity(activity: &str) {
    println!("{}", activity);
}
```

# Further Resources:
- Project Crate: https://crates.io/crates/fortifynet_proxy
- Github: https://github.com/JeninSutradhar/fortifynet_proxy
- Github official: https://github.com/JeninSutradhar/