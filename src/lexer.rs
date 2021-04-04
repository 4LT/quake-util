use std::collections::HashSet;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::io::Read;
use std::collections::VecDeque;

const TEXT_CAPACITY: usize = 32;

#[derive(Debug, Clone)]
pub struct Token {
    pub text: Vec<u8>,
    pub line_number: usize
}

impl Token {
    pub fn match_byte(&self, byte: u8) -> bool {
        self.text.len() == 1 && self.text[0] == byte
    }

    pub fn match_quoted(&self) -> bool {
        self.text.len() >= 2 &&
            self.text[0] == b'"' &&
            self.text.last() == Some(&b'"')
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = String::from_utf8_lossy(&self.text);
        write!(f, "{}: line {}", text, self.line_number)
    }
}

#[derive(Debug, Copy, Clone)]
enum LexerState {
    Default,
    Comment,
    MaybeComment,
    Unquoted,
    Quoted
}

pub fn lex<R: Read>(reader: R) -> VecDeque<Token> {
    let token_q = RefCell::new(VecDeque::new());
    let text: RefCell<Option<Vec<u8>>> = RefCell::new(None);
    let state = Cell::new(LexerState::Default);
    let byte = Cell::new(0u8);
    let last_byte: Cell<Option<u8>> = Cell::new(None);
    let line_number = Cell::new(1usize);

    let whitespace: HashSet<u8> = [b' ', b'\t', b'\n', b'\r']
        .iter()
        .cloned()
        .collect();

    let lex_default = || {
        let byte = byte.get();
        if !whitespace.contains(&byte) {
            if byte == b'"' {
                state.set(LexerState::Quoted);
                let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
                text_bytes.push(byte);
                *text.borrow_mut() = Some(text_bytes);
            } else if byte == b'/' {
                state.set(LexerState::MaybeComment);
            } else {
                state.set(LexerState::Unquoted);
                let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
                text_bytes.push(byte);
                *text.borrow_mut() = Some(text_bytes);
            }
        }
    };

    let lex_comment = || {
        let byte = byte.get();
        if byte == b'\r' || byte == b'\n' {
            state.set(LexerState::Default);
        }
    };

    let lex_maybe_comment = || {
        let byte = byte.get();
        if byte == b'/' {
            state.set(LexerState::Comment);
        } else {
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(b'/');
            text_bytes.push(byte);
            *text.borrow_mut() = Some(text_bytes);
            state.set(LexerState::Unquoted);
        }
    };

    let lex_quoted = || {
        let byte = byte.get();
        text.borrow_mut().as_mut().unwrap().push(byte);
        if byte == b'"' {
            let local_text = text.replace(None).unwrap();
            token_q
                .borrow_mut()
                .push_back(Token {
                    text: local_text,
                    line_number: line_number.get()
                });
            state.set(LexerState::Default);
        } 
    };

    let lex_unquoted = || {
        let byte = byte.get();
        if whitespace.contains(&byte) {
            let local_text = text.replace(None).unwrap();
            token_q
                .borrow_mut()
                .push_back(Token {
                    text: local_text,
                    line_number: line_number.get()
                });
            state.set(LexerState::Default)
        } else {
            text.borrow_mut().as_mut().unwrap().push(byte);
        }
    };

    for b in reader.bytes() {
        byte.set(b.unwrap());

        match state.get() {
            LexerState::Default => lex_default(),
            LexerState::Comment => lex_comment(),
            LexerState::MaybeComment => lex_maybe_comment(),
            LexerState::Unquoted => lex_unquoted(),
            LexerState::Quoted => lex_quoted()
        }

        if byte.get() == b'\n' || last_byte.get() == Some(b'\r') {
            line_number.set(line_number.get() + 1);
        }

        last_byte.set(Some(byte.get()));
    }

    if let Some(last_text) = text.replace(None) {
        token_q
            .borrow_mut()
            .push_back(Token {
                text: last_text,
                line_number: line_number.get()
            });
    }

    token_q.into_inner()
}
