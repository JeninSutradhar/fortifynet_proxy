// https://github.com/JeninSutradhar/fortifynet_proxy

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// Proxy server configuration
#[derive(Clone)]
pub struct ProxyConfig {
    /// IP address to bind the server to
    pub ip_address: String,
    /// Port number to bind the server to
    pub port: u16,
    /// Flag indicating whether authentication is required
    pub authentication: bool,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Flag indicating whether caching is enabled
    pub cache_enabled: bool,
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
        }
    }
}

/// Handles an incoming client connection
///
/// # Arguments
/// * `stream` - The client's TCP stream
/// * `config` - The proxy server's configuration
/// * `cache` - Cache for storing responses
pub fn handle_client(mut stream: TcpStream, config: &ProxyConfig, cache: Arc<Mutex<HashMap<String, Vec<u8>>>>) {
    // Check if authentication is required and handle authentication
    if config.authentication && !handle_authentication(&mut stream, &config) {
        return;
    }

    // Handle the HTTP request
    if let Err(err) = handle_http_request(&mut stream, config, cache) {
        eprintln!("Error handling HTTP request: {}", err);
    }
}

/// Handles authentication for incoming client connections
///
/// # Arguments
/// * `stream` - The client's TCP stream
/// * `config` - The proxy server's configuration
///
/// # Returns
/// A boolean indicating whether the authentication was successful
#[allow(unused)]
pub fn handle_authentication(stream: &mut TcpStream, config: &ProxyConfig) -> bool {
    let mut login_buffer = [0; 1024];

    // Read login data from the client
    if let Err(err) = stream.read(&mut login_buffer) {
        eprintln!("Error reading from stream: {}", err);
        return false;
    }

    let login_data = String::from_utf8_lossy(&login_buffer);

    // Check if the login data matches the configured username and password
    if login_data.contains(&format!("{}:{}", config.username, config.password)) {
        log_activity("Successful login");
        true
    } else {
        // If authentication fails, send a 401 Unauthorized response to the client
        if let Err(err) = stream.write(b"HTTP/1.1 401 Unauthorized\r\n\r\n") {
            eprintln!("Error writing to stream: {}", err);
        }
        log_activity("Failed login attempt");
        false
    }
}

/// Logs an activity
///
/// # Arguments
/// * `activity` - The activity to log
pub fn log_activity(activity: &str) {
    println!("{}", activity);
}

/// Handles an HTTP request
///
/// # Arguments
/// * `stream` - The client's TCP stream
/// * `config` - The proxy server's configuration
/// * `cache` - Cache for storing responses
///
/// # Returns
/// An `std::io::Result` indicating success or failure
#[allow(unused)]
pub fn handle_http_request(stream: &mut TcpStream, config: &ProxyConfig, cache: Arc<Mutex<HashMap<String, Vec<u8>>>>) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // Parse the request (simple parsing, expand as necessary)
    let request_str = String::from_utf8_lossy(&buffer);
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

    // Check cache
    if config.cache_enabled {
        let cache = cache.lock().unwrap();
        if let Some(response) = cache.get(url) {
            stream.write_all(response)?;
            stream.flush()?;
            return Ok(());
        }
    }

    // Forward the request to the target server (basic implementation)
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><h1>Your Server has been Started!</h1><h4>you can make additional changes to the code according to Requirements</h4><h2>Project Crate: <a>https://crates.io/crates/fortifynet_proxy</a><br>
        Github: https://github.com/JeninSutradhar/fortifynet_proxy<br>
        Github official: https://github.com/JeninSutradhar/</h2></body></html>"
    );

    // Update cache
    if config.cache_enabled {
        let mut cache = cache.lock().unwrap();
        cache.insert(url.to_string(), response.as_bytes().to_vec());
    }

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

/// Starts the proxy server
///
/// # Arguments
/// * `config` - The proxy server's configuration
#[allow(unused)]
pub fn start_proxy_server(config: &ProxyConfig) {
    // Bind a TcpListener to the specified IP address and port from the configuration.
    let listener = match TcpListener::bind(format!("{}:{}", config.ip_address, config.port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Error binding to address: {}", err);
            return;
        }
    };

    // Initialize a cache using a HashMap wrapped in Arc and Mutex for thread-safe access.
    let cache: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    // Enter a loop to accept incoming connections. For each connection, spawn a new thread to handle the client using the `handle_client` function.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cache_clone = Arc::clone(&cache);
                let config_clone = config.clone();

                thread::spawn(move || {
                    handle_client(stream, &config_clone, cache_clone);
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
    
    // Call the shutdown proxy server method
    shutdown_proxy_server();
}

/// Shuts down the proxy server
pub fn shutdown_proxy_server() {
    println!("Shutting down proxy server...");
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::process::exit(0);
    });
}

