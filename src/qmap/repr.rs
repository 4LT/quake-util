#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc_fills")]
extern crate alloc;

#[cfg(feature = "std")]
use std::{
    ffi::{CStr, CString},
    io,
    string::String,
    vec::Vec,
};

#[cfg(feature = "alloc_fills")]
use {
    alloc::ffi::CString, alloc::format, alloc::string::String, alloc::vec::Vec,
    core::ffi::CStr,
};

#[cfg(feature = "std")]
use crate::{WriteAttempt, WriteError};

/// Return type for validating writability of entities and other items
pub type ValidationResult = Result<(), String>;

/// Validation of entities and other items
pub trait CheckWritable {
    /// Determine if an item can be written to file
    ///
    /// Note that passing this check only implies that the item can be written
    /// to file and can also be parsed back non-destructively.  It is up to the
    /// consumer to ensure that the maps written are in a form that can be used
    /// by other tools, e.g. qbsp.
    fn check_writable(&self) -> ValidationResult;
}

/// 3-dimensional point used to determine the half-space a surface lies on
pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

/// Transparent data structure representing a Quake source map
///
/// Contains a list of entities. Internal texture alignments may be in the
/// original "legacy" Id format, the "Valve 220" format, or a mix of the two.
#[derive(Clone, Debug)]
pub struct QuakeMap {
    pub entities: Vec<Entity>,
}

impl QuakeMap {
    /// Instantiate a new map with 0 entities
    pub const fn new() -> Self {
        QuakeMap {
            entities: Vec::new(),
        }
    }

    /// Writes the map to the provided writer in text format, failing if
    /// validation fails or an I/O error occurs
    #[cfg(feature = "std")]
    pub fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        for ent in &self.entities {
            ent.write_to(writer)?;
        }
        Ok(())
    }
}

impl Default for QuakeMap {
    fn default() -> Self {
        Self::new()
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

/// Entity type: `Brush` for entities with brushes, `Point` for entities without
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityKind {
    Point,
    Brush,
}

/// A collection of key/value pairs in the form of an *edict* and 0 or more
/// brushes
#[derive(Clone, Debug)]
pub struct Entity {
    pub edict: Edict,
    pub brushes: Vec<Brush>,
}

impl Entity {
    /// Instantiate a new entity without any keyvalues or brushes
    pub fn new() -> Self {
        Entity {
            edict: Edict::new(),
            brushes: Vec::new(),
        }
    }

    /// Determine whether this is a point or brush entity
    pub fn kind(&self) -> EntityKind {
        if self.brushes.is_empty() {
            EntityKind::Point
        } else {
            EntityKind::Brush
        }
    }

    #[cfg(feature = "std")]
    pub fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        self.check_writable().map_err(WriteError::Validation)?;

        writer.write_all(b"{\r\n").map_err(WriteError::Io)?;

        write_edict_to(&self.edict, writer)?;

        for brush in &self.brushes {
            write_brush_to(brush, writer)?;
        }

        writer.write_all(b"}\r\n").map_err(WriteError::Io)?;
        Ok(())
    }
}

impl Default for Entity {
    /// Same as `Entity::new`
    fn default() -> Self {
        Entity::new()
    }
}

impl CheckWritable for Entity {
    fn check_writable(&self) -> ValidationResult {
        self.edict.check_writable()?;

        for brush in &self.brushes {
            brush.check_writable()?
        }

        Ok(())
    }
}

/// Entity dictionary
pub type Edict = Vec<(CString, CString)>;

impl CheckWritable for Edict {
    fn check_writable(&self) -> ValidationResult {
        for (k, v) in self {
            check_writable_quoted(k)?;
            check_writable_quoted(v)?;
        }

        Ok(())
    }
}

/// Convex polyhedron
pub type Brush = Vec<Surface>;

impl CheckWritable for Brush {
    fn check_writable(&self) -> ValidationResult {
        for surface in self {
            surface.check_writable()?;
        }

        Ok(())
    }
}

/// Brush face
#[derive(Clone, Debug)]
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
        write_texture_to(&self.texture, writer)?;
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

/// A set of 3 points that determine a plane with its facing direction
/// determined by the winding order of the points
pub type HalfSpace = [Point; 3];

impl CheckWritable for HalfSpace {
    fn check_writable(&self) -> ValidationResult {
        for num in self.iter().flatten() {
            check_writable_f64(*num)?;
        }

        Ok(())
    }
}

/// Texture alignment properties
///
/// If axes are present, the alignment will be written in the "Valve220" format;
/// otherwise it will be written in the "legacy" format pre-dating Valve220.
#[derive(Clone, Copy, Debug)]
pub struct Alignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2,

    /// Describes the X and Y texture-space axes
    pub axes: Option<[Vec3; 2]>,
}

impl Alignment {
    #[cfg(feature = "std")]
    fn write_to<W: io::Write>(&self, writer: &mut W) -> WriteAttempt {
        match self.axes {
            None => {
                write!(
                    writer,
                    "{} {} {} {} {}",
                    self.offset[0],
                    self.offset[1],
                    self.rotation,
                    self.scale[0],
                    self.scale[1]
                )
                .map_err(WriteError::Io)?;
            }
            Some([u, v]) => {
                write!(
                    writer,
                    "[ {} {} {} {} ] [ {} {} {} {} ] {} {} {}",
                    u[0],
                    u[1],
                    u[2],
                    self.offset[0],
                    v[0],
                    v[1],
                    v[2],
                    self.offset[1],
                    self.rotation,
                    self.scale[0],
                    self.scale[1]
                )
                .map_err(WriteError::Io)?;
            }
        }
        Ok(())
    }
}

impl CheckWritable for Alignment {
    fn check_writable(&self) -> ValidationResult {
        check_writable_array(self.offset)?;
        check_writable_f64(self.rotation)?;
        check_writable_array(self.scale)?;

        if let Some(axes) = self.axes {
            for axis in axes {
                check_writable_array(axis)?;
            }
        }

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

#[cfg(feature = "std")]
fn write_texture_to<W: io::Write>(
    texture: &CStr,
    writer: &mut W,
) -> WriteAttempt {
    let needs_quotes =
        texture.to_bytes().iter().any(|c| c.is_ascii_whitespace())
            || texture.to_bytes().is_empty();

    if needs_quotes {
        writer.write_all(b"\"").map_err(WriteError::Io)?;
    }

    writer
        .write_all(texture.to_bytes())
        .map_err(WriteError::Io)?;

    if needs_quotes {
        writer.write_all(b"\"").map_err(WriteError::Io)?;
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
