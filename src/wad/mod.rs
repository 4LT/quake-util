mod repr;

mod parser;

pub use parser::Parser;

pub use repr::Entry;

#[cfg(test)]
mod repr_test;
