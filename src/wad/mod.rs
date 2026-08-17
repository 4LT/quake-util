//! Quake WAD parsing
//!
//! # Example
//! ```
//! # use quake_util::wad;
//! #
//! # fn main () {
//! # let bytes = Vec::new();
//! # let mut src = std::io::Cursor::new(bytes);
//!
//! if let Ok(mut parser) = wad::Parser::new(&mut src) {
//!     let dir = parser.directory().to_vec();
//!
//!     for entry in dir {
//!         let entry_name = entry.name_to_string().expect("bad name");
//!
//!         let kind = parser.parse_inferred(&entry).map(
//!             |lump| lump.kind().to_string(),
//!         ).unwrap_or(
//!             "<error>".to_string()
//!         );
//!
//!         println!("Entry {entry_name} has lump type {kind}");
//!     }
//! } else {
//!     eprintln!("Error loading WAD");
//! }
//!
//! # }
//! ```

mod repr;

mod parser;

pub use parser::Parser;

pub use repr::Entry;

#[cfg(test)]
mod repr_test;

#[cfg(test)]
mod parser_test;
