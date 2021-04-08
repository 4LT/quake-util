use std::collections::VecDeque;
use std::str::FromStr;

use crate::qmap::lexer::Token;
use crate::qmap::quake_map_elements::{
    QuakeMap,
    Entity,
    Edict,
    Brush,
    Surface,
    HalfSpace,
    Alignment,
    BaseAlignment,
    Point,
};

const MIN_BRUSH_SURFACES: usize = 4;

pub struct ParseError {
    pub token: Option<Token>,
    message: String
}

impl ParseError {
    pub fn new(token: Option<Token>, message: String) -> ParseError {
        ParseError{ token, message }
    }

    pub fn eof() -> ParseError {
        ParseError{ token: None, message: String::from("Unexpected EOF") }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.token.as_ref() {
            Some(tok) => write!(
                f,
                "Line {}: {}",
                tok.line_number,
                self.message
            ),
            None => write!(f, "{}", self.message)
        }
    }
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub fn parse(mut tokens: VecDeque<Token>) -> ParseResult<QuakeMap> {
    let mut entities: Vec<Entity> = Vec::new();

    while !tokens.is_empty() {
        let entity = parse_entity(&mut tokens)?;
        entities.push(entity);
    }

    Ok(QuakeMap{entities})
}

fn parse_entity(tokens: &mut VecDeque<Token>) -> ParseResult<Entity> {
    expect_byte(tokens.pop_front().as_ref(), b'{')?;

    let edict = parse_edict(tokens)?;
    let brushes = parse_brushes(tokens)?; 

    expect_byte(tokens.pop_front().as_ref(), b'}')?;

    match brushes.len() {
        0 => Ok(Entity::Point(edict)),
        _ => Ok(Entity::Brush(edict, brushes)),
    }
}

fn parse_edict(tokens: &mut VecDeque<Token>) -> ParseResult<Edict> {
    let mut edict = Edict::new();

    while tokens.front().map_or(false, |tok| tok.match_quoted()) {
        let key = strip_quoted(&tokens.pop_front().as_ref().unwrap().text);
        let maybe_value = tokens.pop_front();
        expect_quoted(maybe_value.as_ref())?;
        let value = strip_quoted(&maybe_value.unwrap().text);
        edict.insert(key, value);
    }

    Ok(edict)
}

fn parse_brushes(tokens: &mut VecDeque<Token>) -> ParseResult<Vec<Brush>> {
    let mut brushes = Vec::new();

    while tokens.front().map_or(false, |tok| tok.match_byte(b'{')) {
        brushes.push(parse_brush(tokens)?);
    }

    Ok(brushes)
}

fn parse_brush(tokens: &mut VecDeque<Token>) -> ParseResult<Brush> {
    let mut surfaces = Vec::with_capacity(MIN_BRUSH_SURFACES);
    expect_byte(tokens.pop_front().as_ref(), b'{')?;

    while tokens.front().map_or(false, |tok| tok.match_byte(b'(')) {
        surfaces.push(parse_surface(tokens)?);
    }

    expect_byte(tokens.pop_front().as_ref(), b'}')?;
    Ok(surfaces)
}

fn parse_surface(tokens: &mut VecDeque<Token>) -> ParseResult<Surface> {
    let pt1 = parse_point(tokens)?;
    let pt2 = parse_point(tokens)?;
    let pt3 = parse_point(tokens)?;

    let half_space = HalfSpace(pt1, pt2, pt3);

    let texture = unwrap_token(tokens.pop_front().as_ref())?.text.clone();

    let alignment = if tokens.front().map_or(false, |tok| tok.match_byte(b'[')) {
        parse_valve_alignment(tokens)?
    } else {
        parse_standard_alignment(tokens)?
    };

    Ok(Surface{ half_space, texture, alignment })
}

fn parse_point(tokens: &mut VecDeque<Token>) -> ParseResult<Point> {
    expect_byte(tokens.pop_front().as_ref(), b'(')?;
    let x = expect_float(tokens.pop_front().as_ref())?;
    let y = expect_float(tokens.pop_front().as_ref())?;
    let z = expect_float(tokens.pop_front().as_ref())?;
    expect_byte(tokens.pop_front().as_ref(), b')')?;

    Ok([x, y, z])
}

fn parse_standard_alignment(tokens: &mut VecDeque<Token>) -> ParseResult<Alignment> {
    let offset_x = expect_float(tokens.pop_front().as_ref())?;
    let offset_y = expect_float(tokens.pop_front().as_ref())?;
    let rotation = expect_float(tokens.pop_front().as_ref())?;
    let scale_x = expect_float(tokens.pop_front().as_ref())?;
    let scale_y = expect_float(tokens.pop_front().as_ref())?;

    Ok(Alignment::Standard(BaseAlignment{
        offset: [offset_x, offset_y],
        rotation,
        scale: [scale_x, scale_y],
    }))
}

fn parse_valve_alignment(tokens: &mut VecDeque<Token>) -> ParseResult<Alignment> {
    expect_byte(tokens.pop_front().as_ref(), b'[')?;
    let u_x = expect_float(tokens.pop_front().as_ref())?;
    let u_y = expect_float(tokens.pop_front().as_ref())?;
    let u_z = expect_float(tokens.pop_front().as_ref())?;
    let offset_x = expect_float(tokens.pop_front().as_ref())?;
    expect_byte(tokens.pop_front().as_ref(), b']')?;

    expect_byte(tokens.pop_front().as_ref(), b'[')?;
    let v_x = expect_float(tokens.pop_front().as_ref())?;
    let v_y = expect_float(tokens.pop_front().as_ref())?;
    let v_z = expect_float(tokens.pop_front().as_ref())?;
    let offset_y = expect_float(tokens.pop_front().as_ref())?;
    expect_byte(tokens.pop_front().as_ref(), b']')?;

    let rotation = expect_float(tokens.pop_front().as_ref())?;
    let scale_x = expect_float(tokens.pop_front().as_ref())?;
    let scale_y = expect_float(tokens.pop_front().as_ref())?;

    Ok(Alignment::Valve220{
        base: BaseAlignment{
            offset: [offset_x, offset_y],
            rotation,
            scale: [scale_x, scale_y],
        },
        u: [u_x, u_y, u_z],
        v: [v_x, v_y, v_z],
    })
}

fn expect_byte(token: Option<&Token>, byte: u8) -> ParseResult<()> {
    match token {
        Some(payload) if payload.match_byte(byte) => Ok(()),
        Some(payload) => Err(ParseError::new(
                Some(payload.clone()),
                format!(
                    "Expected `{}`, got `{}`",
                    char::from(byte),
                    String::from_utf8_lossy(&payload.text)))),
        _ => Err(ParseError::eof()),
    }
}

fn expect_quoted(token: Option<&Token>) -> ParseResult<()> {
    match token {
        Some(payload) if payload.match_quoted() => Ok(()),
        Some(payload) => Err(ParseError::new(
                Some(payload.clone()),
                format!(
                    "Expected quoted, got `{}`",
                    String::from_utf8_lossy(&payload.text)))),
        _ => Err(ParseError::eof()),
    }
}

fn expect_float(token: Option<&Token>) -> ParseResult<f64> {
    match token {
        Some(payload) =>
            match f64::from_str(&String::from_utf8_lossy(&payload.text)) {
                Ok(num) => Ok(num),
                Err(_) => {
                    Err(ParseError::new(
                        Some(payload.clone()),
                        format!(
                            "Expected number, got `{}`",
                            String::from_utf8_lossy(&payload.text))))
                }
            }, 
        None => Err(ParseError::eof()),
    }
}

fn unwrap_token(token: Option<&Token>) -> ParseResult<&Token> {
    match token {
        Some(payload) => Ok(payload),
        None => Err(ParseError::eof())
    }
}

fn strip_quoted(quoted_text: &[u8]) -> Vec<u8> {
    quoted_text[1 .. quoted_text.len()-1].to_vec()
}
