use thiserror::Error;

/// Domain and application errors for archive operations.
#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("unsupported or unknown archive format")]
    UnsupportedFormat,

    #[error("could not determine archive format; specify --format or use a known extension")]
    FormatUndetermined,

    #[error("input path does not exist: {0}")]
    InputNotFound(String),

    #[error("no input paths provided")]
    EmptyInput,

    #[error("archive path does not exist: {0}")]
    ArchiveNotFound(String),

    #[error("path traversal rejected for member: {0}")]
    PathTraversal(String),

    #[error("invalid member path: {0}")]
    InvalidMemberPath(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("archive engine error: {0}")]
    Engine(String),

    #[error("invalid media kind: {0}")]
    InvalidMedia(String),

    #[error("invalid connection kind: {0}")]
    InvalidConnection(String),

    #[error("invalid size: {0}")]
    InvalidSize(String),

    #[error("mapping error: {0}")]
    Mapping(String),

    #[error("budget required: use --target-size or --media (budget must be > 0)")]
    InvalidBudget,

    #[error("source is not a directory: {0}")]
    SourceNotDirectory(String),

    #[error("content hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("throughput measurement failed: {0}")]
    MeasureFailed(String),

    #[error("interrupted (Ctrl+C); progress saved where possible — re-run with --resume")]
    Interrupted,

    #[error("insufficient resources: {0}")]
    InsufficientResources(String),

    #[error("verify failed: {0}")]
    VerifyFailed(String),

    /// Exclusive create failed because the path already exists (lock conflict).
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Another pack holds the output or work-dir lock.
    #[error("pack busy: {0}")]
    PackBusy(String),
}
