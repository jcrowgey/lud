use crate::rr::RRType;
use crate::utils::{byte_combine, extract_name};
use std::convert::{AsRef, TryFrom};
use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum QType {
    RRType(RRType),
    AXFR,  //  252 A request for a transfer of an entire zone
    MAILB, //  253 A request for mailbox-related records (MB, MG or MR)
    MAILA, //  254 A request for mail agent RRs (Obsolete - see MX)
    ANY,   //  255 A request for all records
}

impl fmt::Display for QType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QType::RRType(rrt) => write!(f, "{:?}", rrt),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError;
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid qtype")
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "invalid qtype"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl From<QType> for u16 {
    fn from(original: QType) -> u16 {
        match original {
            QType::RRType(rrtype) => rrtype as u16,
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::ANY => 255,
        }
    }
}

impl TryFrom<String> for QType {
    type Error = ParseError;
    fn try_from(original: String) -> Result<Self, Self::Error> {
        match RRType::try_from(original.clone()) {
            Ok(rrt) => Ok(QType::RRType(rrt)),
            Err(_) => match original.to_uppercase().as_ref() {
                "AXFR" => Ok(QType::AXFR),
                "MAILB" => Ok(QType::MAILB),
                "MAILA" => Ok(QType::MAILA),
                "ANY" => Ok(QType::ANY),
                _ => Err(ParseError),
            },
        }
    }
}

impl TryFrom<u16> for QType {
    type Error = ParseError;
    fn try_from(original: u16) -> Result<Self, Self::Error> {
        match original {
            1..=251 => match RRType::try_from(original) {
                Ok(rrt) => Ok(QType::RRType(rrt)),
                Err(_) => Err(ParseError),
            },
            252 => Ok(QType::AXFR),
            253 => Ok(QType::MAILB),
            254 => Ok(QType::MAILA),
            255 => Ok(QType::ANY),
            _ => Err(ParseError),
        }
    }
}

pub struct Question {
    pub qname: Vec<String>,
    pub qtype: QType,
    pub qclass: u16,
}

impl Question {
    pub fn from_wire(wire: &[u8], mut offset: usize) -> (Question, usize) {
        let (qname, l_offset) = extract_name(wire, offset);
        offset = l_offset;
        let qtype = QType::try_from(byte_combine(wire[offset], wire[offset + 1])).unwrap();
        offset += 2;
        let qclass = byte_combine(wire[offset], wire[offset + 1]);
        offset += 2;

        let q = Question {
            qname: qname,
            qtype: qtype,
            qclass: qclass,
        };
        (q, offset)
    }

    pub fn new(qname: Vec<String>, qtype: QType, qclass: u16) -> Question {
        Question {
            qname: qname,
            qtype: qtype,
            qclass: qclass,
        }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\tQTYPE: {:}; CLASS: {:?}",
            self.qname.join("."),
            self.qtype,
            self.qclass
        )
    }
}
