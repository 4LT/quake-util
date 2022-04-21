#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc_fills")]
extern crate alloc;

#[cfg(feature = "std")]
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    io,
    string::String,
    vec::Vec,
};

#[cfg(feature = "alloc_fills")]
use {alloc::vec::Vec, cstr_core::CString, hashbrown::HashMap};

pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];
pub type ValidationResult = Result<(), String>;

pub trait CheckWritable {
    fn check_writable(&self) -> ValidationResult;
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum WriteError {
    Validation(String),
    Io(io::Error),
}

#[cfg(feature = "std")]
pub type WriteAttempt = Result<(), WriteError>;

#[derive(Clone)]
pub struct QuakeMap {
    pub entities: Vec<Entity>,
}

impl QuakeMap {
    pub const fn new() -> Self {
        QuakeMap {
            entities: Vec::new(),
        }
    }

    #[cfg(feature = "std")]
    pub fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        for ent in &self.entities {
            ent.write_to(writer)?;
        }
        Ok(())
    }
}

impl CheckWritable for QuakeMap {
    fn check_writable(&self) -> ValidationResult {
        for ent in &self.entities {
            ent.check_writable()?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum Entity {
    Brush(Edict, Vec<Brush>),
    Point(Edict),
}

impl Entity {
    pub fn edict(&self) -> &Edict {
        match self {
            Self::Point(edict) => edict,
            Self::Brush(edict, _) => edict,
        }
    }

    pub fn edict_mut(&mut self) -> &mut Edict {
        match *self {
            Self::Point(ref mut edict) => edict,
            Self::Brush(ref mut edict, _) => edict,
        }
    }

    #[cfg(feature = "std")]
    pub fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        self.check_writable().map_err(WriteError::Validation)?;

        writer.write_all(b"{\r\n").map_err(WriteError::Io)?;

        match self {
            Entity::Brush(edict, brushes) => {
                write_edict_to(edict, writer)?;
                for brush in brushes {
                    write_brush_to(brush, writer)?;
                }
            }
            Entity::Point(edict) => {
                write_edict_to(edict, writer)?;
            }
        }

        writer.write_all(b"}\r\n").map_err(WriteError::Io)?;
        Ok(())
    }
}

impl CheckWritable for Entity {
    fn check_writable(&self) -> ValidationResult {
        self.edict().check_writable()?;

        if let Entity::Brush(_, brushes) = self {
            for brush in brushes {
                brush.check_writable()?
            }
        }

        Ok(())
    }
}

pub type Edict = HashMap<CString, CString>;

impl CheckWritable for Edict {
    fn check_writable(&self) -> ValidationResult {
        for (k, v) in self {
            check_writable_quoted(k)?;
            check_writable_quoted(v)?;
        }

        Ok(())
    }
}

pub type Brush = Vec<Surface>;

impl CheckWritable for Brush {
    fn check_writable(&self) -> ValidationResult {
        for surface in self {
            surface.check_writable()?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Surface {
    pub half_space: HalfSpace,
    pub texture: CString,
    pub alignment: Alignment,
}

impl Surface {
    #[cfg(feature = "std")]
    fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        write_half_space_to(&self.half_space, writer)?;
        writer.write_all(b" ").map_err(WriteError::Io)?;
        writer
            .write_all(self.texture.as_bytes())
            .map_err(WriteError::Io)?;
        writer.write_all(b" ").map_err(WriteError::Io)?;
        self.alignment.write_to(writer)?;
        Ok(())
    }
}

impl CheckWritable for Surface {
    fn check_writable(&self) -> ValidationResult {
        self.half_space.check_writable()?;
        check_writable_texture(&self.texture)?;
        self.alignment.check_writable()
    }
}

pub type HalfSpace = [Point; 3];

impl CheckWritable for HalfSpace {
    fn check_writable(&self) -> ValidationResult {
        for num in self.iter().flatten() {
            check_writable_f64(*num)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Alignment {
    Standard(BaseAlignment),
    Valve220(BaseAlignment, [Vec3; 2]),
}

impl Alignment {
    pub fn base(&self) -> &BaseAlignment {
        match self {
            Alignment::Standard(base) => base,
            Alignment::Valve220(base, _) => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseAlignment {
        match *self {
            Alignment::Standard(ref mut base) => base,
            Alignment::Valve220(ref mut base, _) => base,
        }
    }

    #[cfg(feature = "std")]
    fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        match self {
            Alignment::Standard(base) => {
                write!(
                    writer,
                    "{} {} {} {} {}",
                    base.offset[0],
                    base.offset[1],
                    base.rotation,
                    base.scale[0],
                    base.scale[1]
                )
                .map_err(WriteError::Io)?;
            }
            Alignment::Valve220(base, [u, v]) => {
                write!(
                    writer,
                    "[ {} {} {} {} ] [ {} {} {} {} ] {} {} {}",
                    u[0],
                    u[1],
                    u[2],
                    base.offset[0],
                    v[0],
                    v[1],
                    v[2],
                    base.offset[1],
                    base.rotation,
                    base.scale[0],
                    base.scale[1]
                )
                .map_err(WriteError::Io)?;
            }
        }
        Ok(())
    }
}

impl CheckWritable for Alignment {
    fn check_writable(&self) -> ValidationResult {
        match self {
            Alignment::Standard(base) => base.check_writable(),
            Alignment::Valve220(base, axes) => {
                base.check_writable()?;

                for axis in axes {
                    check_writable_array(*axis)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct BaseAlignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2,
}

impl CheckWritable for BaseAlignment {
    fn check_writable(&self) -> ValidationResult {
        check_writable_array(self.offset)?;
        check_writable_f64(self.rotation)?;
        check_writable_array(self.scale)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
fn write_edict_to<W: io::Write>(edict: &Edict, writer: &mut W) -> WriteAttempt {
    for (key, value) in edict {
        writer.write_all(b"\"").map_err(WriteError::Io)?;
        writer.write_all(key.as_bytes()).map_err(WriteError::Io)?;
        writer.write_all(b"\" \"").map_err(WriteError::Io)?;
        writer.write_all(value.as_bytes()).map_err(WriteError::Io)?;
        writer.write_all(b"\"\r\n").map_err(WriteError::Io)?;
    }
    Ok(())
}

#[cfg(feature = "std")]
fn write_brush_to<W: io::Write>(brush: &Brush, writer: &mut W) -> WriteAttempt {
    writer.write_all(b"{\r\n").map_err(WriteError::Io)?;

    for surf in brush {
        surf.write_to(writer)?;
        writer.write_all(b"\r\n").map_err(WriteError::Io)?;
    }

    writer.write_all(b"}\r\n").map_err(WriteError::Io)?;
    Ok(())
}

#[cfg(feature = "std")]
fn write_half_space_to<W: io::Write>(
    half_space: &HalfSpace,
    writer: &mut W,
) -> WriteAttempt {
    for (index, pt) in half_space.iter().enumerate() {
        writer.write_all(b"( ").map_err(WriteError::Io)?;

        for element in pt.iter() {
            write!(writer, "{} ", element).map_err(WriteError::Io)?;
        }

        writer.write_all(b")").map_err(WriteError::Io)?;

        if index != 2 {
            writer.write_all(b" ").map_err(WriteError::Io)?;
        }
    }
    Ok(())
}

fn check_writable_array<const N: usize>(arr: [f64; N]) -> ValidationResult {
    for num in arr {
        check_writable_f64(num)?;
    }

    Ok(())
}

fn check_writable_f64(num: f64) -> ValidationResult {
    if num.is_finite() {
        Ok(())
    } else {
        Err(format!("Non-finite number ({})", num))
    }
}

fn check_writable_texture(s: &CStr) -> ValidationResult {
    if check_writable_unquoted(s).is_ok() {
        return Ok(());
    }

    match check_writable_quoted(s) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!(
            "Cannot write texture {:?}, not quotable and contains whitespace",
            s
        )),
    }
}

fn check_writable_quoted(s: &CStr) -> ValidationResult {
    let bad_chars = [b'"', b'\r', b'\n'];

    for c in s.to_bytes() {
        if bad_chars.contains(c) {
            return Err(format!(
                "Cannot write quote-wrapped string, contains {:?}",
                char::from(*c)
            ));
        }
    }

    Ok(())
}

fn check_writable_unquoted(s: &CStr) -> ValidationResult {
    let s_bytes = s.to_bytes();

    if s_bytes.is_empty() {
        return Err(String::from("Cannot write unquoted empty string"));
    }

    if s_bytes[0] == b'"' {
        return Err(String::from("Cannot lead unquoted string with quote"));
    }

    if contains_ascii_whitespace(s) {
        Err(String::from(
            "Cannot write unquoted string, contains whitespace",
        ))
    } else {
        Ok(())
    }
}

fn contains_ascii_whitespace(s: &CStr) -> bool {
    s.to_bytes().iter().any(|c| c.is_ascii_whitespace())
}
