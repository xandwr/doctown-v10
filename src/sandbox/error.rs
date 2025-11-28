use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to download repository: {0}")]
    DownloadFailed(String),

    #[error("Failed to parse ZIP archive: {0}")]
    ZipParseFailed(String),

    #[error("Invalid path in archive: {0}")]
    InvalidPath(String),

    #[error("File too large: {size} bytes (max: {max})")]
    FileTooLarge { size: u64, max: u64 },
}
