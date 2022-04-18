#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

#[cfg(feature = "cstr_core")]
use cstr_core::CString;

#[cfg(not(feature = "cstr_core"))]
use std::ffi::{CStr, CString};

#[cfg(feature = "std")]
use std::{io, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

#[cfg(feature = "std")]
pub trait Writes<W: io::Write> {
    fn write_to(&self, writer: &mut W) -> io::Result<()>;
}

#[derive(Clone)]
pub struct QuakeMap {
    pub entities: Vec<Entity>,
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for QuakeMap {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for ent in &self.entities {
            ent.write_to(writer)?;
        }
        Ok(())
    }
}

impl QuakeMap {
    pub const fn new() -> Self {
        QuakeMap {
            entities: Vec::new(),
        }
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
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Entity {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for (k, v) in self.edict() {
            check_writable_quoted(k)?;
            check_writable_quoted(v)?;
        }

        writer.write_all(b"{\r\n")?;

        match self {
            Entity::Brush(edict, brushes) => {
                edict.write_to(writer)?;
                for brush in brushes {
                    brush.write_to(writer)?;
                }
            }
            Entity::Point(edict) => {
                edict.write_to(writer)?;
            }
        }

        writer.write_all(b"}\r\n")?;
        Ok(())
    }
}

pub type Edict = HashMap<CString, CString>;

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Edict {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for (key, value) in self {
            writer.write_all(b"\"")?;
            writer.write_all(key.as_bytes())?;
            writer.write_all(b"\" \"")?;
            writer.write_all(value.as_bytes())?;
            writer.write_all(b"\"\r\n")?;
        }
        Ok(())
    }
}

pub type Brush = Vec<Surface>;

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Brush {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"{\r\n")?;

        for surf in self {
            surf.write_to(writer)?;
            writer.write_all(b"\r\n")?;
        }

        writer.write_all(b"}\r\n")?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Surface {
    pub half_space: HalfSpace,
    pub texture: CString,
    pub alignment: Alignment,
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Surface {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for num in self.half_space.iter().flatten() {
            check_writable_f64(*num)?;
        }
        check_writable_texture(&self.texture)?;

        self.half_space.write_to(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(self.texture.as_bytes())?;
        writer.write_all(b" ")?;
        self.alignment.write_to(writer)?;
        Ok(())
    }
}

pub type HalfSpace = [Point; 3];

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for HalfSpace {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for (index, pt) in self.iter().enumerate() {
            writer.write_all(b"( ")?;

            for element in pt.iter() {
                write!(writer, "{} ", element)?;
            }

            writer.write_all(b")")?;

            if index != 2 {
                writer.write_all(b" ")?;
            }
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
    fn check_writable(&self) -> io::Result<()> {
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

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Alignment {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        self.check_writable()?;

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
                )?;
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
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct BaseAlignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2,
}

#[cfg(feature = "std")]
impl BaseAlignment {
    fn check_writable(&self) -> io::Result<()> {
        check_writable_array(self.offset)?;
        check_writable_f64(self.rotation)?;
        check_writable_array(self.scale)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
fn check_writable_array<const N: usize>(arr: [f64; N]) -> io::Result<()> {
    for num in arr {
        check_writable_f64(num)?;
    }

    Ok(())
}

#[cfg(feature = "std")]
fn check_writable_f64(num: f64) -> io::Result<()> {
    if num.is_finite() {
        Ok(())
    } else {
        Err(io::Error::other(format!("Non-finite number ({})", num)))
    }
}

#[cfg(feature = "std")]
fn check_writable_texture(s: &CStr) -> io::Result<()> {
    if check_writable_unquoted(s).is_ok() {
        return Ok(());
    }

    match check_writable_quoted(s) {
        Ok(_) => Ok(()),
        Err(_) => Err(io::Error::other(format!(
            "Cannot write texture {:?}, not quotable and contains whitespace",
            s
        ))),
    }
}

#[cfg(feature = "std")]
fn check_writable_quoted(s: &CStr) -> io::Result<()> {
    let bad_chars = [b'"', b'\r', b'\n'];

    for c in s.to_bytes() {
        if bad_chars.contains(c) {
            return Err(io::Error::other(format!(
                "Cannot write quote-wrapped string, contains {:?}",
                char::from(*c)
            )));
        }
    }

    Ok(())
}

#[cfg(feature = "std")]
fn check_writable_unquoted(s: &CStr) -> io::Result<()> {
    let s_bytes = s.to_bytes();

    if s_bytes.is_empty() {
        return Err(io::Error::other("Cannot write unquoted empty string"));
    }

    if s_bytes[0] == b'"' {
        return Err(io::Error::other("Cannot lead unquoted string with quote"));
    }

    if contains_ascii_whitespace(s) {
        Err(io::Error::other(
            "Cannot write unquoted string, contains whitespace",
        ))
    } else {
        Ok(())
    }
}

#[cfg(feature = "std")]
fn contains_ascii_whitespace(s: &CStr) -> bool {
    s.to_bytes().iter().any(|c| c.is_ascii_whitespace())
}
