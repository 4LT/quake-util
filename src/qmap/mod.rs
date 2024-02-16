//! Quake source map data structures, parsing, and writing
//!
//! # Example
//!
//! ```
//! # use std::ffi::CString;
//! # use std::io::Read;
//! #
//! #
//! # fn main() -> Result<(), String> {
//! #   #[cfg(feature="std")]
//! #   {
//! #       let mut source = &b"
//! #           {
//! #           }
//! #           "[..];
//! #
//! #       let mut dest = Vec::<u8>::new();
//! #
//! use quake_util::{qmap, WriteError, TextParseError};
//! use qmap::Entity;
//!
//! let mut map = qmap::parse(&mut source).map_err(|err| match err {
//!     TextParseError::Io(_) => String::from("Failed to read map"),
//!     l_err@TextParseError::Lexer(_) => l_err.to_string(),
//!     p_err@TextParseError::Parser(_) => p_err.to_string(),
//! })?;
//!
//! let mut soldier = Entity::new();
//!
//! soldier.edict.insert(
//!     CString::new("classname").unwrap(),
//!     CString::new("monster_army").unwrap(),
//! );
//!
//! soldier.edict.insert(
//!     CString::new("origin").unwrap(),
//!     CString::new("128 -256 24").unwrap(),
//! );
//!
//! map.entities.push(soldier);
//!
//! map.write_to(&mut dest).map_err(|err| match err {
//!     WriteError::Io(e) => e.to_string(),
//!     WriteError::Validation(msg) => msg
//! })?;
//! #  
//! #   }
//! #   Ok(())
//! # }
//! ```

mod repr;

#[cfg(feature = "std")]
mod lexer;

#[cfg(feature = "std")]
mod parser;

pub use repr::{
    Alignment, Brush, CheckWritable, Edict, Entity, EntityKind, HalfSpace,
    Point, QuakeMap, Surface, ValidationResult, Vec2, Vec3,
};

#[cfg(feature = "std")]
pub use parser::parse;

// test suites

#[cfg(all(test, feature = "std"))]
mod parser_test;

#[cfg(all(test, feature = "std"))]
mod lexer_test;

#[cfg(test)]
mod repr_test;
