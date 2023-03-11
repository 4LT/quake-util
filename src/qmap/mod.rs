pub mod repr;

#[cfg(feature = "std")]
mod lexer;

#[cfg(feature = "std")]
pub mod parser;

#[cfg(feature = "std")]
pub mod parse_result;

pub use repr::{
    Alignment, Brush, CheckWritable, Edict, Entity, EntityKind, HalfSpace,
    Point, QuakeMap, Surface, ValidationResult, Vec2, Vec3,
};

#[cfg(feature = "std")]
pub use parser::parse;

#[cfg(feature = "std")]
pub use parse_result::{LineError, ParseError, ParseResult};

#[cfg(feature = "std")]
pub use repr::{WriteAttempt, WriteError};

// test suites

#[cfg(all(test, feature = "std"))]
mod parser_test;

#[cfg(all(test, feature = "std"))]
mod lexer_test;

#[cfg(test)]
mod repr_test;
