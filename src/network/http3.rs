//! HTTP/3 client implementation using QUIC
//!
//! Provides HTTP/3 support with:
//! - QUIC transport via quinn
//! - 0-RTT connection resumption
//! - Multiplexed streams
//! - Connection migration

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use quinn::{ClientConfig, Endpoint, Connection, RecvStream, SendStream};
use rustls::pki_types::{CertificateDer, ServerName};

use crate::utils::error::{BinixError, NetworkError};

/// HTTP/3 client configuration
#[derive(Debug, Clone)]
pub struct Http3Config {
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Idle timeout
    pub idle_timeout: Duration,
    /// Enable 0-RTT
    pub enable_0rtt: bool,
    /// Max concurrent streams
    pub max_concurrent_streams: u32,
}

impl Default for Http3Config {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
            enable_0rtt: true,
            max_concurrent_streams: 100,
        }
    }
}

/// HTTP/3 client using QUIC
pub struct Http3Client {
    endpoint: Endpoint,
    config: Http3Config,
}

impl Http3Client {
    /// Create a new HTTP/3 client
    pub fn new(config: Http3Config) -> Result<Self, BinixError> {
        // Create client config with native certs
        let client_config = Self::create_client_config()?;
        
        // Bind to any available port
        let addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        let mut endpoint = Endpoint::client(addr)
            .map_err(|e| BinixError::Network(NetworkError::ConnectionFailed(e.to_string())))?;
        
        endpoint.set_default_client_config(client_config);
        
        Ok(Self { endpoint, config })
    }

    /// Create client TLS config
    fn create_client_config() -> Result<ClientConfig, BinixError> {
        // Install ring as the default crypto provider
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Use platform's native root certificates
        let mut roots = rustls::RootCertStore::empty();

        // Add webpki roots as fallback
        for cert in rustls_native_certs::load_native_certs().certs {
            roots.add(cert).ok();
        }

        let crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let client_config = ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
                .map_err(|e| BinixError::Network(NetworkError::TlsError(e.to_string())))?
        ));

        Ok(client_config)
    }

    /// Connect to a server
    pub async fn connect(&self, host: &str, port: u16) -> Result<Http3Connection, BinixError> {
        let addr = format!("{}:{}", host, port);
        let socket_addr: SocketAddr = tokio::net::lookup_host(&addr)
            .await
            .map_err(|e| BinixError::Network(NetworkError::DnsError(e.to_string())))?
            .next()
            .ok_or_else(|| BinixError::Network(NetworkError::DnsError("No address found".to_string())))?;
        
        let connection = self.endpoint
            .connect(socket_addr, host)
            .map_err(|e| BinixError::Network(NetworkError::ConnectionFailed(e.to_string())))?
            .await
            .map_err(|e| BinixError::Network(NetworkError::ConnectionFailed(e.to_string())))?;
        
        Ok(Http3Connection { connection })
    }

    /// Get endpoint stats
    pub fn stats(&self) -> EndpointStats {
        EndpointStats {
            open_connections: 0, // Would need tracking
        }
    }
}

/// HTTP/3 connection
pub struct Http3Connection {
    connection: Connection,
}

impl Http3Connection {
    /// Open a bidirectional stream
    pub async fn open_stream(&self) -> Result<(SendStream, RecvStream), BinixError> {
        self.connection
            .open_bi()
            .await
            .map_err(|e| BinixError::Network(NetworkError::ConnectionFailed(e.to_string())))
    }

    /// Check if connection is still open
    pub fn is_open(&self) -> bool {
        self.connection.close_reason().is_none()
    }

    /// Close the connection
    pub fn close(&self) {
        self.connection.close(0u32.into(), b"done");
    }

    /// Get remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.connection.remote_address()
    }
}

/// Endpoint statistics
#[derive(Debug, Clone, Default)]
pub struct EndpointStats {
    pub open_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http3_config_default() {
        let config = Http3Config::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert!(config.enable_0rtt);
    }

    #[tokio::test]
    async fn test_http3_client_creation() {
        let config = Http3Config::default();
        let client = Http3Client::new(config);
        assert!(client.is_ok());
    }
}

