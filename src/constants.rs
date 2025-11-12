/// API constants for consistent endpoint and tag naming
/// 
/// This module centralizes all API-related constants to ensure consistency
/// across the codebase and make it easier to maintain API structure.

// API prefix for all authenticated endpoints
pub const API_PREFIX: &str = "/api";

// API tags for OpenAPI documentation
pub mod tags {
    pub const AUTH: &str = "auth";
    pub const USERS: &str = "users";
    pub const CUSTOMERS: &str = "customers";
}

// API paths
pub mod paths {
    pub const USERS: &str = "/users";
    pub const CUSTOMERS: &str = "/customers";
}

// API context paths for OpenAPI documentation
pub mod context_paths {
    use super::API_PREFIX;
    
    pub fn users() -> String {
        format!("{}{}", API_PREFIX, super::paths::USERS)
    }
    
    pub fn customers() -> String {
        format!("{}{}", API_PREFIX, super::paths::CUSTOMERS)
    }
}
