#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("fonty error: {0}")]
    Fonty(#[from] fonty::Error),
    #[error("failed to compile shader: {0}")]
    ShaderCompilationFailed(String),
    #[error("attempted to nest components relatively")]
    UnsupportedNestedComponent,
}

pub type Result<T> = std::result::Result<T, Error>;
