use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConduitError>;

#[derive(Debug, Error)]
pub enum ConduitError {
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("invalid packet: {0}")]
    InvalidPacket(String),

    #[error("checksum mismatch: expected {expected:#010x}, got {actual:#010x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("buffer overflow: need {needed} bytes, have {available}")]
    BufferOverflow { needed: usize, available: usize },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
