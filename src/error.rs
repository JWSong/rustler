use std::io;
use std::net::AddrParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] io::Error),

    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] AddrParseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("State machine error: {0}")]
    StateMachine(String),

    #[error("Unknown error occurred")]
    Unknown,
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        tracing::error!("Bincode error: {}", e);
        Error::Network(io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
}
