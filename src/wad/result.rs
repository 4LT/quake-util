use std::io;
use std::string::String;

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Parse(String),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

pub type ReadResult<T> = core::result::Result<T, ReadError>;
