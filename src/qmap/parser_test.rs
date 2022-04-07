use crate::qmap;
use qmap::parser::parse;
use std::io;

struct ErroringReader {}

impl ErroringReader {
    fn new() -> Self {
        Self {}
    }
}

impl io::Read for ErroringReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "Generic test error"))
    }
}

// Parse successes

#[test]
fn parse_empty_map() {
    let ast = parse(&b""[..]).unwrap();
    assert_eq!(ast.entities.len(), 0);
}

// Parse errors

#[test]
fn parse_token_error() {
    let err = parse(&b"\""[..]).err().unwrap();
    if let qmap::result::Error::Lexer(lex_err) = err {
        assert_eq!(u64::from(lex_err.line_number.unwrap()), 1u64);
    } else {
        panic!("Unexpected error variant for {}", err);
    }
}

#[test]
fn parse_io_error() {
    let reader = ErroringReader::new();
    let err = parse(reader).err().unwrap();
    if let qmap::result::Error::Io(_) = err {
    } else {
        panic!("Unexpected error variant for {}", err);
    }
}
