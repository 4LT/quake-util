#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
mod common;

#[cfg(feature = "std")]
pub use common::{Palette, QUAKE_PALETTE};

#[cfg(feature = "std")]
use common::slice_to_cstring;

#[cfg(feature = "std")]
pub mod lump;

#[cfg(feature = "std")]
pub mod wad;

#[cfg(feature = "std")]
pub mod bsp;

pub mod qmap;

#[cfg(feature = "std")]
mod error;

#[cfg(feature = "std")]
pub use error::BinParse as BinParseError;

#[cfg(feature = "std")]
pub use error::TextParse as TextParseError;

#[cfg(feature = "std")]
pub use error::Write as WriteError;

#[cfg(feature = "std")]
pub type BinParseResult<T> = Result<T, BinParseError>;

#[cfg(feature = "std")]
pub type TextParseResult<T> = std::result::Result<T, TextParseError>;

#[cfg(feature = "std")]
pub type WriteAttempt = Result<(), WriteError>;
