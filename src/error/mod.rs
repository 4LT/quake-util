use std::io;
use std::string::String;

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

impl std::fmt::Display for BinParse {
    fn fmt(
        &self,
        fmt: &mut std::fmt::Formatter,
    ) -> Result<(), std::fmt::Error> {
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

pub type BinParseResult<T> = Result<T, BinParse>;
