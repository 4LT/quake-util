mod repr;

#[cfg(feature = "std")]
mod parser;

pub use repr::{Entry, Lump};

#[cfg(feature = "std")]
pub use parser::{parse_directory, parse_lump};
