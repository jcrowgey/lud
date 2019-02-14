use crate::utils::{byte_combine, extract_name};
use std::convert::{From, AsRef};
use std::fmt;
use std::error;
use crate::tryfrom::TryFrom;

#[derive(Debug, Clone, Copy)]
pub enum QType {
   // TODO: inherit the RRTypes
   A,
   NS,
   AXFR,  //  252 A request for a transfer of an entire zone
   MAILB, //  253 A request for mailbox-related records (MB, MG or MR)
   MAILA, //  254 A request for mail agent RRs (Obsolete - see MX)
   ANY,   //  255 A request for all records
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

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl TryFrom<String> for QType {
    type Error = ParseError;
    fn try_from(original: String) -> Result<Self, Self::Error> {
        match original.as_ref() {
            "A" => Ok(QType::A),
            "NS" => Ok(QType::NS),
            "AXFR" => Ok(QType::AXFR),
            "MAILB" => Ok(QType::MAILB),
            "MAILA" => Ok(QType::MAILA),
            "ANY" => Ok(QType::ANY),
            _ => Err(ParseError),
        }
    }
}

impl TryFrom<u16> for QType {
    type Error = ParseError;
    fn try_from(original: u16) -> Result<Self, Self::Error> {
        match original {
            1 => Ok(QType::A),
            2 => Ok(QType::NS),
            252 => Ok(QType::AXFR),
            253 => Ok(QType::MAILB),
            254 => Ok(QType::MAILA),
            255 => Ok(QType::ANY),
            _ => Err(ParseError),
        }
    }
}

impl From<QType> for u16 {
    fn from(original: QType) -> u16 {
        match original {
            QType::A => 1,
            QType::NS => 2,
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::ANY => 255,
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
            "{}\tQTYPE: {:?}; CLASS: {:?}",
            self.qname.join("."),
            self.qtype,
            self.qclass
        )
    }
}
