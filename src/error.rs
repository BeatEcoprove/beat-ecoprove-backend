use core::fmt;

#[derive(Debug)]
pub enum ProxyError {
    ConfigError(String),
    RouteNotFound(String),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProxyError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            ProxyError::RouteNotFound(path) => write!(f, "Route not found: {}", path),
        }
    }
}

impl std::error::Error for ProxyError {}
