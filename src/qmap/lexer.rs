use std::cell::RefCell;
use std::fmt;
use std::num::NonZeroU8;
use std::io::Read;
use std::collections::VecDeque;
use std::convert::TryInto;

const TEXT_CAPACITY: usize = 32;

#[derive(Debug, Clone)]
pub struct Token {
    pub text: Vec<NonZeroU8>,
    pub line_number: usize
}

impl Token {
    pub fn match_byte(&self, byte: u8) -> bool {
        self.text.len() == 1 && self.text[0].get() == byte
    }

    pub fn match_quoted(&self) -> bool {
        self.text.len() >= 2 &&
            self.text[0] == b'"'.try_into().unwrap() &&
            self.text.last() == Some(&b'"'.try_into().unwrap())
    }

    pub fn text_as_string(&self) -> String {
        self.text.iter()
            .map::<char, _>(|ch| ch.get().into())
            .collect()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: line {}", self.text_as_string(), self.line_number)
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
    text: RefCell<Option<Vec<NonZeroU8>>>,
    state: fn(ctx: &mut LexerContext),
    byte: Option<NonZeroU8>,
    last_byte: Option<NonZeroU8>,
    line_number: usize,
}

impl LexerContext {
    fn new() -> LexerContext {
        LexerContext {
            token_q: VecDeque::new(),
            text: RefCell::new(None),
            state: lex_default,
            byte: None,
            last_byte: None,
            line_number: 1,
        }
    }
}

pub fn lex<R: Read>(reader: R) -> Result {
    let mut ctx = LexerContext::new();

    for b in reader.bytes() {
        let byte = b.map_err(LexerError::Io)?;
        ctx.byte = Some(byte.try_into().map_err(
                |_| LexerError::Token(TokenError{
                    message: String::from("Null byte"),
                    line_number: ctx.line_number
                }))?);
        (ctx.state)(&mut ctx);

        if ctx.byte == NonZeroU8::new(b'\n')
            || ctx.last_byte == NonZeroU8::new(b'\r')
        {
            ctx.line_number+= 1;
        }

        ctx.last_byte = ctx.byte;
    }

    if let Some(last_text) = ctx.text.replace(None) {
        if last_text[0] == NonZeroU8::new(b'"').unwrap()
            && last_text.last() != NonZeroU8::new(b'"').as_ref()
        {
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
    if !ctx.byte.unwrap().get().is_ascii_whitespace() {
        if ctx.byte == NonZeroU8::new(b'"') {
            ctx.state = lex_quoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(ctx.byte.unwrap());
            *ctx.text.borrow_mut() = Some(text_bytes);
        } else if ctx.byte == NonZeroU8::new(b'/') {
            ctx.state = lex_maybe_comment;
        } else {
            ctx.state = lex_unquoted;
            let mut text_bytes = Vec::with_capacity(TEXT_CAPACITY);
            text_bytes.push(ctx.byte.unwrap());
            *ctx.text.borrow_mut() = Some(text_bytes);
        }
    }
}

fn lex_comment(ctx: &mut LexerContext) {
    if ctx.byte == NonZeroU8::new(b'\r') || ctx.byte == NonZeroU8::new(b'\n') {
        ctx.state = lex_default;
    }
}

fn lex_maybe_comment(ctx: &mut LexerContext) {
    if ctx.byte == NonZeroU8::new(b'/') {
        ctx.state = lex_comment;
    } else {
        let mut text_bytes: Vec<NonZeroU8> = Vec::with_capacity(TEXT_CAPACITY);
        text_bytes.push(NonZeroU8::new(b'/').unwrap());
        text_bytes.push(ctx.byte.unwrap());
        *ctx.text.borrow_mut() = Some(text_bytes);
        ctx.state = lex_unquoted;
    }
}

fn lex_quoted(ctx: &mut LexerContext) {
    ctx.text.borrow_mut().as_mut().unwrap().push(ctx.byte.unwrap());
    if ctx.byte == NonZeroU8::new(b'"') {
        let local_text = ctx.text.replace(None).unwrap();
        ctx.token_q.push_back(Token {
            text: local_text,
            line_number: ctx.line_number
        });
        ctx.state = lex_default;
    } 
}

fn lex_unquoted(ctx: &mut LexerContext) {
    if ctx.byte.unwrap().get().is_ascii_whitespace() {
        let local_text = ctx.text.replace(None).unwrap();
        ctx.token_q.push_back(Token {
            text: local_text,
            line_number: ctx.line_number
        });
        ctx.state = lex_default;
    } else {
        ctx.text.borrow_mut().as_mut().unwrap().push(ctx.byte.unwrap());
    }
}
