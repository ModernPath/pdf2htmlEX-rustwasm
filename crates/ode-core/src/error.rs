use thiserror::Error;

pub type OdeResult<T> = Result<T, OdeError>;

#[derive(Error, Debug)]
pub enum OdeError {
    #[error("PDF parsing error: {0}")]
    PdfParseError(String),

    #[error("Font processing error: {0}")]
    FontError(String),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Text extraction error: {0}")]
    TextError(String),

    #[error("Compressed data rejected (zip bomb detected): ratio {ratio}:1 exceeds limit")]
    ZipBomb { ratio: u32 },

    #[error("Timeout exceeded: {0}")]
    Timeout(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}
