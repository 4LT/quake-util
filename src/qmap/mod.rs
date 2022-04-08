pub mod repr;

#[cfg(feature = "std")]
mod lexer;

#[cfg(feature = "std")]
pub mod parser;

#[cfg(feature = "std")]
pub mod result;

pub use repr::{
    Alignment, BaseAlignment, Brush, Edict, Entity, HalfSpace, Point, QuakeMap,
    Surface, Validates, Vec2, Vec3,
};

#[cfg(feature = "std")]
pub use repr::Writes;

#[cfg(feature = "std")]
pub use parser::parse;

#[cfg(feature = "std")]
pub use result::{Error, LineError, Result};

// test suites

#[cfg(all(test, feature = "std"))]
mod parser_test;

#[cfg(all(test, feature = "std"))]
mod lexer_test;
