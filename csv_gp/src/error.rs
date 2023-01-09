use std::{error, fmt, io};

#[derive(Debug, Clone)]
pub struct UnknownEncoding {
    encoding: String,
}

impl UnknownEncoding {
    pub fn new(encoding: String) -> Self {
        Self { encoding }
    }
}

impl fmt::Display for UnknownEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown encoding {}", self.encoding)
    }
}

impl error::Error for UnknownEncoding {}

#[derive(Debug, PartialEq, Eq)]
pub enum DelimiterError {
    ZeroLengthDelimiter,
    MultiByteDelimiter,
}

impl fmt::Display for DelimiterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DelimiterError::ZeroLengthDelimiter => write!(f, "delimiter cannot be empty"),
            DelimiterError::MultiByteDelimiter => write!(f, "delimiter must be exactly one byte"),
        }
    }
}

impl error::Error for DelimiterError {}

#[derive(Debug)]
pub enum CSVError {
    UnknownEncoding(UnknownEncoding),
    IO(io::Error),
    DelimiterError(DelimiterError),
}

impl fmt::Display for CSVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CSVError::IO(e) => write!(f, "{}", e),
            CSVError::UnknownEncoding(e) => write!(f, "{}", e),
            CSVError::DelimiterError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for CSVError {}

impl From<io::Error> for CSVError {
    fn from(e: io::Error) -> Self {
        CSVError::IO(e)
    }
}

impl From<UnknownEncoding> for CSVError {
    fn from(e: UnknownEncoding) -> Self {
        CSVError::UnknownEncoding(e)
    }
}

impl From<DelimiterError> for CSVError {
    fn from(e: DelimiterError) -> Self {
        CSVError::DelimiterError(e)
    }
}
