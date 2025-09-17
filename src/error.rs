use thiserror::Error;

#[derive(Error, Debug)]
pub enum SniperError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Solana client error: {0}")]
    SolanaClient(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Price fetch error: {0}")]
    PriceFetch(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Base58 decode error: {0}")]
    Base58(#[from] bs58::decode::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SniperError>;
