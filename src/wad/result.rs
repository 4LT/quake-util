#[derive(Debug, Clone)]
pub enum ReadError {
    Io(std::io::Error),
}

pub type ReadResult<T> = core::result::Result<T, ReadError>;
