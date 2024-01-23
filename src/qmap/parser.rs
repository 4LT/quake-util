#[cfg(feature = "std")]
extern crate std;

use std::{io::Read, iter::Peekable, num::NonZeroU8, str::FromStr, vec::Vec};

use crate::qmap;
use qmap::lexer::{Token, TokenIterator};
use qmap::repr::{Alignment, Brush, Edict, Entity, Point, QuakeMap, Surface};
use qmap::ParseResult;

type TokenPeekable<R> = Peekable<TokenIterator<R>>;
const MIN_BRUSH_SURFACES: usize = 4;

/// Parses a Quake source map
///
/// Maps must be in the Quake 1 format (Quake 2 surface flags and Quake 3
/// `brushDef`s/`patchDef`s are not presently supported) but may have texture
/// alignment in either "Valve220" format or the "legacy" predecessor (i.e.
/// without texture axes)
pub fn parse<R: Read>(reader: R) -> ParseResult<QuakeMap> {
    let mut entities: Vec<Entity> = Vec::new();
    let mut peekable_tokens = TokenIterator::new(reader).peekable();

    while peekable_tokens.peek().is_some() {
        let entity = parse_entity(&mut peekable_tokens)?;
        entities.push(entity);
    }

    Ok(QuakeMap { entities })
}

fn parse_entity<R: Read>(tokens: &mut TokenPeekable<R>) -> ParseResult<Entity> {
    expect_byte(&tokens.next().transpose()?, b'{')?;

    let edict = parse_edict(tokens)?;
    let brushes = parse_brushes(tokens)?;

    expect_byte(&tokens.next().transpose()?, b'}')?;

    Ok(Entity { edict, brushes })
}

fn parse_edict<R: Read>(tokens: &mut TokenPeekable<R>) -> ParseResult<Edict> {
    let mut edict = Edict::new();

    while let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.clone())?.match_quoted() {
            let key = strip_quoted(&tokens.next().transpose()?.unwrap().text)
                .to_vec()
                .into();
            let maybe_value = tokens.next().transpose()?;
            expect_quoted(&maybe_value)?;
            let value =
                strip_quoted(&maybe_value.unwrap().text).to_vec().into();
            edict.insert(key, value);
        } else {
            break;
        }
    }

    Ok(edict)
}

fn parse_brushes<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> ParseResult<Vec<Brush>> {
    let mut brushes = Vec::new();

    while let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.clone())?.match_byte(b'{') {
            brushes.push(parse_brush(tokens)?);
        } else {
            break;
        }
    }

    Ok(brushes)
}

fn parse_brush<R: Read>(tokens: &mut TokenPeekable<R>) -> ParseResult<Brush> {
    let mut surfaces = Vec::with_capacity(MIN_BRUSH_SURFACES);
    expect_byte(&tokens.next().transpose()?, b'{')?;

    while let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.clone())?.match_byte(b'(') {
            surfaces.push(parse_surface(tokens)?);
        } else {
            break;
        }
    }

    expect_byte_or(&tokens.next().transpose()?, b'}', &[b'('])?;
    Ok(surfaces)
}

fn parse_surface<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> ParseResult<Surface> {
    let pt1 = parse_point(tokens)?;
    let pt2 = parse_point(tokens)?;
    let pt3 = parse_point(tokens)?;

    let half_space = [pt1, pt2, pt3];

    let texture_token = &tokens
        .next()
        .transpose()?
        .ok_or_else(qmap::ParseError::eof)?;

    let texture = if b'"' == (&texture_token.text)[0].into() {
        strip_quoted(&texture_token.text[..]).to_vec().into()
    } else {
        texture_token.text.clone().into()
    };

    let alignment = if let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.clone())?.match_byte(b'[') {
            parse_valve_alignment(tokens)?
        } else {
            parse_legacy_alignment(tokens)?
        }
    } else {
        return Err(qmap::ParseError::eof());
    };

    Ok(Surface {
        half_space,
        texture,
        alignment,
    })
}

fn parse_point<R: Read>(tokens: &mut TokenPeekable<R>) -> ParseResult<Point> {
    expect_byte(&tokens.next().transpose()?, b'(')?;
    let x = expect_float(&tokens.next().transpose()?)?;
    let y = expect_float(&tokens.next().transpose()?)?;
    let z = expect_float(&tokens.next().transpose()?)?;
    expect_byte(&tokens.next().transpose()?, b')')?;

    Ok([x, y, z])
}

fn parse_legacy_alignment<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> ParseResult<Alignment> {
    let offset_x = expect_float(&tokens.next().transpose()?)?;
    let offset_y = expect_float(&tokens.next().transpose()?)?;
    let rotation = expect_float(&tokens.next().transpose()?)?;
    let scale_x = expect_float(&tokens.next().transpose()?)?;
    let scale_y = expect_float(&tokens.next().transpose()?)?;

    Ok(Alignment {
        offset: [offset_x, offset_y],
        rotation,
        scale: [scale_x, scale_y],
        axes: None,
    })
}

fn parse_valve_alignment<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> ParseResult<Alignment> {
    expect_byte(&tokens.next().transpose()?, b'[')?;
    let u_x = expect_float(&tokens.next().transpose()?)?;
    let u_y = expect_float(&tokens.next().transpose()?)?;
    let u_z = expect_float(&tokens.next().transpose()?)?;
    let offset_x = expect_float(&tokens.next().transpose()?)?;
    expect_byte(&tokens.next().transpose()?, b']')?;

    expect_byte(&tokens.next().transpose()?, b'[')?;
    let v_x = expect_float(&tokens.next().transpose()?)?;
    let v_y = expect_float(&tokens.next().transpose()?)?;
    let v_z = expect_float(&tokens.next().transpose()?)?;
    let offset_y = expect_float(&tokens.next().transpose()?)?;
    expect_byte(&tokens.next().transpose()?, b']')?;

    let rotation = expect_float(&tokens.next().transpose()?)?;
    let scale_x = expect_float(&tokens.next().transpose()?)?;
    let scale_y = expect_float(&tokens.next().transpose()?)?;

    Ok(Alignment {
        offset: [offset_x, offset_y],
        rotation,
        scale: [scale_x, scale_y],
        axes: Some([[u_x, u_y, u_z], [v_x, v_y, v_z]]),
    })
}

fn expect_byte(token: &Option<Token>, byte: u8) -> ParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_byte(byte) => Ok(()),
        Some(payload) => Err(qmap::ParseError::from_parser(
            format!(
                "Expected `{}`, got `{}`",
                char::from(byte),
                payload.text_as_string()
            ),
            payload.line_number,
        )),
        _ => Err(qmap::ParseError::eof()),
    }
}

fn expect_byte_or(
    token: &Option<Token>,
    byte: u8,
    rest: &[u8],
) -> ParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_byte(byte) => Ok(()),
        Some(payload) => {
            let rest_str = rest
                .iter()
                .copied()
                .map(|b| format!("`{}`", char::from(b)))
                .collect::<Vec<_>>()[..]
                .join(", ");

            Err(qmap::ParseError::from_parser(
                format!(
                    "Expected {} or `{}`, got `{}`",
                    rest_str,
                    char::from(byte),
                    payload.text_as_string()
                ),
                payload.line_number,
            ))
        }
        _ => Err(qmap::ParseError::eof()),
    }
}

fn expect_quoted(token: &Option<Token>) -> ParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_quoted() => Ok(()),
        Some(payload) => Err(qmap::ParseError::from_parser(
            format!("Expected quoted, got `{}`", payload.text_as_string()),
            payload.line_number,
        )),
        _ => Err(qmap::ParseError::eof()),
    }
}

fn expect_float(token: &Option<Token>) -> ParseResult<f64> {
    match token.as_ref() {
        Some(payload) => match f64::from_str(&payload.text_as_string()) {
            Ok(num) => Ok(num),
            Err(_) => Err(qmap::ParseError::from_parser(
                format!("Expected number, got `{}`", payload.text_as_string()),
                payload.line_number,
            )),
        },
        None => Err(qmap::ParseError::eof()),
    }
}

fn strip_quoted(quoted_text: &[NonZeroU8]) -> &[NonZeroU8] {
    &quoted_text[1..quoted_text.len() - 1]
}
