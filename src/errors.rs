use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Represents all possible errors that can occur in the dbug application
#[derive(Error, Debug)]
pub enum DbugError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Invalid project path: {0}")]
    InvalidProjectPath(PathBuf),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Failed to run target: {0}")]
    RuntimeError(String),

    #[error("Instrumentation error: {0}")]
    InstrumentationError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Timeout waiting for response")]
    ResponseTimeout,

    #[error("Failed to parse source file: {0}")]
    SourceParseError(String),

    #[error("Debug point error: {0}")]
    DebugPointError(String),

    #[error("Variable inspection error: {0}")]
    VariableInspectionError(String),

    #[error("CLI error: {0}")]
    CliError(String),

    #[error("Terminal UI error: {0}")]
    TuiError(String),

    #[error("Not a valid Rust project (no Cargo.toml found)")]
    NotARustProject,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for dbug operations
pub type DbugResult<T> = Result<T, DbugError>;

/// Extension trait for custom error handling methods
pub trait ErrorExt<T> {
    /// Adds context to an error message
    fn with_context<C: AsRef<str>>(self, context: C) -> DbugResult<T>;
}

impl<T, E: std::error::Error + 'static> ErrorExt<T> for Result<T, E> {
    fn with_context<C: AsRef<str>>(self, context: C) -> DbugResult<T> {
        self.map_err(|e| DbugError::Unknown(format!("{}: {}", context.as_ref(), e)))
    }
}

/// Utility function to convert any error to a DbugError with a custom message
pub fn to_dbug_error<E: std::error::Error>(err: E, message: &str) -> DbugError {
    DbugError::Unknown(format!("{}: {}", message, err))
}
