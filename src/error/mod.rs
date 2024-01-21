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
