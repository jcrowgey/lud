use std::fmt;
use crate::utils::{extract_name, byte_combine};

pub struct Question {
    qname: Vec<String>,
    qtype: u16,
    qclass: u16,
}

impl Question {
    pub fn from_wire(wire: &[u8], mut offset: usize) -> (Question, usize) {
        let (qname, loffset) = extract_name(wire, offset);
        offset = loffset;
        let qtype = byte_combine(wire[offset + 1], wire[offset]);
        offset += 2;
        let qclass = byte_combine(wire[offset + 1], wire[offset]);
        offset += 2;

        let q = Question { qname: qname,
                           qtype: qtype,
                           qclass: qclass, };
        (q, offset)
    }

    pub fn new(qname: Vec<String>, qtype: u16, qclass: u16) -> Question {
        Question{ qname: qname,
                  qtype: qtype,
                  qclass: qclass }
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
