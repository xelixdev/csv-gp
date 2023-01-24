use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnknownEncoding {
    #[error("unknown encoding {0}")]
    Encoding(String),
}

#[derive(Debug, Error)]
pub enum CSVError {
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    UnknownEncoding(#[from] UnknownEncoding),
}
