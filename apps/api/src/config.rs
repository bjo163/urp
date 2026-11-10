use std::env;

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub database_url: String,
    pub database_max_connections: u32,
    pub port: u16,
    pub log_filter: String,
    pub password_memory_kib: u32,
    pub password_iterations: u32,
    pub password_parallelism: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable DATABASE_URL is missing or empty")]
    MissingDatabaseUrl,
    #[error("environment variable {0} must be a valid positive integer")]
    InvalidInteger(&'static str),
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or(ConfigError::MissingDatabaseUrl)?;

        let port = parse_positive_u16("API_PORT", 8080)?;
        let database_max_connections = parse_positive_u32("DATABASE_MAX_CONNECTIONS", 10)?;
        let password_memory_kib = parse_positive_u32("PASSWORD_ARGON2_MEMORY_KIB", 19 * 1024)?;
        let password_iterations = parse_positive_u32("PASSWORD_ARGON2_ITERATIONS", 2)?;
        let password_parallelism = parse_positive_u32("PASSWORD_ARGON2_PARALLELISM", 1)?;

        let log_filter =
            env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn,hyper=warn".to_owned());

        Ok(Self {
            database_url,
            database_max_connections,
            port,
            log_filter,
            password_memory_kib,
            password_iterations,
            password_parallelism,
        })
    }
}

fn parse_positive_u16(name: &'static str, default: u16) -> Result<u16, ConfigError> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u16>()
            .ok()
            .filter(|parsed| *parsed > 0)
            .ok_or(ConfigError::InvalidInteger(name)),
        Err(_) => Ok(default),
    }
}

fn parse_positive_u32(name: &'static str, default: u32) -> Result<u32, ConfigError> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u32>()
            .ok()
            .filter(|parsed| *parsed > 0)
            .ok_or(ConfigError::InvalidInteger(name)),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, parse_positive_u16, parse_positive_u32};

    #[test]
    fn default_port_is_used_when_unset() {
        assert!(parse_positive_u16("URP_TEST_MISSING_PORT", 8080).is_ok_and(|port| port == 8080));
    }

    #[test]
    fn default_pool_size_is_used_when_unset() {
        assert!(parse_positive_u32("URP_TEST_MISSING_POOL_SIZE", 10).is_ok_and(|size| size == 10));
    }

    #[test]
    fn zero_port_is_rejected() {
        let parsed = "0"
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or(ConfigError::InvalidInteger("API_PORT"));
        assert!(matches!(
            parsed,
            Err(ConfigError::InvalidInteger("API_PORT"))
        ));
    }
}
