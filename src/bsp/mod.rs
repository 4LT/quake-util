mod repr;

mod parser;

pub use repr::{Entry, EntryOffset, BSP2_VERSION, BSP_VERSION};

pub(crate) use repr::Head;

pub use parser::Parser;

#[cfg(test)]
mod repr_test;

#[cfg(test)]
mod parser_test;
