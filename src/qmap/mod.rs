pub mod repr;

#[cfg(feature = "std")]
mod lexer;

#[cfg(feature = "std")]
pub mod parser;

#[cfg(feature = "std")]
pub mod parse_result;

pub mod repr_write;

pub use repr::{
    Alignment, BaseAlignment, Brush, Edict, Entity, HalfSpace, Point, QuakeMap,
    Surface, Vec2, Vec3,
};

#[cfg(feature = "std")]
pub use parser::parse;

#[cfg(feature = "std")]
pub use parse_result::{LineError, ParseError, ParseResult};

pub use repr_write::{CheckWritable, ValidationResult};

#[cfg(feature = "std")]
pub use repr_write::{WriteAttempt, WriteError};

// test suites

#[cfg(all(test, feature = "std"))]
mod parser_test;

#[cfg(all(test, feature = "std"))]
mod lexer_test;

#[cfg(test)]
mod repr_test;
