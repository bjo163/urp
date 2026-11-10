use std::{env, error::Error, fmt, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_address: SocketAddr,
    pub log_filter: String,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_address = env::var("URP_API_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
        let log_filter = env::var("URP_LOG").unwrap_or_else(|_| "info".to_owned());

        Self::parse(&bind_address, &log_filter)
    }

    pub fn parse(bind_address: &str, log_filter: &str) -> Result<Self, ConfigError> {
        let bind_address = bind_address
            .parse()
            .map_err(|_| ConfigError::InvalidBindAddress(bind_address.to_owned()))?;

        if log_filter.trim().is_empty() {
            return Err(ConfigError::EmptyLogFilter);
        }

        Ok(Self {
            bind_address,
            log_filter: log_filter.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidBindAddress(String),
    EmptyLogFilter,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBindAddress(value) => {
                write!(
                    formatter,
                    "URP_API_BIND is not a valid socket address: {value}"
                )
            }
            Self::EmptyLogFilter => write!(formatter, "URP_LOG must not be empty"),
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_configuration() {
        let config = ApiConfig::parse("127.0.0.1:8080", "info").unwrap();

        assert_eq!(config.bind_address, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(config.log_filter, "info");
    }

    #[test]
    fn rejects_invalid_bind_address() {
        assert!(matches!(
            ApiConfig::parse("not-an-address", "info"),
            Err(ConfigError::InvalidBindAddress(_))
        ));
    }

    #[test]
    fn rejects_empty_log_filter() {
        assert_eq!(
            ApiConfig::parse("127.0.0.1:8080", " "),
            Err(ConfigError::EmptyLogFilter)
        );
    }
}
