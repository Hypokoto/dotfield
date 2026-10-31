use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotfieldError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Profile validation failed: {0}")]
    Validation(String),

    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Process reload error: {0}")]
    Reload(String),
}

pub type Result<T> = std::result::Result<T, DotfieldError>;
