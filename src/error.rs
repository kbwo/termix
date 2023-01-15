#[derive(thiserror::Error, Debug)]
pub enum TermixError {
    #[error("Failed to write to stdout")]
    Write(String),
    #[error("Failed to detect cursor position")]
    CursorDetection,
    #[error("Failed to listen keys")]
    KeyListener,
    #[error("Unexpected byte")]
    KeyRead(u8),
    #[error("Something happend")]
    Any(#[from] anyhow::Error),
}
