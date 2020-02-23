use crate::rr::RRType;
use crate::utils::extract_name;
use std::error;
use std::{fmt, str};

pub struct AData {
    address: [u8; 4],
}

impl AData {
    pub fn from_wire(buf: &[u8], offset: usize) -> AData {
        // rdlength should always be 4 in this case
        AData {
            address: [
                buf[offset].to_owned(),
                buf[offset + 1].to_owned(),
                buf[offset + 2].to_owned(),
                buf[offset + 3].to_owned(),
            ],
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
    pub fn from_wire(buf: &[u8], offset: usize) -> Result<NSData, &dyn error::Error> {
        let (nsdname, _) = extract_name(buf, offset)?;
        Ok(NSData { nsdname: nsdname })
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
    pub fn from_wire(buf: &[u8], offset: usize) -> Result<CNAMEData, &dyn error::Error> {
        let (cname, _) = extract_name(buf, offset)?;
        Ok(CNAMEData { cname: cname })
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
    pub fn from_wire(buf: &[u8], offset: usize) -> Result<SOAData, &dyn error::Error> {
        let (mname, offset) = extract_name(buf, offset)?;
        let (rname, offset) = extract_name(buf, offset)?;
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
        Ok(SOAData {
            mname: mname,
            rname: rname,
            serial: serial,
            refresh: refresh,
            retry: retry,
            expire: expire,
        })
    }
}

impl fmt::Display for SOAData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}",
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
    pub fn from_wire(buf: &[u8], offset: usize) -> Result<MXData, &dyn error::Error> {
        let preference = (buf[offset] as u16) << 8 | (buf[offset + 1] as u16);
        let (exchange, _) = extract_name(buf, offset + 2)?;
        Ok(MXData {
            preference: preference,
            exchange: exchange,
        })
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
    pub fn from_wire(buf: &[u8], offset: usize) -> Result<PTRData, &dyn error::Error> {
        let (ptrdname, _) = extract_name(buf, offset)?;
        Ok(PTRData { ptrdname: ptrdname })
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
            txtdata: buf[offset..offset + rdlength].to_owned(),
        }
    }
}

impl fmt::Display for TXTData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX: TXT is not necessarily utf8, read the specs and fix
        write!(f, "{:?}", str::from_utf8(&self.txtdata).unwrap())
    }
}

pub struct AAAAData {
    address: [u8; 16],
}

impl AAAAData {
    pub fn from_wire(buf: &[u8], offset: usize) -> AAAAData {
        let mut addr = [0u8; 16];
        for (i, byte) in buf[offset..offset + 16].iter().enumerate() {
            addr[i] = byte.to_owned();
        }

        AAAAData { address: addr }
    }
}

impl fmt::Display for AAAAData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let quibbles: Vec<_> = self
            .address
            .chunks(2)
            .map(|q| (q[0] as u16) << 8 | q[1] as u16)
            .collect();

        let mut lo: usize = 0;
        let mut hi: usize = 0;
        let mut last_zero: Option<usize> = None;
        for (i, quibble) in quibbles.iter().enumerate() {
            if *quibble == 0 {
                // entering
                if last_zero == None {
                    last_zero = Some(i);
                }
            // staying in: noop
            } else {
                if let Some(index) = last_zero {
                    // exiting
                    if i - 1 - index > hi - lo {
                        // new best range
                        hi = i - 1;
                        lo = index;
                    }
                    last_zero = None;
                }
                // staying out: noop
            }
        }

        if let Some(index) = last_zero {
            // range may run to the end
            if quibbles.len() - 1 - index > hi - lo {
                // new best range
                hi = quibbles.len() - 1;
                lo = index;
            }
        }

        let mut fmt_str = String::new();
        let mut sep = "";

        if hi - lo > 0 {
            for quibble in quibbles[0..lo].iter() {
                fmt_str.push_str(sep);
                fmt_str.push_str(&format!("{:x}", quibble));
                sep = ":";
            }

            sep = "::";
            for quibble in quibbles[hi + 1..quibbles.len()].iter() {
                fmt_str.push_str(sep);
                fmt_str.push_str(&format!("{:x}", quibble));
                sep = ":";
            }
        } else {
            for quibble in quibbles.iter() {
                fmt_str.push_str(sep);
                fmt_str.push_str(&format!("{:x}", quibble));
                sep = ":";
            }
            fmt_str.push_str("\n");
        }
        write!(f, "{}", fmt_str)
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
    AAAA(AAAAData),
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
            RData::AAAA(aaaa_data) => aaaa_data.fmt(f),
            RData::UNKNOWN(rrtype) => write!(f, "{}", rrtype),
        }
    }
}

impl RData {
    pub fn from_wire(
        rrtype: RRType,
        buf: &[u8],
        offset: usize,
        rdlength: usize,
    ) -> Result<RData, &dyn error::Error> {
        match rrtype {
            RRType::A => Ok(RData::A(AData::from_wire(buf, offset))),
            RRType::NS => {
                let ns = NSData::from_wire(buf, offset)?;
                Ok(RData::NS(ns))
            }
            RRType::CNAME => {
                let cname = CNAMEData::from_wire(buf, offset)?;
                Ok(RData::CNAME(cname))
            }
            RRType::SOA => {
                let soa = SOAData::from_wire(buf, offset)?;
                Ok(RData::SOA(soa))
            }
            RRType::PTR => {
                let ptr = PTRData::from_wire(buf, offset)?;
                Ok(RData::PTR(ptr))
            }
            RRType::MX => {
                let mx = MXData::from_wire(buf, offset)?;
                Ok(RData::MX(mx))
            }
            RRType::TXT => Ok(RData::TXT(TXTData::from_wire(buf, offset, rdlength))),
            RRType::AAAA => Ok(RData::AAAA(AAAAData::from_wire(buf, offset))),
            _ => Ok(RData::UNKNOWN(rrtype as u16)),
        }
    }
}
