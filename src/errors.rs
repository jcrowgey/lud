use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct ParseError;
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid rrtype")
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "invalid rrtype"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

