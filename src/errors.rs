use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidClass,
    InvalidRRType,
    PointerForward,
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::InvalidClass => "InvalidClass",
            ParseError::InvalidRRType => "InvalidRRType",
            ParseError::PointerForward => "PointerForward",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
