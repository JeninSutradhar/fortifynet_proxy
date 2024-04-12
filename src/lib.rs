// jeninsutradhar@gmail.com
// https://github.com/JeninSutradhar/fortifynet_proxy

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProxyConfig {
    pub ip_address: String,
    pub port: u16,
    pub authentication: bool,
    pub username: String,
    pub password: String,
    pub cache_enabled: bool,
}

// Implementing Default for ProxyConfig
impl Default for ProxyConfig {
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

/// Handles an incoming client connection by reading the request and
/// sending a response.
///
/// # Arguments
/// * `stream` - The client's TCP stream
/// * `config` - The proxy server's configuration
pub fn handle_client(mut stream: TcpStream, config: &ProxyConfig) {
    let mut buffer = [0; 1024];
    // Read from the stream
    if let Err(err) = stream.read(&mut buffer) {
        eprintln!("Error reading from stream: {}", err);
        return;
    }

    // Handle authentication if it's enabled
    if config.authentication && !handle_authentication(&mut stream, &config) {
        return;
    }

    // Handle the HTTP request
    if let Err(err) = handle_http_request(&mut stream) {
        eprintln!("Error handling HTTP request: {}", err);
    }
}


// Authentication Handling
/// Handles authentication for incoming client connections.
///
/// # Arguments
/// * `stream` - The client's TCP stream
/// * `config` - The proxy server's configuration
///
/// # Returns
/// A boolean indicating whether the authentication was successful
pub fn handle_authentication(stream: &mut TcpStream, config: &ProxyConfig) -> bool {
    let mut login_buffer = [0; 1024];
    if let Err(err) = stream.read(&mut login_buffer) {
        eprintln!("Error reading from stream: {}", err);
        return false;
    }

    let login_data = String::from_utf8_lossy(&login_buffer);
    if login_data.contains(&format!("{}:{}", config.username, config.password)) {
        log_activity("Successful login");
        true
    } else {
        if let Err(err) = stream.write(b"HTTP/1.1 401 Unauthorized\r\n\r\n") {
            eprintln!("Error writing to stream: {}", err);
        }
        log_activity("Failed login attempt");
        false
    }
}


// Handling HTTP request
pub fn handle_http_request(mut stream: &TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><h1>Hello, World!</h1></body></html>";
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn log_activity(activity: &str) {
    println!("{}", activity);
}

#[allow(unused)]
// Start Server
pub fn start_proxy_server(config: ProxyConfig) {
    let listener = match TcpListener::bind(format!("{}:{}", config.ip_address, config.port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Error binding to address: {}", err);
            return;
        }
    };
    
    // Collect
    let cache: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    // #[derive(Clone)] used for .clone in config_clone
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cache_clone = Arc::clone(&cache);
                let config_clone = config.clone();
                thread::spawn(move || {
                    handle_client(stream, &config_clone);
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }

    // Shutdown Server
    shutdown_proxy_server();
}

pub fn shutdown_proxy_server() {
    println!("Shutting down proxy server...");
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::process::exit(0);
    });
}


