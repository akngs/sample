use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidSampleSize,
    InvalidSeedValue,
    InvalidPercentage,
    HashRequiresCsvMode,
    HashRequiresPercentage,
    ColumnNotFound(String),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
