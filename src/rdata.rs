use std::{fmt, str};
use crate::utils::extract_name;
use crate::rr::RRType;

pub struct AData {
    address: [u8; 4],
}

impl AData {
    pub fn from_wire(buf: &[u8], offset: usize) -> AData {
        // rdlength should always be 4 in this case
        AData {
          address: [buf[offset].to_owned(),
                    buf[offset + 1].to_owned(),
                    buf[offset + 2].to_owned(),
                    buf[offset + 3].to_owned()],
        }
    }
}

impl fmt::Display for AData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fmt_str = String::new();
        let mut sep = "";
        for byte in self.address.iter() {
            fmt_str.push_str(sep);
            fmt_str.push_str(&byte.to_string());
            sep = ".";
        }
        fmt_str.push_str("\n");
        write!(f, "{}", fmt_str)
    }
}

pub struct NSData {
    nsdname: Vec<String>,
}

impl NSData {
    pub fn from_wire(buf: &[u8], offset: usize) -> NSData {
        let (nsdname, _) = extract_name(buf, offset);
        NSData {
            nsdname: nsdname
        }
    }
}

impl fmt::Display for NSData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.nsdname.join("."))
    }
}

pub struct CNAMEData {
    cname: Vec<String>,
}

impl CNAMEData {
    pub fn from_wire(buf: &[u8], offset: usize) -> CNAMEData {
        let (cname, _) = extract_name(buf, offset);
        CNAMEData {
            cname: cname
        }
    }
}

impl fmt::Display for CNAMEData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cname.join("."))
    }
}


pub struct SOAData {
    mname: Vec<String>,
    rname: Vec<String>,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
}

impl SOAData {
    pub fn from_wire(buf: &[u8], offset: usize) -> SOAData {
        let (mname, offset) = extract_name(buf, offset);
        let (rname, offset) = extract_name(buf, offset);
        let serial = (buf[offset] as u32) << 24
                      | (buf[offset + 1] as u32) << 16
                      | (buf[offset + 2] as u32) << 8
                      | (buf[offset + 3] as u32);
        let offset = offset + 4;
        let refresh = (buf[offset] as u32) << 24
                      | (buf[offset + 1] as u32) << 16
                      | (buf[offset + 2] as u32) << 8
                      | (buf[offset + 3] as u32);
        let offset = offset + 4;
        let retry = (buf[offset] as u32) << 24
                      | (buf[offset + 1] as u32) << 16
                      | (buf[offset + 2] as u32) << 8
                      | (buf[offset + 3] as u32);
        let offset = offset + 4;
        let expire = (buf[offset] as u32) << 24
                      | (buf[offset + 1] as u32) << 16
                      | (buf[offset + 2] as u32) << 8
                      | (buf[offset + 3] as u32);
        SOAData {
            mname: mname,
            rname: rname,
            serial: serial,
            refresh: refresh,
            retry: retry,
            expire: expire,
        }
    }
}

impl fmt::Display for SOAData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{}\t{}\t{}\t{}\t{}\t{}",
            self.mname.join("."),
            self.rname.join("."),
            self.serial,
            self.refresh,
            self.retry,
            self.expire
        )
    }
}

pub struct MXData {
    preference: u16,
    exchange: Vec<String>,
}

impl MXData {
    pub fn from_wire(buf: &[u8], offset: usize) -> MXData {
        let preference = (buf[offset] as u16) << 8 | (buf[offset + 1] as u16);
        let (exchange, _) = extract_name(buf, offset + 2);
        MXData {
            preference: preference,
            exchange: exchange,
        }
    }
}

impl fmt::Display for MXData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.preference, self.exchange.join("."))
    }
}


pub struct PTRData {
    ptrdname: Vec<String>,
}

impl PTRData {
    pub fn from_wire(buf: &[u8], offset: usize) -> PTRData {
        let (ptrdname, _) = extract_name(buf, offset);
        PTRData {
            ptrdname: ptrdname
        }
    }
}

impl fmt::Display for PTRData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ptrdname.join("."))
    }
}

pub struct TXTData {
    txtdata: Vec<u8>, // Should actually be a Vec of <character-string> (cf RFC1035)
}

impl TXTData {
    pub fn from_wire(buf: &[u8], offset: usize, rdlength: usize) -> TXTData {
        TXTData {
            txtdata: buf[offset .. offset + rdlength].to_owned()
        }
    }
}

impl fmt::Display for TXTData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX: TXT is not necessarily utf8, read the specs and fix
        write!(f, "{:?}", str::from_utf8(&self.txtdata).unwrap())
    }
}

pub enum RData {
    A(AData),
    NS(NSData),
    CNAME(CNAMEData),
    SOA(SOAData),
    PTR(PTRData),
    MX(MXData),
    TXT(TXTData),
    UNKNOWN(u16),
}

impl fmt::Display for RData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RData::A(a_data) => a_data.fmt(f),
            RData::NS(ns_data) => ns_data.fmt(f),
            RData::CNAME(cname_data) => cname_data.fmt(f),
            RData::SOA(soa_data) => soa_data.fmt(f),
            RData::PTR(ptr_data) => ptr_data.fmt(f),
            RData::MX(mx_data) => mx_data.fmt(f),
            RData::TXT(txt_data) => txt_data.fmt(f),
            RData::UNKNOWN(rrtype) => write!(f, "{}", rrtype),
        }
    }
}

impl RData {
    pub fn from_wire(rrtype: RRType, buf: &[u8], offset: usize, rdlength: usize) -> RData {
        match rrtype {
            RRType::A => RData::A(AData::from_wire(buf, offset)),
            RRType::NS => RData::NS(NSData::from_wire(buf, offset)),
            RRType::CNAME => RData::CNAME(CNAMEData::from_wire(buf, offset)),
            RRType::SOA => RData::SOA(SOAData::from_wire(buf, offset)),
            RRType::PTR => RData::PTR(PTRData::from_wire(buf, offset)),
            RRType::MX => RData::MX(MXData::from_wire(buf, offset)),
            RRType::TXT => RData::TXT(TXTData::from_wire(buf, offset, rdlength)),
            _ => RData::UNKNOWN(rrtype as u16),
        }
    }
}
