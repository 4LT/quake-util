#[cfg(feature = "std")]
extern crate std;

use std::{
    fmt,
    num::NonZeroU64,
    string::{String, ToString},
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
pub enum ParseError {
    Io(String),
    Lexer(LineError),
    Parser(LineError),
}

impl ParseError {
    pub fn from_lexer(message: String, line_number: NonZeroU64) -> ParseError {
        ParseError::Lexer(LineError {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn from_parser(message: String, line_number: NonZeroU64) -> ParseError {
        ParseError::Parser(LineError {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn from_io(io_error: std::io::Error) -> ParseError {
        ParseError::Io(io_error.to_string())
    }

    pub fn eof() -> ParseError {
        ParseError::Parser(LineError {
            message: String::from("Unexpected end-of-file"),
            line_number: None,
        })
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            ParseError::Io(msg) => write!(f, "{}", msg),
            ParseError::Lexer(err) => write!(f, "{}", err),
            ParseError::Parser(err) => write!(f, "{}", err),
        }
    }
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
