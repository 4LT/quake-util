extern crate alloc;

use alloc::string::String;

pub type ValidationResult = Result<(), String>;

pub trait CheckWritable {
    fn check_writable(&self) -> ValidationResult;
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum WriteError {
    Validation(String),
    Io(std::io::Error),
}

#[cfg(feature = "std")]
pub type WriteAttempt = Result<(), WriteError>;
