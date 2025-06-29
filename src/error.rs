use std::fmt;

/// Errors that can occur when working with the file system.
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemError {
    /// File was not found at the specified path.
    FileNotFound(String),
    /// The provided path is invalid (e.g., empty string).
    InvalidPath(String),
    /// An error occurred during a read operation.
    ReadError(String),
    /// An error occurred during a write operation.
    WriteError(String),
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::FileNotFound(path) => {
                write!(f, "File not found: {path}")
            }
            FileSystemError::InvalidPath(path) => {
                write!(f, "Invalid path: {path}")
            }
            FileSystemError::ReadError(msg) => {
                write!(f, "Read error: {msg}")
            }
            FileSystemError::WriteError(msg) => {
                write!(f, "Write error: {msg}")
            }
        }
    }
}

impl std::error::Error for FileSystemError {}
