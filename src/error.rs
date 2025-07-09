use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeatzyError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid mode: {0}")]
    InvalidMode(String),
    
    #[error("No authentication token set")]
    NoToken,
    
    #[error("API error: {0}")]
    Api(String),
}