#[cfg(feature = "std")]
extern crate std;

use std::{
    cell::RefCell,
    convert::TryInto,
    fmt, io,
    num::{NonZeroU64, NonZeroU8},
    string::String,
    vec::Vec,
};

use crate::qmap;
use qmap::{ParseError, ParseResult};

const TEXT_CAPACITY: usize = 32;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub text: Vec<NonZeroU8>,
    pub line_number: NonZeroU64,
}

impl Token {
    pub fn match_byte(&self, byte: u8) -> bool {
        self.text.len() == 1 && self.text[0].get() == byte
    }

    pub fn match_quoted(&self) -> bool {
        self.text.len() >= 2
            && self.text[0] == b'"'.try_into().unwrap()
            && self.text.last() == Some(&b'"'.try_into().unwrap())
    }

    pub fn text_as_string(&self) -> String {
        self.text
            .iter()
            .map::<char, _>(|ch| ch.get().into())
            .collect()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: line {}", self.text_as_string(), self.line_number)
    }
}

pub struct TokenIterator<R: io::Read> {
    text: RefCell<Option<Vec<NonZeroU8>>>,
    state: fn(iter: &mut TokenIterator<R>) -> Option<Token>,
    byte: Option<NonZeroU8>,
    last_byte: Option<NonZeroU8>,
    line_number: NonZeroU64,
    input: io::Bytes<R>,
}

impl<R: io::Read> TokenIterator<R> {
    pub fn new(reader: R) -> TokenIterator<R> {
        TokenIterator {
            text: RefCell::new(None),
            state: lex_default,
            byte: None,
            last_byte: None,
            line_number: NonZeroU64::new(1).unwrap(),
            input: reader.bytes(),
        }
    }

    fn byte_read(&mut self, b: io::Result<u8>) -> ParseResult<Option<Token>> {
        let byte = b.map_err(ParseError::from_io)?;

        self.byte = Some(byte.try_into().map_err(|_| {
            ParseError::from_lexer(String::from("Null byte"), self.line_number)
        })?);

        let maybe_token = (self.state)(self);

        if self.byte == NonZeroU8::new(b'\n')
            || self.last_byte == NonZeroU8::new(b'\r')
        {
            let next_line = self.line_number.get().saturating_add(1);
            unsafe {
                self.line_number = NonZeroU64::new_unchecked(next_line);
            }
        }

        self.last_byte = self.byte;

        Ok(maybe_token)
    }

    fn eof_read(&mut self) -> ParseResult<Option<Token>> {
        if let Some(last_text) = self.text.replace(None) {
            if last_text[0] == NonZeroU8::new(b'"').unwrap()
                && (last_text.last() != NonZeroU8::new(b'"').as_ref()
                    || last_text.len() == 1)
            {
                Err(ParseError::from_lexer(
                    String::from("Missing closing quote"),
                    self.line_number,
                ))
            } else {
                Ok(Some(Token {
                    text: last_text,
                    line_number: self.line_number,
                }))
            }
        } else {
            Ok(None)
        }
    }
}

impl<R: io::Read> Iterator for TokenIterator<R> {
    type Item = ParseResult<Token>;

    fn next(&mut self) -> Option<ParseResult<Token>> {
        loop {
            if let Some(b) = self.input.next() {
                if let token @ Some(_) = self.byte_read(b).transpose() {
                    break token;
                }
            } else {
                break self.eof_read().transpose();
            }
        }
    }
}

fn lex_default<R: io::Read>(iterator: &mut TokenIterator<R>) -> Option<Token> {
    if !iterator.byte.unwrap().get().is_ascii_whitespace() {
        if iterator.byte == NonZeroU8::new(b'"') {
            iterator.state = lex_quoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(iterator.byte.unwrap());
            *iterator.text.borrow_mut() = Some(text_bytes);
        } else if iterator.byte == NonZeroU8::new(b'/') {
            iterator.state = lex_maybe_comment;
        } else {
            iterator.state = lex_unquoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(iterator.byte.unwrap());
            *iterator.text.borrow_mut() = Some(text_bytes);
        }
    }

    None
}

fn lex_comment<R: io::Read>(iterator: &mut TokenIterator<R>) -> Option<Token> {
    if iterator.byte == NonZeroU8::new(b'\r')
        || iterator.byte == NonZeroU8::new(b'\n')
    {
        iterator.state = lex_default;
    }

    None
}

fn lex_maybe_comment<R: io::Read>(
    iterator: &mut TokenIterator<R>,
) -> Option<Token> {
    if iterator.byte == NonZeroU8::new(b'/') {
        iterator.state = lex_comment;
    } else {
        let mut text_bytes: Vec<NonZeroU8> = Vec::with_capacity(TEXT_CAPACITY);
        text_bytes.push(NonZeroU8::new(b'/').unwrap());
        text_bytes.push(iterator.byte.unwrap());
        *iterator.text.borrow_mut() = Some(text_bytes);
        iterator.state = lex_unquoted;
    }

    None
}

fn lex_quoted<R: io::Read>(iterator: &mut TokenIterator<R>) -> Option<Token> {
    iterator
        .text
        .borrow_mut()
        .as_mut()
        .unwrap()
        .push(iterator.byte.unwrap());
    if iterator.byte == NonZeroU8::new(b'"') {
        let local_text = iterator.text.replace(None).unwrap();
        iterator.state = lex_default;

        Some(Token {
            text: local_text,
            line_number: iterator.line_number,
        })
    } else {
        None
    }
}

fn lex_unquoted<R: io::Read>(iterator: &mut TokenIterator<R>) -> Option<Token> {
    if iterator.byte.unwrap().get().is_ascii_whitespace() {
        let local_text = iterator.text.replace(None).unwrap();
        iterator.state = lex_default;

        Some(Token {
            text: local_text,
            line_number: iterator.line_number,
        })
    } else {
        iterator
            .text
            .borrow_mut()
            .as_mut()
            .unwrap()
            .push(iterator.byte.unwrap());

        None
    }
}
