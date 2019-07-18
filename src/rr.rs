use std::fmt;

use std::convert::TryFrom;
use crate::utils::{byte_combine, bytes_to_name_offset, extract_name};
use crate::errors::ParseError;
use crate::rdata;
use crate::rdata::RData;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum RRType {
    A = 1,     // a host address
    NS = 2,    // an authoritative name server
    MD = 3,    // a mail destination (Obsolete - use MX)
    MF = 4,    // a mail forwarder (Obsolete - use MX)
    CNAME = 5, // the canonical name for an alias
    SOA = 6,   // marks the start of a zone of authority
    MB = 7,    // a mailbox domain name (EXPERIMENTAL)
    MG = 8,    // a mail group member (EXPERIMENTAL)
    MR = 9,    // a mail rename domain name (EXPERIMENTAL)
    NULL = 10,  // a null RR (EXPERIMENTAL)
    WKS = 11,   // a well known service description
    PTR = 12,   // a domain name pointer
    HINFO = 13, // host information
    MINFO = 14, // mailbox or mail list information
    MX = 15,    // mail exchange
    TXT = 16,   // text strings
}

impl TryFrom<u16> for RRType {
    type Error = ParseError;
    fn try_from(original: u16) -> Result<Self, Self::Error> {
        match original {
            1 => Ok(RRType::A),
            2 => Ok(RRType::NS),
            3 => Ok(RRType::MD),
            4 => Ok(RRType::MF),
            5 => Ok(RRType::CNAME),
            6 => Ok(RRType::SOA),
            7 => Ok(RRType::MB),
            8 => Ok(RRType::MG),
            9 => Ok(RRType::MR),
            10 => Ok(RRType::NULL),
            11 => Ok(RRType::WKS),
            12 => Ok(RRType::PTR),
            13 => Ok(RRType::HINFO),
            14 => Ok(RRType::MINFO),
            15 => Ok(RRType::MX),
            16 => Ok(RRType::TXT),
            _ => Err(ParseError)
        }
    }
}

impl TryFrom<String> for RRType {
    type Error = ParseError;
    fn try_from(original: String) -> Result<Self, Self::Error> {
        match original.as_ref() {
             "A" => Ok(RRType::A),     // 1 a host address
             "NS" => Ok(RRType::NS),    // 2 an authoritative name server
             "MD" => Ok(RRType::MD),    // 3 a mail destination (Obsolete - use MX)
             "MF" => Ok(RRType::MF),    // 4 a mail forwarder (Obsolete - use MX)
             "CNAME" => Ok(RRType::CNAME), // 5 the canonical name for an alias
             "SOA" => Ok(RRType::SOA),   // 6 marks the start of a zone of authority
             "MB" => Ok(RRType::MB),    // 7 a mailbox domain name (EXPERIMENTAL)
             "MG" => Ok(RRType::MG),    // 8 a mail group member (EXPERIMENTAL)
             "MR" => Ok(RRType::MR),    // 9 a mail rename domain name (EXPERIMENTAL)
             "NULL" => Ok(RRType::NULL),  // 10 a null RR (EXPERIMENTAL)
             "WKS" => Ok(RRType::WKS),   // 11 a well known service description
             "PTR" => Ok(RRType::PTR),   // 12 a domain name pointer
             "HINFO" => Ok(RRType::HINFO), // 13 host information
             "MINFO" => Ok(RRType::MINFO), // 14 mailbox or mail list information
             "MX" => Ok(RRType::MX),    // 15 mail exchange
             "TXT" => Ok(RRType::TXT),   // 16 text strings
            _ => Err(ParseError),
        }
    }
}


#[derive(Debug, Clone)]
pub enum Class {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4
}


impl TryFrom<u16> for Class {
    type Error = ParseError;
    fn try_from(original: u16) -> Result<Self, Self::Error> {
        match original {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),
            _ => Err(ParseError),
        }
    }
}


fn extract_ttl(bytes: &[u8], offset: usize) -> i32 {
    (16 * 16 * (byte_combine(bytes[offset], bytes[offset + 1]) as i32)
        + (byte_combine(bytes[offset + 2], bytes[offset + 3]) as i32))
}

pub struct RR {
    name: Vec<String>,
    rrtype: RRType,
    class: Class,
    ttl: i32,
    rdlength: u16,
    rdata_parsed: RData,
}

impl RR {
    pub fn from_wire(buf: &[u8], mut offset: usize) -> (RR, usize) {
        let mut name = Vec::new();
        let name_type = buf[offset] >> 6;

        if name_type == 3 {
            let name_offset = bytes_to_name_offset(buf[offset], buf[offset + 1]);
            let (mut ref_name, _loffset) = extract_name(buf, name_offset as usize);
            name.append(&mut ref_name);
            offset += 2;
        } else if name_type == 0 {
            let (mut ref_name, new_offset) = extract_name(buf, offset);
            name.append(&mut ref_name);
            offset = new_offset;
        } else {
            panic!("Unimplemented name type: {:#b}", name_type);
        }

        // XXX: do we really want to unwrap here?!
        let rrtype = RRType::try_from(byte_combine(buf[offset], buf[offset + 1])).unwrap();
        offset += 2;
        let class = Class::try_from(byte_combine(buf[offset], buf[offset + 1])).unwrap();
        offset += 2;
        let ttl = extract_ttl(buf, offset);
        offset += 4;

        let rdlength = byte_combine(buf[offset], buf[offset + 1]) as usize;
        offset += 2;

        let rdata_parsed = rdata::from_wire(rrtype, buf, offset, rdlength);
        let rr = RR {
            name: name,
            rrtype: rrtype,
            class: class,
            ttl: ttl,
            rdlength: rdlength as u16,
            rdata_parsed: rdata_parsed,
        };

        offset += rdlength;
        (rr, offset)
    }
}

impl fmt::Display for RR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rdata_fmt = self.rdata_parsed.to_string();

        write!(
            f,
            "{}\t{:?}\t{:?}\tTTL: {:?}, RDLEN: {:?}\n{}",
            self.name.join("."),
            self.rrtype,
            self.class,
            self.ttl,
            self.rdlength,
            rdata_fmt
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrtype_u16_round_trip() {
        let cname: u16 = 5;
        let rrt = RRType::try_from(cname).unwrap();
        let _u16 = rrt as u16;
        assert_eq!(_u16, cname);
    }
}
