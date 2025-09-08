use crate::netabase_trait::NetabaseSchema;
use crate::traits;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Result type for configuration operations
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;

/// Errors that can occur during configuration operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid configuration format: {message}")]
    InvalidFormat { message: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Environment variable error: {var} - {message}")]
    EnvironmentError { var: String, message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Configuration is locked and cannot be modified")]
    ConfigurationLocked,

    #[error("Invalid configuration key: {key}")]
    InvalidKey { key: String },

    #[error("Type conversion error: {message}")]
    TypeConversionError { message: String },

    #[error("Configuration source error: {source_name} - {message}")]
    SourceError {
        source_name: String,
        message: String,
    },
}

/// Sources from which configuration can be loaded
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationSource {
    /// Load from a file
    File { path: String, format: FileFormat },
    /// Load from environment variables
    Environment { prefix: Option<String> },
    /// Load from command line arguments
    CommandLine,
    /// Use default values
    Default,
    /// Load from a remote URL
    Remote { url: String },
    /// Load from memory/inline configuration
    Memory { data: HashMap<String, String> },
}

/// Supported configuration file formats
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormat {
    Json,
    Yaml,
    Toml,
    Ini,
}

/// Configuration validation levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationLevel {
    None,
    Basic,
    Strict,
    Custom,
}

/// Configuration change event
#[derive(Debug, Clone)]
pub enum ConfigurationEvent {
    /// Configuration was loaded
    Loaded { source: ConfigurationSource },
    /// Configuration was updated
    Updated {
        source: ConfigurationSource,
        changes: Vec<ConfigurationChange>,
    },
    /// Configuration validation failed
    ValidationFailed { errors: Vec<ConfigurationError> },
    /// Configuration file was modified
    FileModified { path: String },
    /// Configuration reload failed
    ReloadFailed { error: ConfigurationError },
}

/// Represents a change in configuration
#[derive(Debug, Clone)]
pub struct ConfigurationChange {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub source: ConfigurationSource,
}

/// Configuration metadata
#[derive(Debug, Clone)]
pub struct ConfigurationMetadata {
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub source: ConfigurationSource,
    pub version: String,
    pub checksum: String,
    pub is_valid: bool,
    pub validation_errors: Vec<ConfigurationError>,
}

/// Configuration builder options
#[derive(Debug, Clone)]
pub struct ConfigurationOptions {
    pub sources: Vec<ConfigurationSource>,
    pub validation_level: ValidationLevel,
    pub watch_for_changes: bool,
    pub auto_reload: bool,
    pub merge_strategy: MergeStrategy,
    pub environment_prefix: Option<String>,
    pub required_fields: Vec<String>,
    pub default_values: HashMap<String, String>,
}

/// Strategies for merging configurations from multiple sources
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Later sources override earlier ones
    Override,
    /// Merge nested objects, override primitives
    DeepMerge,
    /// Only use the first source that provides a value
    FirstWins,
    /// Use custom merge logic
    Custom,
}

impl Default for ConfigurationOptions {
    fn default() -> Self {
        Self {
            sources: vec![ConfigurationSource::Default],
            validation_level: ValidationLevel::Basic,
            watch_for_changes: false,
            auto_reload: false,
            merge_strategy: MergeStrategy::Override,
            environment_prefix: None,
            required_fields: vec![],
            default_values: HashMap::new(),
        }
    }
}

/// Core configuration management trait
#[async_trait]
pub trait NetabaseConfiguration: Send + Sync {
    /// Load configuration from the specified sources
    async fn load(&mut self, options: ConfigurationOptions) -> ConfigurationResult<()>;

    /// Reload configuration from all sources
    async fn reload(&mut self) -> ConfigurationResult<()>;

    /// Save configuration to the specified destination
    async fn save<P: AsRef<Path> + Send>(
        &self,
        path: P,
        format: FileFormat,
    ) -> ConfigurationResult<()>;

    /// Validate the current configuration
    async fn validate(&self) -> ConfigurationResult<Vec<ConfigurationError>>;

    /// Check if the configuration is valid
    fn is_valid(&self) -> bool;

    /// Get configuration metadata
    fn metadata(&self) -> ConfigurationMetadata;

    /// Get a configuration value by key
    fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ConfigurationResult<Option<T>>;

    /// Set a configuration value by key
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> ConfigurationResult<()>;

    /// Check if a key exists in the configuration
    fn contains_key(&self, key: &str) -> bool;

    /// Get all configuration keys
    fn keys(&self) -> Vec<String>;

    /// Get all configuration values as a flattened map
    fn to_map(&self) -> HashMap<String, String>;

    /// Clear all configuration values
    fn clear(&mut self);

    /// Merge another configuration into this one
    fn merge(&mut self, other: &Self, strategy: MergeStrategy) -> ConfigurationResult<()>;

    /// Create a snapshot of the current configuration
    fn snapshot(&self) -> ConfigurationResult<Self>
    where
        Self: Clone;

    /// Restore from a configuration snapshot
    fn restore(&mut self, snapshot: Self) -> ConfigurationResult<()>;

    /// Subscribe to configuration changes
    async fn subscribe_changes(
        &self,
    ) -> ConfigurationResult<tokio::sync::broadcast::Receiver<ConfigurationEvent>>;

    /// Start watching for configuration file changes
    async fn start_watching(&mut self) -> ConfigurationResult<()>;

    /// Stop watching for configuration file changes
    async fn stop_watching(&mut self) -> ConfigurationResult<()>;

    /// Export configuration in the specified format
    fn export(&self, format: FileFormat) -> ConfigurationResult<String>;

    /// Import configuration from a string
    fn import(&mut self, data: &str, format: FileFormat) -> ConfigurationResult<()>;
}

/// Extension trait for typed configuration access
#[async_trait]
pub trait NetabaseConfigurationExt: NetabaseConfiguration {
    /// Get a required configuration value (fails if not present)
    fn get_required<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ConfigurationResult<T>;

    /// Get a configuration value with a default
    fn get_or_default<T: for<'de> Deserialize<'de> + Default>(&self, key: &str) -> T;

    /// Get a configuration value with a custom default
    fn get_or<T: for<'de> Deserialize<'de>>(&self, key: &str, default: T) -> T;

    /// Set a configuration value if it doesn't exist
    fn set_if_missing<T: Serialize>(&mut self, key: &str, value: T) -> ConfigurationResult<bool>;

    /// Update a configuration value using a closure
    fn update<T, F>(&mut self, key: &str, updater: F) -> ConfigurationResult<()>
    where
        T: for<'de> Deserialize<'de> + Serialize,
        F: FnOnce(T) -> T;

    /// Get a nested configuration section
    fn get_section(&self, section: &str) -> ConfigurationResult<Self>
    where
        Self: Default;

    /// Set an entire configuration section
    fn set_section(&mut self, section: &str, config: &Self) -> ConfigurationResult<()>;

    /// Delete a configuration key
    fn delete(&mut self, key: &str) -> ConfigurationResult<bool>;

    /// Get configuration as a specific struct type
    fn to_struct<T: for<'de> Deserialize<'de>>(&self) -> ConfigurationResult<T>;

    /// Update configuration from a struct
    fn from_struct<T: Serialize>(&mut self, value: &T) -> ConfigurationResult<()>;
}

/// Trait for configuration validation
#[async_trait]
pub trait ConfigurationValidator: Send + Sync {
    /// Validate a configuration value
    async fn validate_value(&self, key: &str, value: &str) -> ConfigurationResult<()>;

    /// Validate the entire configuration
    async fn validate_all(&self, config: &Self) -> ConfigurationResult<Vec<ConfigurationError>>;

    /// Add a custom validation rule
    fn add_rule(&mut self, key: &str, rule: String) -> ConfigurationResult<()>;

    /// Remove a validation rule
    fn remove_rule(&mut self, key: &str) -> ConfigurationResult<bool>;

    /// Get all validation rules
    fn rules(&self) -> HashMap<String, String>;
}

/// Trait for individual validation rules
pub trait ValidationRule: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &str) -> ConfigurationResult<()>;

    /// Get the rule description
    fn description(&self) -> &str;

    /// Check if the rule is required
    fn is_required(&self) -> bool {
        false
    }
}

/// Common validation rules
pub struct RequiredRule;
pub struct RangeRule<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}
pub struct RegexRule {
    pub pattern: String,
}
pub struct EnumRule {
    pub allowed_values: Vec<String>,
}
pub struct CustomRule<F: Fn(&str) -> ConfigurationResult<()> + Send + Sync> {
    pub validator: F,
    pub description: String,
}

impl ValidationRule for RequiredRule {
    fn validate(&self, value: &str) -> ConfigurationResult<()> {
        if value.trim().is_empty() {
            Err(ConfigurationError::ValidationError {
                field: "value".to_string(),
                message: "Value is required".to_string(),
            })
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        "Value is required and cannot be empty"
    }

    fn is_required(&self) -> bool {
        true
    }
}

impl<T> ValidationRule for RangeRule<T>
where
    T: std::str::FromStr + PartialOrd + std::fmt::Display + Send + Sync,
    T::Err: std::fmt::Display,
{
    fn validate(&self, value: &str) -> ConfigurationResult<()> {
        let parsed: T = value
            .parse()
            .map_err(|e| ConfigurationError::TypeConversionError {
                message: format!("Cannot parse value '{}': {}", value, e),
            })?;

        if let Some(ref min) = self.min {
            if parsed < *min {
                return Err(ConfigurationError::ValidationError {
                    field: "value".to_string(),
                    message: format!("Value {} is less than minimum {}", parsed, min),
                });
            }
        }

        if let Some(ref max) = self.max {
            if parsed > *max {
                return Err(ConfigurationError::ValidationError {
                    field: "value".to_string(),
                    message: format!("Value {} is greater than maximum {}", parsed, max),
                });
            }
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "Value must be within the specified range"
    }
}

impl ValidationRule for RegexRule {
    fn validate(&self, value: &str) -> ConfigurationResult<()> {
        let regex =
            regex::Regex::new(&self.pattern).map_err(|e| ConfigurationError::ValidationError {
                field: "pattern".to_string(),
                message: format!("Invalid regex pattern: {}", e),
            })?;

        if !regex.is_match(value) {
            return Err(ConfigurationError::ValidationError {
                field: "value".to_string(),
                message: format!(
                    "Value '{}' does not match pattern '{}'",
                    value, self.pattern
                ),
            });
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "Value must match the specified regular expression"
    }
}

impl ValidationRule for EnumRule {
    fn validate(&self, value: &str) -> ConfigurationResult<()> {
        if !self.allowed_values.contains(&value.to_string()) {
            return Err(ConfigurationError::ValidationError {
                field: "value".to_string(),
                message: format!(
                    "Value '{}' is not one of the allowed values: {:?}",
                    value, self.allowed_values
                ),
            });
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Value must be one of the allowed values"
    }
}

impl<
    F: for<'a> Fn(&'a str) -> Result<(), traits::configuration::ConfigurationError>
        + std::marker::Send
        + std::marker::Sync,
> ValidationRule for CustomRule<F>
{
    fn validate(&self, value: &str) -> ConfigurationResult<()> {
        (self.validator)(value)
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Trait for configuration providers (sources)
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    /// Load configuration from this provider
    async fn load(&self) -> ConfigurationResult<HashMap<String, String>>;

    /// Check if this provider supports watching for changes
    fn supports_watching(&self) -> bool;

    /// Start watching for changes (if supported)
    async fn start_watching(
        &mut self,
    ) -> ConfigurationResult<tokio::sync::broadcast::Receiver<ConfigurationEvent>>;

    /// Stop watching for changes
    async fn stop_watching(&mut self) -> ConfigurationResult<()>;

    /// Get provider metadata
    fn metadata(&self) -> HashMap<String, String>;

    /// Check if the provider is available
    async fn is_available(&self) -> bool;
}

/// Builder for creating configuration instances
pub struct ConfigurationBuilder<P: ConfigurationProvider, V: ConfigurationValidator> {
    options: ConfigurationOptions,
    providers: Vec<P>,
    validators: Vec<V>,
}

impl<P: ConfigurationProvider, V: ConfigurationValidator> ConfigurationBuilder<P, V> {
    pub fn new() -> Self {
        Self {
            options: ConfigurationOptions::default(),
            providers: vec![],
            validators: vec![],
        }
    }

    pub fn with_options(mut self, options: ConfigurationOptions) -> Self {
        self.options = options;
        self
    }

    pub fn add_source(mut self, source: ConfigurationSource) -> Self {
        self.options.sources.push(source);
        self
    }

    pub fn add_provider(mut self, provider: P) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn add_validator(mut self, validator: V) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn validation_level(mut self, level: ValidationLevel) -> Self {
        self.options.validation_level = level;
        self
    }

    pub fn watch_changes(mut self, enabled: bool) -> Self {
        self.options.watch_for_changes = enabled;
        self
    }

    pub fn auto_reload(mut self, enabled: bool) -> Self {
        self.options.auto_reload = enabled;
        self
    }

    pub fn merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.options.merge_strategy = strategy;
        self
    }

    pub async fn build<NC: NetabaseConfiguration>(self) -> ConfigurationResult<NC> {
        // This would return a concrete implementation
        todo!("Implementation would create a concrete configuration instance")
    }
}

impl<P: ConfigurationProvider, V: ConfigurationValidator> Default for ConfigurationBuilder<P, V> {
    fn default() -> Self {
        Self::new()
    }
}
