mod repr;

#[cfg(feature = "std")]
mod parser;

#[cfg(feature = "std")]
pub use parser::Parser;

pub use repr::Entry;
