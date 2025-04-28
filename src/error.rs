use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error encounted in reader: {0}")]
    Reader(#[from] io::Error),
    #[error("unsupported cmap format: 0x{0:02x}")]
    UnsupportedCmapFormat(u16),
    #[error("could not find suitable cmap subtable")]
    NoSuitableCmapFormat,
    #[error("could not find character `{0}` in cmap table")]
    CodepointNotInCmap(char),
    #[error("invalid head field: {0}")]
    InvalidHeaderField(String),
    #[error("invalid subtable tag: {0:?}")]
    InvalidSubtable([u8; 4]),
}

pub type Result<T> = std::result::Result<T, Error>;
