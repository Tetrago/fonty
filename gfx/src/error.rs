#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("fonty error: {0}")]
    Fonty(#[from] fonty::Error),
    #[error("unsupported feature")]
    UnsupportedFeature,
    #[error("failed to compile shader: {0}")]
    ShaderCompilationFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;
