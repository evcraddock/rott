//! Storage error handling
//!
//! Provides typed errors for storage operations with descriptive messages
//! and recovery suggestions.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    /// Failed to create data directory
    #[error("Failed to create data directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Permission denied accessing path
    #[error("Permission denied: cannot access '{path}'. Check file permissions.")]
    PermissionDenied {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Disk is full or quota exceeded
    #[error(
        "Disk full or quota exceeded while writing to '{path}'. Free up disk space and try again."
    )]
    DiskFull {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Failed to read file
    #[error("Failed to read '{path}': {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Failed to write file
    #[error("Failed to write '{path}': {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Document is corrupted
    #[error("Document at '{path}' is corrupted: {details}. A backup has been created at '{backup_path}'.")]
    CorruptDocument {
        path: PathBuf,
        backup_path: PathBuf,
        details: String,
    },

    /// Document format is invalid (cannot be parsed)
    #[error("Invalid document format in '{path}': {details}")]
    InvalidFormat { path: PathBuf, details: String },

    /// SQLite database error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Automerge error
    #[error("Automerge error: {0}")]
    Automerge(String),

    /// File not found (when expected to exist)
    #[error("File not found: '{path}'")]
    NotFound { path: PathBuf },

    /// Atomic write failed during rename
    #[error("Atomic write failed: could not rename '{from}' to '{to}': {source}")]
    AtomicWriteFailed {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

impl StorageError {
    /// Create an error from an I/O error with path context
    ///
    /// Classifies the error based on its kind (permission, disk full, etc.)
    pub fn from_io(error: io::Error, path: PathBuf) -> Self {
        match error.kind() {
            io::ErrorKind::PermissionDenied => StorageError::PermissionDenied {
                path,
                source: error,
            },
            io::ErrorKind::NotFound => StorageError::NotFound { path },
            // StorageFull is available but may not be on all platforms
            // Also check for "No space left" in the error message
            _ if is_disk_full_error(&error) => StorageError::DiskFull {
                path,
                source: error,
            },
            _ => StorageError::WriteError {
                path,
                source: error,
            },
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            StorageError::DiskFull { .. }
                | StorageError::PermissionDenied { .. }
                | StorageError::CorruptDocument { .. }
        )
    }

    /// Get a recovery suggestion for this error
    pub fn recovery_suggestion(&self) -> Option<&'static str> {
        match self {
            StorageError::DiskFull { .. } => {
                Some("Free up disk space and try again.")
            }
            StorageError::PermissionDenied { .. } => {
                Some("Check file and directory permissions. You may need to run with different permissions or change ownership.")
            }
            StorageError::CorruptDocument { .. } => {
                Some("A backup of the corrupted file has been created. You can try to recover data from it manually, or start fresh.")
            }
            StorageError::CreateDirectory { .. } => {
                Some("Check that the parent directory exists and you have write permissions.")
            }
            _ => None,
        }
    }
}

/// Check if an I/O error indicates disk full condition
fn is_disk_full_error(error: &io::Error) -> bool {
    // Check error message for disk full indicators
    let msg = error.to_string().to_lowercase();
    msg.contains("no space left")
        || msg.contains("disk full")
        || msg.contains("quota exceeded")
        || msg.contains("not enough space")
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_denied_classification() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = StorageError::from_io(io_err, PathBuf::from("/test/path"));

        assert!(matches!(err, StorageError::PermissionDenied { .. }));
        assert!(err.is_recoverable());
        assert!(err.recovery_suggestion().is_some());
    }

    #[test]
    fn test_not_found_classification() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = StorageError::from_io(io_err, PathBuf::from("/missing/file"));

        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn test_disk_full_detection() {
        let io_err = io::Error::new(io::ErrorKind::Other, "No space left on device");
        let err = StorageError::from_io(io_err, PathBuf::from("/full/disk"));

        assert!(matches!(err, StorageError::DiskFull { .. }));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = StorageError::PermissionDenied {
            path: PathBuf::from("/test/file"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };

        let msg = err.to_string();
        assert!(msg.contains("Permission denied"));
        assert!(msg.contains("/test/file"));
    }

    #[test]
    fn test_corrupt_document_display() {
        let err = StorageError::CorruptDocument {
            path: PathBuf::from("/data/doc.automerge"),
            backup_path: PathBuf::from("/data/doc.automerge.corrupt.backup"),
            details: "invalid header".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("corrupted"));
        assert!(msg.contains("backup"));
    }
}
