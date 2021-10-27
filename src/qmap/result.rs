#[cfg(feature = "std")]
extern crate std;

use std::{
    fmt,
    num::NonZeroU64,
    string::{ String, ToString }
};

#[derive(Debug, Clone)]
pub struct LineError {
    pub message: String,
    pub line_number: Option<NonZeroU64>,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.line_number {
            Some(ln) => write!(f, "Line {}: {}", ln, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Io(String),
    Lexer(LineError),
    Parser(LineError),
}

impl Error {
    pub fn from_lexer(message: String, line_number: NonZeroU64) -> Error {
        Error::Lexer(LineError {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn from_parser(message: String, line_number: NonZeroU64) -> Error {
        Error::Parser(LineError {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn from_io(io_error: std::io::Error) -> Error {
        Error::Io(io_error.to_string())
    }

    pub fn eof() -> Error {
        Error::Parser(LineError {
            message: String::from("Unexpected EOF"),
            line_number: None,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            Error::Io(msg) => write!(f, "{}", msg),
            Error::Lexer(err) => write!(f, "{}", err),
            Error::Parser(err) => write!(f, "{}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
