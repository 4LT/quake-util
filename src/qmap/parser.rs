#[cfg(feature = "std")]
extern crate std;

use std::{
    cell::Cell, io::Read, iter::Peekable, num::NonZeroU8, str::FromStr,
    vec::Vec,
};

use crate::{common, error, qmap};
use common::CellOptionExt;
use qmap::lexer::{Token, TokenIterator};
use qmap::repr::{Alignment, Brush, Edict, Entity, Point, QuakeMap, Surface};

type TokenPeekable<R> = Peekable<TokenIterator<R>>;

trait Extract {
    fn extract(&mut self) -> Result<Option<Token>, error::TextParse>;
}

impl<R> Extract for TokenPeekable<R>
where
    R: Read,
{
    fn extract(&mut self) -> Result<Option<Token>, error::TextParse> {
        self.next().transpose().map_err(|e| e.into_unwrapped())
    }
}

trait Normalize {
    fn normalize(&self) -> Result<&Option<Token>, error::TextParse>;
}

impl Normalize for Result<Option<Token>, Cell<Option<error::TextParse>>> {
    fn normalize(&self) -> Result<&Option<Token>, error::TextParse> {
        self.as_ref().map_err(|e| e.steal())
    }
}

const MIN_BRUSH_SURFACES: usize = 4;

/// Parses a Quake source map
///
/// Maps must be in the Quake 1 format (Quake 2 surface flags and Quake 3
/// `brushDef`s/`patchDef`s are not presently supported) but may have texture
/// alignment in either "Valve220" format or the "legacy" predecessor (i.e.
/// without texture axes)
pub fn parse<R: Read>(reader: &mut R) -> error::TextParseResult<QuakeMap> {
    let mut entities: Vec<Entity> = Vec::new();
    let mut peekable_tokens = TokenIterator::new(reader).peekable();

    while peekable_tokens.peek().is_some() {
        let entity = parse_entity(&mut peekable_tokens)?;
        entities.push(entity);
    }

    Ok(QuakeMap { entities })
}

fn parse_entity<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Entity> {
    expect_byte(&tokens.extract()?, b'{')?;

    let edict = parse_edict(tokens)?;
    let brushes = parse_brushes(tokens)?;

    expect_byte(&tokens.extract()?, b'}')?;

    Ok(Entity { edict, brushes })
}

fn parse_edict<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Edict> {
    let mut edict = Edict::new();

    while let Some(tok_res) = tokens.peek() {
        if tok_res
            .as_ref()
            .map_err(CellOptionExt::steal)?
            .match_quoted()
        {
            let key = strip_quoted(&tokens.extract()?.unwrap().text)
                .to_vec()
                .into();
            let maybe_value = tokens.extract()?;
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
) -> error::TextParseResult<Vec<Brush>> {
    let mut brushes = Vec::new();

    while let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.steal())?.match_byte(b'{') {
            brushes.push(parse_brush(tokens)?);
        } else {
            break;
        }
    }

    Ok(brushes)
}

fn parse_brush<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Brush> {
    let mut surfaces = Vec::with_capacity(MIN_BRUSH_SURFACES);
    expect_byte(&tokens.extract()?, b'{')?;

    while let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.steal())?.match_byte(b'(') {
            surfaces.push(parse_surface(tokens)?);
        } else {
            break;
        }
    }

    expect_byte_or(&tokens.extract()?, b'}', &[b'('])?;
    Ok(surfaces)
}

fn parse_surface<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Surface> {
    let pt1 = parse_point(tokens)?;
    let pt2 = parse_point(tokens)?;
    let pt3 = parse_point(tokens)?;

    let half_space = [pt1, pt2, pt3];

    let texture_token = &tokens.extract()?.ok_or_else(error::TextParse::eof)?;

    let texture = if b'"' == (&texture_token.text)[0].into() {
        strip_quoted(&texture_token.text[..]).to_vec().into()
    } else {
        texture_token.text.clone().into()
    };

    let alignment = if let Some(tok_res) = tokens.peek() {
        if tok_res.as_ref().map_err(|e| e.steal())?.match_byte(b'[') {
            parse_valve_alignment(tokens)?
        } else {
            parse_legacy_alignment(tokens)?
        }
    } else {
        return Err(error::TextParse::eof());
    };

    Ok(Surface {
        half_space,
        texture,
        alignment,
    })
}

fn parse_point<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Point> {
    expect_byte(&tokens.extract()?, b'(')?;
    let x = expect_float(&tokens.extract()?)?;
    let y = expect_float(&tokens.extract()?)?;
    let z = expect_float(&tokens.extract()?)?;
    expect_byte(&tokens.extract()?, b')')?;

    Ok([x, y, z])
}

fn parse_legacy_alignment<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Alignment> {
    let offset_x = expect_float(&tokens.extract()?)?;
    let offset_y = expect_float(&tokens.extract()?)?;
    let rotation = expect_float(&tokens.extract()?)?;
    let scale_x = expect_float(&tokens.extract()?)?;
    let scale_y = expect_float(&tokens.extract()?)?;

    Ok(Alignment {
        offset: [offset_x, offset_y],
        rotation,
        scale: [scale_x, scale_y],
        axes: None,
    })
}

fn parse_valve_alignment<R: Read>(
    tokens: &mut TokenPeekable<R>,
) -> error::TextParseResult<Alignment> {
    expect_byte(&tokens.extract()?, b'[')?;
    let u_x = expect_float(&tokens.extract()?)?;
    let u_y = expect_float(&tokens.extract()?)?;
    let u_z = expect_float(&tokens.extract()?)?;
    let offset_x = expect_float(&tokens.extract()?)?;
    expect_byte(&tokens.extract()?, b']')?;

    expect_byte(&tokens.extract()?, b'[')?;
    let v_x = expect_float(&tokens.extract()?)?;
    let v_y = expect_float(&tokens.extract()?)?;
    let v_z = expect_float(&tokens.extract()?)?;
    let offset_y = expect_float(&tokens.extract()?)?;
    expect_byte(&tokens.extract()?, b']')?;

    let rotation = expect_float(&tokens.extract()?)?;
    let scale_x = expect_float(&tokens.extract()?)?;
    let scale_y = expect_float(&tokens.extract()?)?;

    Ok(Alignment {
        offset: [offset_x, offset_y],
        rotation,
        scale: [scale_x, scale_y],
        axes: Some([[u_x, u_y, u_z], [v_x, v_y, v_z]]),
    })
}

fn expect_byte(token: &Option<Token>, byte: u8) -> error::TextParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_byte(byte) => Ok(()),
        Some(payload) => Err(error::TextParse::from_parser(
            format!(
                "Expected `{}`, got `{}`",
                char::from(byte),
                payload.text_as_string()
            ),
            payload.line_number,
        )),
        _ => Err(error::TextParse::eof()),
    }
}

fn expect_byte_or(
    token: &Option<Token>,
    byte: u8,
    rest: &[u8],
) -> error::TextParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_byte(byte) => Ok(()),
        Some(payload) => {
            let rest_str = rest
                .iter()
                .copied()
                .map(|b| format!("`{}`", char::from(b)))
                .collect::<Vec<_>>()[..]
                .join(", ");

            Err(error::TextParse::from_parser(
                format!(
                    "Expected {} or `{}`, got `{}`",
                    rest_str,
                    char::from(byte),
                    payload.text_as_string()
                ),
                payload.line_number,
            ))
        }
        _ => Err(error::TextParse::eof()),
    }
}

fn expect_quoted(token: &Option<Token>) -> error::TextParseResult<()> {
    match token.as_ref() {
        Some(payload) if payload.match_quoted() => Ok(()),
        Some(payload) => Err(error::TextParse::from_parser(
            format!("Expected quoted, got `{}`", payload.text_as_string()),
            payload.line_number,
        )),
        _ => Err(error::TextParse::eof()),
    }
}

fn expect_float(token: &Option<Token>) -> error::TextParseResult<f64> {
    match token.as_ref() {
        Some(payload) => match f64::from_str(&payload.text_as_string()) {
            Ok(num) => Ok(num),
            Err(_) => Err(error::TextParse::from_parser(
                format!("Expected number, got `{}`", payload.text_as_string()),
                payload.line_number,
            )),
        },
        None => Err(error::TextParse::eof()),
    }
}

fn strip_quoted(quoted_text: &[NonZeroU8]) -> &[NonZeroU8] {
    &quoted_text[1..quoted_text.len() - 1]
}
