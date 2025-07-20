//! Configuration management for NetaBase cross-machine testing
//!
//! This module provides safe parsing and validation of configuration from:
//! - Environment variables
//! - Command-line arguments
//! - Default values
//!
//! It supports both writer and reader node configurations with proper validation.

use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

/// Main configuration parser that handles both CLI and environment variables
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "netabase-test")]
pub struct Config {
    #[command(subcommand)]
    pub command: TestCommand,
}

#[derive(Subcommand, Debug)]
pub enum TestCommand {
    /// Run a writer node that stores records in the DHT
    Writer(WriterConfig),
    /// Run a reader node that retrieves records from the DHT
    Reader(ReaderConfig),
    /// Run a local test with both writer and reader
    Local(LocalConfig),
}

/// Writer node configuration
#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct WriterConfig {
    /// IP address and port to listen on
    #[arg(short = 'a', long = "addr", env = "NETABASE_WRITER_ADDR")]
    pub address: Option<String>,

    /// Test key to store records under
    #[arg(short = 'k', long = "key", env = "NETABASE_TEST_KEY")]
    pub test_key: Option<String>,

    /// Comma-separated values to store in the DHT
    #[arg(short = 'v', long = "values", env = "NETABASE_TEST_VALUES")]
    pub test_values: Option<String>,

    /// How long to keep the writer running (in seconds, 0 for indefinite)
    #[arg(short = 't', long = "timeout", env = "NETABASE_WRITER_TIMEOUT")]
    pub timeout: Option<u64>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Reader node configuration
#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    /// Writer address to connect to
    #[arg(short = 'c', long = "connect", env = "NETABASE_READER_CONNECT_ADDR")]
    pub connect_addr: Option<String>,

    /// Test key to retrieve records from
    #[arg(short = 'k', long = "key", env = "NETABASE_TEST_KEY")]
    pub test_key: Option<String>,

    /// Comma-separated expected values
    #[arg(short = 'v', long = "values", env = "NETABASE_TEST_VALUES")]
    pub test_values: Option<String>,

    /// Timeout for the test in seconds
    #[arg(short = 't', long = "timeout", env = "NETABASE_TEST_TIMEOUT")]
    pub timeout: Option<u64>,

    /// Number of retry attempts
    #[arg(short = 'r', long = "retries", env = "NETABASE_READER_RETRIES")]
    pub retries: Option<u32>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Local test configuration (combines writer and reader)
#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Test key to use for the test
    #[arg(short = 'k', long = "key", env = "NETABASE_TEST_KEY")]
    pub test_key: Option<String>,

    /// Comma-separated values to test
    #[arg(short = 'v', long = "values", env = "NETABASE_TEST_VALUES")]
    pub test_values: Option<String>,

    /// Timeout for the test in seconds
    #[arg(short = 't', long = "timeout", env = "NETABASE_TEST_TIMEOUT")]
    pub timeout: Option<u64>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Validated writer configuration with defaults applied
#[derive(Debug, Clone)]
pub struct ValidatedWriterConfig {
    pub address: SocketAddr,
    pub test_key: String,
    pub test_values: Vec<String>,
    pub timeout: Option<Duration>,
    pub verbose: bool,
}

/// Validated reader configuration with defaults applied
#[derive(Debug, Clone)]
pub struct ValidatedReaderConfig {
    pub connect_addr: SocketAddr,
    pub test_key: String,
    pub test_values: Vec<String>,
    pub timeout: Duration,
    pub retries: u32,
    pub verbose: bool,
}

/// Validated local test configuration
#[derive(Debug, Clone)]
pub struct ValidatedLocalConfig {
    pub test_key: String,
    pub test_values: Vec<String>,
    pub timeout: Duration,
    pub verbose: bool,
}

/// Environment-based configuration (fallback when not using CLI)
#[derive(Deserialize, Debug, Default)]
pub struct EnvConfig {
    pub netabase_writer_addr: Option<String>,
    pub netabase_reader_connect_addr: Option<String>,
    pub netabase_test_key: Option<String>,
    pub netabase_test_values: Option<String>,
    pub netabase_test_timeout: Option<String>,
    pub netabase_writer_timeout: Option<String>,
    pub netabase_reader_retries: Option<String>,
}

impl Config {
    /// Parse configuration from command-line arguments
    pub fn parse() -> Self {
        Self::parse_from(std::env::args())
    }

    /// Parse configuration from environment variables only
    pub fn from_env() -> Result<EnvConfig> {
        envy::from_env().context("Failed to parse environment variables")
    }
}

impl WriterConfig {
    /// Validate and apply defaults to writer configuration
    pub fn validate(self) -> Result<ValidatedWriterConfig> {
        let env_config = Config::from_env().unwrap_or_default();

        // Address with default
        let address_str = self
            .address
            .or(env_config.netabase_writer_addr)
            .unwrap_or_else(|| "0.0.0.0:9901".to_string());

        let address = parse_socket_addr(&address_str).context("Invalid writer address")?;

        // Test key with default
        let test_key = self
            .test_key
            .or(env_config.netabase_test_key)
            .unwrap_or_else(|| "cross_machine_key".to_string());

        // Test values with default
        let test_values_str = self
            .test_values
            .or(env_config.netabase_test_values)
            .unwrap_or_else(|| "Value1,Value2,Value3,HelloWorld".to_string());

        let test_values = parse_comma_separated(&test_values_str)?;

        // Timeout (optional)
        let timeout = if let Some(timeout_secs) = self.timeout {
            Some(Duration::from_secs(timeout_secs))
        } else if let Some(timeout_str) = env_config.netabase_writer_timeout {
            let secs = timeout_str
                .parse::<u64>()
                .context("Invalid writer timeout value")?;
            Some(Duration::from_secs(secs))
        } else {
            None // Run indefinitely by default
        };

        Ok(ValidatedWriterConfig {
            address,
            test_key,
            test_values,
            timeout,
            verbose: self.verbose,
        })
    }
}

impl ReaderConfig {
    /// Validate and apply defaults to reader configuration
    pub fn validate(self) -> Result<ValidatedReaderConfig> {
        let env_config = Config::from_env().unwrap_or_default();

        // Connect address with default
        let connect_addr_str = self
            .connect_addr
            .or(env_config.netabase_reader_connect_addr)
            .unwrap_or_else(|| "127.0.0.1:9901".to_string());

        let connect_addr =
            parse_socket_addr(&connect_addr_str).context("Invalid reader connect address")?;

        // Test key with default
        let test_key = self
            .test_key
            .or(env_config.netabase_test_key)
            .unwrap_or_else(|| "cross_machine_key".to_string());

        // Test values with default
        let test_values_str = self
            .test_values
            .or(env_config.netabase_test_values)
            .unwrap_or_else(|| "Value1,Value2,Value3,HelloWorld".to_string());

        let test_values = parse_comma_separated(&test_values_str)?;

        // Timeout with default
        let timeout_secs = if let Some(timeout) = self.timeout {
            timeout
        } else if let Some(timeout_str) = env_config.netabase_test_timeout {
            timeout_str
                .parse::<u64>()
                .context("Invalid timeout value")?
        } else {
            120 // Default 2 minutes
        };

        if timeout_secs == 0 {
            return Err(anyhow!("Timeout must be greater than 0"));
        }

        let timeout = Duration::from_secs(timeout_secs);

        // Retries with default
        let retries = if let Some(retries) = self.retries {
            retries
        } else if let Some(retries_str) = env_config.netabase_reader_retries {
            retries_str
                .parse::<u32>()
                .context("Invalid retries value")?
        } else {
            3 // Default 3 retries
        };

        Ok(ValidatedReaderConfig {
            connect_addr,
            test_key,
            test_values,
            timeout,
            retries,
            verbose: self.verbose,
        })
    }
}

impl LocalConfig {
    /// Validate and apply defaults to local test configuration
    pub fn validate(self) -> Result<ValidatedLocalConfig> {
        let env_config = Config::from_env().unwrap_or_default();

        // Test key with default
        let test_key = self
            .test_key
            .or(env_config.netabase_test_key)
            .unwrap_or_else(|| "cross_machine_key".to_string());

        // Test values with default
        let test_values_str = self
            .test_values
            .or(env_config.netabase_test_values)
            .unwrap_or_else(|| "Value1,Value2,Value3,HelloWorld".to_string());

        let test_values = parse_comma_separated(&test_values_str)?;

        // Timeout with default
        let timeout_secs = if let Some(timeout) = self.timeout {
            timeout
        } else if let Some(timeout_str) = env_config.netabase_test_timeout {
            timeout_str
                .parse::<u64>()
                .context("Invalid timeout value")?
        } else {
            60 // Default 1 minute for local tests
        };

        if timeout_secs == 0 {
            return Err(anyhow!("Timeout must be greater than 0"));
        }

        let timeout = Duration::from_secs(timeout_secs);

        Ok(ValidatedLocalConfig {
            test_key,
            test_values,
            timeout,
            verbose: self.verbose,
        })
    }
}

/// Parse a socket address from string, handling common formats
fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr> {
    // Try parsing directly first
    if let Ok(addr) = SocketAddr::from_str(addr_str) {
        return Ok(addr);
    }

    // If direct parsing fails, try to help with common issues
    if !addr_str.contains(':') {
        return Err(anyhow!(
            "Address must include port (e.g., '127.0.0.1:9901')"
        ));
    }

    // Try to resolve hostname if it's not an IP
    let parts: Vec<&str> = addr_str.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid address format: {}", addr_str));
    }

    let port_str = parts[0];
    let host_str = parts[1];

    let port: u16 = port_str.parse().context("Invalid port number")?;

    // For now, we'll be strict and require IP addresses
    // In the future, we could add hostname resolution
    match host_str.parse::<std::net::IpAddr>() {
        Ok(ip) => Ok(SocketAddr::new(ip, port)),
        Err(_) => Err(anyhow!(
            "Hostname resolution not supported yet. Please use IP addresses."
        )),
    }
}

/// Parse comma-separated values, trimming whitespace
fn parse_comma_separated(values_str: &str) -> Result<Vec<String>> {
    if values_str.trim().is_empty() {
        return Err(anyhow!("Test values cannot be empty"));
    }

    let values: Vec<String> = values_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if values.is_empty() {
        return Err(anyhow!("No valid test values found"));
    }

    Ok(values)
}

/// Utility function to create a writer config from environment variables only
pub fn writer_config_from_env() -> Result<ValidatedWriterConfig> {
    let config = WriterConfig {
        address: None,
        test_key: None,
        test_values: None,
        timeout: None,
        verbose: false,
    };
    config.validate()
}

/// Utility function to create a reader config from environment variables only
pub fn reader_config_from_env() -> Result<ValidatedReaderConfig> {
    let config = ReaderConfig {
        connect_addr: None,
        test_key: None,
        test_values: None,
        timeout: None,
        retries: None,
        verbose: false,
    };
    config.validate()
}

/// Utility function to create a local config from environment variables only
pub fn local_config_from_env() -> Result<ValidatedLocalConfig> {
    let config = LocalConfig {
        test_key: None,
        test_values: None,
        timeout: None,
        verbose: false,
    };
    config.validate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_addr_parsing() {
        // Valid addresses
        assert!(parse_socket_addr("127.0.0.1:9901").is_ok());
        assert!(parse_socket_addr("0.0.0.0:8080").is_ok());
        assert!(parse_socket_addr("192.168.1.1:1234").is_ok());

        // Invalid addresses
        assert!(parse_socket_addr("127.0.0.1").is_err()); // No port
        assert!(parse_socket_addr("invalid:port").is_err()); // Invalid format
        assert!(parse_socket_addr("127.0.0.1:99999").is_err()); // Invalid port
    }

    #[test]
    fn test_comma_separated_parsing() {
        // Valid inputs
        assert_eq!(parse_comma_separated("a,b,c").unwrap(), vec!["a", "b", "c"]);
        assert_eq!(
            parse_comma_separated("hello, world , test").unwrap(),
            vec!["hello", "world", "test"]
        );

        // Invalid inputs
        assert!(parse_comma_separated("").is_err()); // Empty
        assert!(parse_comma_separated("  ").is_err()); // Only whitespace
        assert!(parse_comma_separated(",,,").is_err()); // Only commas
    }

    #[test]
    fn test_config_validation() {
        // Test writer config with defaults
        let writer_config = WriterConfig {
            address: None,
            test_key: None,
            test_values: None,
            timeout: None,
            verbose: false,
        };

        let validated = writer_config.validate().unwrap();
        assert_eq!(validated.address.to_string(), "0.0.0.0:9901");
        assert_eq!(validated.test_key, "cross_machine_key");
        assert!(!validated.test_values.is_empty());

        // Test reader config with custom values
        let reader_config = ReaderConfig {
            connect_addr: Some("192.168.1.100:8080".to_string()),
            test_key: Some("custom_key".to_string()),
            test_values: Some("val1,val2".to_string()),
            timeout: Some(30),
            retries: Some(5),
            verbose: true,
        };

        let validated = reader_config.validate().unwrap();
        assert_eq!(validated.connect_addr.to_string(), "192.168.1.100:8080");
        assert_eq!(validated.test_key, "custom_key");
        assert_eq!(validated.test_values, vec!["val1", "val2"]);
        assert_eq!(validated.timeout, Duration::from_secs(30));
        assert_eq!(validated.retries, 5);
        assert!(validated.verbose);
    }
}
