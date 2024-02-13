#![no_std]

#[cfg(all(not(feature = "std"), not(feature = "alloc_fills")))]
compile_error!("Must use feature 'std' or 'alloc_fills'");

#[cfg(all(feature = "std", feature = "alloc_fills"))]
compile_error!("Features 'std' and 'alloc_fills' are mutually exclusive");

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
pub mod error;

#[cfg(feature = "std")]
pub mod lump;

#[cfg(feature = "std")]
pub mod wad;

pub mod qmap;
