#![no_std]

#[cfg(all(not(feature = "std"), not(feature = "hashbrown")))]
compile_error!("Must use feature 'std' or include 'hashbrown'");

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

pub mod qmap;
