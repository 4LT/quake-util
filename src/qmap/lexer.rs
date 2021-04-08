use std::cell::RefCell;
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

pub struct TokenError {
    pub message: String,
    pub line_number: usize
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.line_number, self.message)
    }
}

pub enum LexerError {
    Token(TokenError),
    Io(std::io::Error)
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexerError::Token(e) => e.fmt(f),
            LexerError::Io(e) => e.fmt(f)
        }
    }
}

pub type Result = std::result::Result<VecDeque<Token>, LexerError>;

struct LexerContext {
    token_q: VecDeque<Token>,
    text: RefCell<Option<Vec<u8>>>,
    state: fn(ctx: &mut LexerContext),
    byte: u8,
    last_byte: Option<u8>,
    line_number: usize,
}

impl LexerContext {
    fn new() -> LexerContext {
        LexerContext {
            token_q: VecDeque::new(),
            text: RefCell::new(None),
            state: lex_default,
            byte: 0,
            last_byte: None,
            line_number: 1,
        }
    }
}

pub fn lex<R: Read>(reader: R) -> Result {
    let mut ctx = LexerContext::new();

    for b in reader.bytes() {
        ctx.byte = b.map_err(LexerError::Io)?;
        (ctx.state)(&mut ctx);

        if ctx.byte == b'\n' || ctx.last_byte == Some(b'\r') {
            ctx.line_number+= 1;
        }

        ctx.last_byte = Some(ctx.byte);
    }

    if let Some(last_text) = ctx.text.replace(None) {
        if last_text[0] == b'"' && last_text.last() != Some(&b'"') {
            return Err(LexerError::Token(TokenError {
                message: String::from("Missing closing quote"),
                line_number: ctx.line_number
            }));
        }

        ctx.token_q.push_back(Token {
            text: last_text,
            line_number: ctx.line_number
        });
    }

    Ok(ctx.token_q)
}

fn lex_default(ctx: &mut LexerContext) {
    if !ctx.byte.is_ascii_whitespace() {
        if ctx.byte == b'"' {
            ctx.state = lex_quoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(ctx.byte);
            *ctx.text.borrow_mut() = Some(text_bytes);
        } else if ctx.byte == b'/' {
            ctx.state = lex_maybe_comment;
        } else {
            ctx.state = lex_unquoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(ctx.byte);
            *ctx.text.borrow_mut() = Some(text_bytes);
        }
    }
}

fn lex_comment(ctx: &mut LexerContext) {
    if ctx.byte == b'\r' || ctx.byte == b'\n' {
        ctx.state = lex_default;
    }
}

fn lex_maybe_comment(ctx: &mut LexerContext) {
    if ctx.byte == b'/' {
        ctx.state = lex_comment;
    } else {
        let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
        text_bytes.push(b'/');
        text_bytes.push(ctx.byte);
        *ctx.text.borrow_mut() = Some(text_bytes);
        ctx.state = lex_unquoted;
    }
}

fn lex_quoted(ctx: &mut LexerContext) {
    ctx.text.borrow_mut().as_mut().unwrap().push(ctx.byte);
    if ctx.byte == b'"' {
        let local_text = ctx.text.replace(None).unwrap();
        ctx.token_q.push_back(Token {
            text: local_text,
            line_number: ctx.line_number
        });
        ctx.state = lex_default;
    } 
}

fn lex_unquoted(ctx: &mut LexerContext) {
    if ctx.byte.is_ascii_whitespace() {
        let local_text = ctx.text.replace(None).unwrap();
        ctx.token_q.push_back(Token {
            text: local_text,
            line_number: ctx.line_number
        });
        ctx.state = lex_default;
    } else {
        ctx.text.borrow_mut().as_mut().unwrap().push(ctx.byte);
    }
}
