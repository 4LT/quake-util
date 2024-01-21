mod repr;

#[cfg(feature = "std")]
pub mod read;

#[cfg(feature = "std")]
mod result;

pub use repr::{Entry, Lump};

#[cfg(feature = "std")]
pub use result::{ReadError, ReadResult};
