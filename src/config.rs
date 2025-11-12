use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database_url: String,
    pub test_database_url: String,
    pub jwt_secret: String,
    pub ldap_uri: String,
    pub ldap_filter: String,
    pub ldap_uid_column: String,
    pub ldap_user_dn: String,
    pub client_host: Option<String>,
    
    // OpenTelemetry configuration
    #[serde(default)]
    pub otel_enabled: Option<bool>,
    #[serde(default)]
    pub otel_endpoint: Option<String>,
    #[serde(default)]
    pub otel_service_name: Option<String>,
    #[serde(default)]
    pub otel_service_version: Option<String>,
}

pub fn get_config() -> Result<Config, String> {
    dotenv().ok();
    
    let config = envy::from_env::<Config>()
        .map_err(|e| format!("Failed to load configuration from environment: {}", e))?;
    
    // Validate OpenTelemetry configuration
    config.validate_otel_config()?;
    
    Ok(config)
}

impl Config {
    /// Returns whether OpenTelemetry is enabled
    pub fn is_otel_enabled(&self) -> bool {
        self.otel_enabled.unwrap_or(false)
    }
    
    /// Returns the OpenTelemetry endpoint with default value
    pub fn get_otel_endpoint(&self) -> String {
        self.otel_endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:4317".to_string())
    }
    
    /// Returns the service name with default value
    pub fn get_otel_service_name(&self) -> String {
        self.otel_service_name
            .clone()
            .unwrap_or_else(|| "rust-api".to_string())
    }
    
    /// Returns the service version with default value
    pub fn get_otel_service_version(&self) -> String {
        self.otel_service_version
            .clone()
            .unwrap_or_else(|| "0.1.0".to_string())
    }
    
    /// Validates OpenTelemetry configuration
    /// Returns an error message if the configuration is invalid
    pub fn validate_otel_config(&self) -> Result<(), String> {
        if !self.is_otel_enabled() {
            return Ok(());
        }
        
        // Validate endpoint format
        if let Some(endpoint) = &self.otel_endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(format!(
                    "Invalid OTEL_ENDPOINT: '{}'. Must start with 'http://' or 'https://'",
                    endpoint
                ));
            }
        }
        
        // Validate service name is not empty
        if let Some(name) = &self.otel_service_name {
            if name.trim().is_empty() {
                return Err(
                    "Invalid OTEL_SERVICE_NAME: Service name cannot be empty".to_string()
                );
            }
        }
        
        // Validate service version is not empty
        if let Some(version) = &self.otel_service_version {
            if version.trim().is_empty() {
                return Err(
                    "Invalid OTEL_SERVICE_VERSION: Service version cannot be empty".to_string()
                );
            }
        }
        
        Ok(())
    }
}
