use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolscopeError {
    #[error("API request failed: {0}")]
    Api(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("RPC error: {code} - {message}")]
    Rpc { code: i64, message: String },
}
