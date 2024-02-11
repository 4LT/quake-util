use crate::{common, error, qmap};
use common::CellOptionExt;
use qmap::lexer::{Token, TokenIterator};
use std::io;
use std::num::NonZeroU8;
use std::string::ToString;
use std::vec::Vec;

fn bytes_to_token_text(bytes: &[u8]) -> Vec<NonZeroU8> {
    bytes.iter().map(|&b: &u8| b.try_into().unwrap()).collect()
}

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

// Successes

#[test]
fn lex_all_symbols() {
    let input = b" // a comment  \n { } x \"\"\t\" \"\n-1.23e4 {k \r\n q\"";
    let iter = TokenIterator::new(&input[..]);

    let expected = [
        (&b"{"[..], 2u64),
        (&b"}"[..], 2u64),
        (&b"x"[..], 2u64),
        (&b"\"\""[..], 2u64),
        (&b"\" \""[..], 2u64),
        (&b"-1.23e4"[..], 3u64),
        (&b"{k"[..], 3u64),
        (&b"q\""[..], 4u64),
    ];

    let expected_iter = expected.iter().map(|&(text, line_number)| Token {
        text: bytes_to_token_text(text),
        line_number: line_number.try_into().unwrap(),
    });

    iter.zip(expected_iter).for_each(|(actual, expected)| {
        assert_eq!(actual.map_err(|_| ()).unwrap(), expected);
    });
}

// Failures

#[test]
fn lex_bad_quoted() {
    let input = b"good-token \"bad-token eof-here-we-come";
    let bad_token = TokenIterator::new(&input[..]).nth(1);

    if let Err(qmap_error) = bad_token.unwrap() {
        if let error::TextParse::Lexer(line_error) = qmap_error.steal() {
            assert!(line_error.message.contains("closing quote"));
            assert_eq!(u64::from(line_error.line_number.unwrap()), 1u64);
        } else {
            panic!("Unexpected error type");
        }
    } else {
        panic!("Expected error");
    }
}

#[test]
fn lex_io_error() {
    let reader = ErroringReader::new();
    let bad_token = TokenIterator::new(reader).next();

    if let Err(qmap_error) = bad_token.unwrap() {
        if let error::TextParse::Io(io_error) = qmap_error.steal() {
            assert!(io_error.to_string().contains("Generic test error"));
        } else {
            panic!("Unexpected error type");
        }
    } else {
        panic!("Expected error");
    }
}
