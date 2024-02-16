extern crate std;

use std::{error, fmt, io, num::NonZeroU64, string::String};

#[derive(Debug)]
pub enum BinParse {
    Io(io::Error),
    Parse(String),
}

impl From<io::Error> for BinParse {
    fn from(err: io::Error) -> BinParse {
        BinParse::Io(err)
    }
}

impl fmt::Display for BinParse {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Io(e) => {
                fmt.write_fmt(format_args!("IO Error: {e}"))?;
            }
            Self::Parse(s) => {
                fmt.write_fmt(format_args!("Binary Parse Error: {s}"))?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for BinParse {}

#[derive(Debug, Clone)]
pub struct Line {
    pub message: String,
    pub line_number: Option<NonZeroU64>,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.line_number {
            Some(ln) => write!(f, "Line {}: {}", ln, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl error::Error for Line {}

#[derive(Debug)]
pub enum TextParse {
    Io(io::Error),
    Lexer(Line),
    Parser(Line),
}

impl TextParse {
    pub fn from_lexer(message: String, line_number: NonZeroU64) -> TextParse {
        TextParse::Lexer(Line {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn from_parser(message: String, line_number: NonZeroU64) -> TextParse {
        TextParse::Parser(Line {
            message,
            line_number: Some(line_number),
        })
    }

    pub fn eof() -> TextParse {
        TextParse::Parser(Line {
            message: String::from("Unexpected end-of-file"),
            line_number: None,
        })
    }
}

impl From<io::Error> for TextParse {
    fn from(err: io::Error) -> Self {
        TextParse::Io(err)
    }
}

impl fmt::Display for TextParse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "{}", msg),
            Self::Lexer(err) => write!(f, "{}", err),
            Self::Parser(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for TextParse {}

#[derive(Debug)]
pub enum Write {
    Validation(String),
    Io(std::io::Error),
}

impl From<io::Error> for Write {
    fn from(err: io::Error) -> Self {
        Write::Io(err)
    }
}

impl fmt::Display for Write {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "{}", msg),
            Self::Io(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Write {}
