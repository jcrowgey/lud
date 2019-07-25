use crate::utils::{byte_combine, extract_name};
use std::convert::{AsRef, From, TryFrom};
use std::error;
use std::fmt;

// TODO: inherit, in some way, the RRTypes
#[derive(Debug, Clone, Copy)]
pub enum QType {
    A,     // 1 a host address
    NS,    // 2 an authoritative name server
    MD,    // 3 a mail destination (Obsolete - use MX)
    MF,    // 4 a mail forwarder (Obsolete - use MX)
    CNAME, // 5 the canonical name for an alias
    SOA,   // 6 marks the start of a zone of authority
    MB,    // 7 a mailbox domain name (EXPERIMENTAL)
    MG,    // 8 a mail group member (EXPERIMENTAL)
    MR,    // 9 a mail rename domain name (EXPERIMENTAL)
    NULL,  // 10 a null RR (EXPERIMENTAL)
    WKS,   // 11 a well known service description
    PTR,   // 12 a domain name pointer
    HINFO, // 13 host information
    MINFO, // 14 mailbox or mail list information
    MX,    // 15 mail exchange
    TXT,   // 16 text strings
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

impl From<QType> for u16 {
    fn from(original: QType) -> u16 {
        match original {
            QType::A => 1,
            QType::NS => 2,
            QType::MD => 3,
            QType::MF => 4,
            QType::CNAME => 5,
            QType::SOA => 6,
            QType::MB => 7,
            QType::MG => 8,
            QType::MR => 9,
            QType::NULL => 10,
            QType::WKS => 11,
            QType::PTR => 12,
            QType::HINFO => 13,
            QType::MINFO => 14,
            QType::MX => 15,
            QType::TXT => 16,
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
        match original.to_uppercase().as_ref() {
            "A" => Ok(QType::A),
            "NS" => Ok(QType::NS),
            "MD" => Ok(QType::MD),
            "MF" => Ok(QType::MF),
            "CNAME" => Ok(QType::CNAME),
            "SOA" => Ok(QType::SOA),
            "MB" => Ok(QType::MB),
            "MG" => Ok(QType::MG),
            "MR" => Ok(QType::MR),
            "NULL" => Ok(QType::NULL),
            "WKS" => Ok(QType::WKS),
            "PTR" => Ok(QType::PTR),
            "HINFO" => Ok(QType::HINFO),
            "MINFO" => Ok(QType::MINFO),
            "MX" => Ok(QType::MX),
            "TXT" => Ok(QType::TXT),
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
            3 => Ok(QType::MD),
            4 => Ok(QType::MF),
            5 => Ok(QType::CNAME),
            6 => Ok(QType::SOA),
            7 => Ok(QType::MB),
            8 => Ok(QType::MG),
            9 => Ok(QType::MR),
            10 => Ok(QType::NULL),
            11 => Ok(QType::WKS),
            12 => Ok(QType::PTR),
            13 => Ok(QType::HINFO),
            14 => Ok(QType::MINFO),
            15 => Ok(QType::MX),
            16 => Ok(QType::TXT),
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
            "{}\tQTYPE: {:?}; CLASS: {:?}",
            self.qname.join("."),
            self.qtype,
            self.qclass
        )
    }
}
