use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    LuaRuntimeError(String),
    InvalidFunction,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidFunction => write!(f, "Thing at the top is not a function."),
            Error::LuaRuntimeError(message) => write!(f, "{}", message),
        }
    }
}
