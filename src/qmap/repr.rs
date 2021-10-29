#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "hashbrown")]
use hashbrown::{HashMap, HashSet};

#[cfg(not(feature = "hashbrown"))]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "cstr_core")]
use cstr_core::{CStr, CString};

#[cfg(not(feature = "cstr_core"))]
use std::ffi::{CStr, CString};

#[cfg(feature = "std")]
use std::{io, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

pub type ValidationResult = core::result::Result<(), String>;
pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

#[cfg(feature = "std")]
pub trait Writes<W: io::Write> {
    fn write_to(&self, writer: &mut W) -> io::Result<()>;
}

pub trait Validates {
    fn validate(&self) -> Result<(), String>;
}

#[cfg(feature = "std")]
pub trait AstElement<W: io::Write>: Writes<W> + Validates {}

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

impl Validates for QuakeMap {
    fn validate(&self) -> ValidationResult {
        for ent in &self.entities {
            ent.validate()?;
        }

        Ok(())
    }
}

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
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Entity {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
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

impl Validates for Entity {
    fn validate(&self) -> ValidationResult {
        self.edict().validate()?;

        if let Entity::Brush(_, brushes) = self {
            for brush in brushes {
                brush.validate()?;
            }
        }

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

impl Validates for Edict {
    fn validate(&self) -> ValidationResult {
        for (k, v) in self {
            validate_keyvalue(k)?;
            validate_keyvalue(v)?;
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

impl Validates for Brush {
    fn validate(&self) -> ValidationResult {
        for surface in self {
            surface.validate()?;
        }

        Ok(())
    }
}

pub struct Surface {
    pub half_space: HalfSpace,
    pub texture: CString,
    pub alignment: Alignment,
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Surface {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        self.half_space.write_to(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(self.texture.as_bytes())?;
        writer.write_all(b" ")?;
        self.alignment.write_to(writer)?;
        Ok(())
    }
}

impl Validates for Surface {
    fn validate(&self) -> ValidationResult {
        self.half_space.validate()?;
        validate_texture(&self.texture)?;
        self.alignment.validate()
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

impl Validates for HalfSpace {
    #[allow(clippy::float_cmp)]
    fn validate(&self) -> ValidationResult {
        for point in self {
            point.validate()?;
        }

        Ok(())
    }
}

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
}

#[cfg(feature = "std")]
impl<W: io::Write> Writes<W> for Alignment {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
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

impl Validates for Alignment {
    fn validate(&self) -> ValidationResult {
        match self {
            Alignment::Standard(base) => base.validate(),
            Alignment::Valve220(base, axes) => {
                base.validate()?;
                for axis in axes {
                    axis.validate()?;
                }
                Ok(())
            }
        }
    }
}

pub struct BaseAlignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2,
}

impl Validates for BaseAlignment {
    fn validate(&self) -> ValidationResult {
        self.offset.validate()?;
        self.rotation.validate()?;
        self.scale.validate()
    }
}

impl<const N: usize> Validates for [f64; N] {
    fn validate(&self) -> ValidationResult {
        for num in self {
            num.validate()?;
        }

        Ok(())
    }
}

impl Validates for f64 {
    fn validate(&self) -> ValidationResult {
        if self.is_finite() {
            Ok(())
        } else {
            Err(format!("Non-finite number ({})", *self))
        }
    }
}

fn validate_keyvalue(s: &CStr) -> ValidationResult {
    #[cfg(feature = "hashbrown")]
    let bad_chars: HashSet<u8> = [b'"', b'\r', b'\n'].iter().cloned().collect();
    #[cfg(not(feature = "hashbrown"))]
    let bad_chars = HashSet::from([b'"', b'\r', b'\n']);
    validate_cstr(s, bad_chars)
}

fn validate_texture(s: &CStr) -> ValidationResult {
    #[cfg(feature = "hashbrown")]
    let bad_chars: HashSet<u8> =
        [b' ', b'\t', b'\r', b'\n'].iter().cloned().collect();
    #[cfg(not(feature = "hashbrown"))]
    let bad_chars = HashSet::from([b' ', b'\t', b'\r', b'\n']);
    validate_cstr(s, bad_chars)
}

fn validate_cstr(s: &CStr, bad_chars: HashSet<u8>) -> ValidationResult {
    for ch in s.to_bytes() {
        if bad_chars.contains(ch) {
            return Err(format!("Key/value has illegal character ({})", ch));
        }
    }

    Ok(())
}
