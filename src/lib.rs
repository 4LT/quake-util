#![no_std]
#![cfg_attr(feature = "std", feature(io_error_other))]

#[cfg(all(not(feature = "std"), not(feature = "alloc_fills")))]
compile_error!("Must use feature 'std' or 'alloc_fills'");

#[cfg(all(feature = "std", feature = "alloc_fills"))]
compile_error!("Features 'std' and 'alloc_fills' are mutually exclusive");

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod common;
pub mod qmap;
pub mod wad;
