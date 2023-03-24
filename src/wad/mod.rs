mod repr;

#[cfg(feature = "std")]
mod parser;

pub use repr::{Lump, Wad, WadEntry};

#[cfg(feature = "std")]
pub use parser::{parse_directory, parse_lump};
