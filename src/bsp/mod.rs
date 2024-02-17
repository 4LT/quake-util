mod repr;

mod parser;

pub use repr::{Entry, EntryOffset};

pub(crate) use repr::Head;

pub use parser::Parser;
