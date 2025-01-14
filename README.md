# 🛡️ FortifyNet Proxy v2

[![crates.io](https://img.shields.io/crates/v/fortifynet_proxy.svg)](https://crates.io/crates/fortifynet_proxy)
[![docs.rs](https://docs.rs/fortifynet_proxy/badge.svg)](https://docs.rs/fortifynet_proxy)
[![GitHub](https://img.shields.io/github/stars/JeninSutradhar/fortifynet_proxy?style=social)](https://github.com/JeninSutradhar/fortifynet_proxy)

**FortifyNet Proxy** is a powerful and flexible asynchronous proxy server library written in Rust. It's designed to be a robust and reusable foundation for building various types of proxy servers, including HTTP, HTTPS, and SOCKS5, with a focus on performance, security, and ease of use.

## Features

*   **Asynchronous Architecture:** Built using `tokio` for handling numerous concurrent connections with optimal efficiency.
*   **HTTP/HTTPS Proxying:** Seamlessly forwards HTTP and HTTPS traffic, ensuring compatibility and security using `hyper` and `tokio-rustls`.
*   **SOCKS5 Proxy Support:** Capable of routing traffic through SOCKS5 proxies using `tokio-socks`, enabling advanced network configurations.
*   **Request Caching:** Implements an in-memory cache to store responses for frequently accessed resources to reduce load and improve response times.
*   **Real-Time Metrics:** Provides built-in real-time traffic statistics, response time analysis, and error tracking.
*   **Dashboard:** Includes a simple web-based dashboard using `warp` for live monitoring of the server.
*   **Highly Configurable:** Offers a flexible `ProxyConfig` struct to customize various proxy server settings.
*   **Authentication:** Supports basic username/password authentication for controlling proxy access.
*   **Dynamic Target Resolution**: Resolves the host address from the request and redirects the request dynamically.

## 📦 Installation

To use FortifyNet Proxy in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
fortifynet_proxy = "2.0.0" # Or the latest Version
tokio = { version = "1", features = ["full"] }
hyper = { version = "0.14", features = ["client","http1","server","tcp"] }
log = "0.4"
env_logger = "0.10"
thiserror = "1"
anyhow = "1"
rustls = "0.21"
tokio-rustls = "0.24"
tokio-socks = "0.3"
url = "2.5"
warp = "0.3"
rustls-pemfile = "1.1"
```

## Basic Usage

Here's how you can quickly set up a basic HTTP proxy server with FortifyNet Proxy:

1.  **Set up your main file (`main.rs`)**

    ```rust
    use fortifynet_proxy::{start_proxy_server, ProxyConfig};
    use log::info;

    #[tokio::main]
    async fn main() -> anyhow::Result<()> {

        // Create a proxy configuration with default values
        let config = ProxyConfig {
            ip_address: "127.0.0.1".to_string(),
            port: 8080,
            authentication: false,
            username: "admin".to_string(),
            password: "password".to_string(),
            cache_enabled: true,
            socks5_address: None,
            https_enabled: false,
            certificate_path: None,
            private_key_path: None,
            target_address: Some("http://localhost".to_string()) // target for non-socks connection
        };
        info!("Starting Proxy server with configuration: {:?}", config);
        // Start the proxy server with the provided configuration
        start_proxy_server(config).await?;
        Ok(())
    }
    ```

2.  **Start the Proxy Server:**

    Run `cargo run` in your terminal. The proxy server will start listening for connections. You can monitor the server's output for logs and metrics.

3.  **Configure Your Client:**

    *   **Web Browser:**
        *   **Proxy Type:** HTTP
        *   **Address:** `127.0.0.1`
        *   **Port:** `8080` (or the port you configured)

    *   **`curl` Command:**

        ```bash
        curl -v --proxy http://127.0.0.1:8080 http://www.example.com
        ```
    * **HTTPS Proxy:**
       If you enable `https_enabled` then your proxy will be listening to https requests and you need to configure your client for https proxy, you also need to create the certificate and key files.
    ```bash
    curl -v --proxy https://127.0.0.1:8080 https://www.example.com
    ```

4.  **Access Metrics Dashboard**
     Open a web browser and navigate to `http://127.0.0.1:<port + 1000>`. For the above example, this is `http://127.0.0.1:9080`.

## Advanced Usage

### Enabling Authentication

To secure your proxy server with basic authentication:

```rust
  let config = ProxyConfig {
	    ip_address: "127.0.0.1".to_string(),
	    port: 8080,
	    authentication: true, // Enable authentication
	    username: "admin".to_string(),
	    password: "password".to_string(),
	    cache_enabled: true,
	    socks5_address: None,
	    https_enabled: false,
		certificate_path: None,
		private_key_path: None,
		target_address: None,
};
```
When `authentication` is enabled, the user will have to provide the authentication header.

### Using a SOCKS5 Proxy

To forward your requests through a SOCKS5 proxy:

```rust
let config = ProxyConfig {
    ip_address: "127.0.0.1".to_string(),
    port: 8080,
    authentication: false,
    username: "".to_string(),
    password: "".to_string(),
    cache_enabled: true,
    socks5_address: Some("127.0.0.1:1080".to_string()), // Using SOCKS5
	https_enabled: false,
	certificate_path: None,
	private_key_path: None,
	target_address: None,
};
```

*   Use the `curl` command with the `--socks5` option.

    ```bash
      curl -v --socks5 127.0.0.1:1080 http://www.example.com
    ```

### Enabling HTTPS Support

To enable HTTPS for secure connections, you need to specify the certificate and key file paths

```rust
 let config = ProxyConfig {
        ip_address: "127.0.0.1".to_string(),
        port: 8080,
        authentication: false,
        username: "".to_string(),
        password: "".to_string(),
        cache_enabled: true,
        socks5_address: None,
        https_enabled: true,    // Enable HTTPS
        certificate_path: Some("cert.pem".to_string()),
        private_key_path: Some("key.pem".to_string()),
        target_address: None,
};
```
You will also need to generate your own certificates and key files.
```bash
     openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
```
*   **Important Note**: Always use valid certificates from a trusted CA in production environments.

### Specify Target Address

If you are not using SOCKS5 and want to use direct connection and forward your request to a specific address you can use the `target_address` field.

```rust
 let config = ProxyConfig {
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
        target_address: Some("http://www.google.com".to_string()),
    };
```

### Advanced Configuration Options

The `ProxyConfig` struct offers several configuration options, allowing you to customize your proxy server:

*   `ip_address`: Binds the proxy to a specific IP address (e.g., "0.0.0.0" for all interfaces).
*   `port`: Specifies the port on which the proxy server listens.
*   `authentication`: Enables or disables basic authentication for the proxy.
*   `username` and `password`: Set the username and password for authentication (if enabled).
*   `cache_enabled`: Enables or disables response caching.
*   `socks5_address`: Sets an optional SOCKS5 server address for routing traffic.
*   `https_enabled`: Enables or disables HTTPS support.
*   `certificate_path` and `private_key_path`: Set the paths to the SSL certificates and key file if HTTPS is enabled.
*   `target_address`: Sets the target address for direct connections.

## Real-Time Metrics and Monitoring

*   **Live Metrics**: Access the dashboard at `http://127.0.0.1:<port + 1000>` in your browser to view real-time metrics about the proxy server, including total requests, average response times, cache hit/miss rates, and error counts.
*   **Console Logs**: Check the console output where the proxy server is running for detailed logs of incoming connections, requests, responses, and any errors encountered.

## Improvements from Previous Versions

*   **Dynamic Target Resolution**: Previously the proxy would forward all requests to a static target, but now it forwards the request to the actual target specified in the request URL.
*   **Corrected Direct Connection Handling**: Fixed the issues for direct connection by creating a complete URL.
*   **Corrected SOCKS5 Connections**: Fixed the SOCKS5 connect logic by using host and port separately.
*   **Improved Caching**: Caching logic is improved to make the proxy more robust.
*   **Enhanced Error Handling**: Improved error handling with `anyhow` and comprehensive logging.
*   **Code Clarity**: Improved the readability of code and proper documentation.
*   **Metrics**: Metrics has been implemented to get the required data regarding the traffic.
*   **Dashboard**: Dashboard has been implemented to get view the metrics.
*   **Documentation**: A comprehensive documentation is added to explain every feature.

## 🔗 Further Resources

*   **Project Crate**: [https://crates.io/crates/fortifynet_proxy](https://crates.io/crates/fortifynet_proxy)
*   **GitHub Repository**: [https://github.com/JeninSutradhar/fortifynet_proxy](https://github.com/JeninSutradhar/fortifynet_proxy)
*   **GitHub Official**: [https://github.com/JeninSutradhar/](https://github.com/JeninSutradhar/)

## ⚖️ License

Licensed under the MIT License. See the `LICENSE` file for details.
