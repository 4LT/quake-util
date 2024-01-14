mod repr;

#[cfg(feature = "std")]
mod parser;

#[cfg(feature = "std")]
mod result;

pub use repr::{Entry, Lump};

#[cfg(feature = "std")]
pub use parser::{parse_directory, parse_lump};

#[cfg(feature = "std")]
pub use result::{ReadError, ReadResult};
