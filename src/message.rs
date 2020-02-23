use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error;
use std::fmt;

use rand::random;

use crate::question::{QType, Question};
use crate::rr::RR;

pub const DNS_MSG_MAX: usize = 512;

#[derive(Debug, Clone, Copy)]
pub enum RCode {
    NoError = 0,
    FormatError = 1,
    ServFail = 2,
    NameError = 3,
    NotImp = 4,
    Refused = 5,
    Reserved = 6,
}

impl From<u16> for RCode {
    fn from(original: u16) -> RCode {
        match original {
            0 => RCode::NoError,
            1 => RCode::FormatError,
            2 => RCode::ServFail,
            3 => RCode::NameError,
            4 => RCode::NotImp,
            5 => RCode::Refused,
            _ => RCode::Reserved, // reserved is 6 through 15 really
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum QR {
    Q,
    R,
}

impl From<bool> for QR {
    fn from(original: bool) -> QR {
        match original {
            false => QR::Q,
            true => QR::R,
        }
    }
}

pub struct MessageMeta {
    pub qr: QR,
    pub opcode: u8,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: u8,
    pub rcode: RCode,
}

impl MessageMeta {
    fn new(meta: u16) -> MessageMeta {
        MessageMeta {
            qr: QR::from((meta >> 15) != 0),
            opcode: ((meta >> 11) & 0b1111) as u8,
            aa: ((meta >> 10) & 0b1) != 0,
            tc: ((meta >> 9) & 0b1) != 0,
            rd: ((meta >> 8) & 0b1) != 0,
            ra: ((meta >> 7) & 0b1) != 0,
            z: ((meta >> 4) & 0b111) as u8,
            rcode: RCode::from(meta & 0b1111),
        }
    }

    /* |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   | */
    fn to_wire(&self) -> Vec<u8> {
        let mut byte_1 = (*&self.qr as u8) << 7;
        byte_1 += self.opcode << 3;
        byte_1 += (self.aa as u8) << 2;
        byte_1 += (self.tc as u8) << 1;
        byte_1 += self.rd as u8;

        let mut byte_2 = (*&self.ra as u8) << 7;
        byte_2 += (self.z as u8) << 2;
        byte_2 += *&self.rcode as u8;

        vec![byte_1, byte_2]
    }
}

impl fmt::Display for MessageMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "QR: {:?}; Opcode: {:?}\nFLAGS: AA {:?}; TC {:?}; RD: {:?}; RA: {:?}; Z: {:?}; {:?}",
            self.qr, self.opcode, self.aa, self.tc, self.rd, self.ra, self.z, self.rcode
        )
    }
}

pub struct Message {
    pub id: u16,
    pub meta: MessageMeta,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
    pub question: Vec<Question>,
    pub answer: Vec<RR>,
    pub authority: Vec<RR>,
    pub additional: Vec<RR>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut to_write = format!(
            "ID: {}\n{}\nQDCOUNT {}; ANCOUNT {}; NSCOUNT {}; ARCOUNT {}",
            self.id, self.meta, self.qdcount, self.ancount, self.nscount, self.arcount
        );

        if self.qdcount > 0 {
            let mut fmt_questions = Vec::new();
            for q in self.question.iter() {
                fmt_questions.push(q.to_string());
            }
            to_write.push_str(&format!("\n\nQuestion\n{}", fmt_questions.join("\n")));
        }

        for section in [
            (self.ancount, &self.answer, "Answer"),
            (self.nscount, &self.authority, "Authority"),
            (self.arcount, &self.additional, "Additional"),
        ]
        .iter()
        {
            if section.0 > 0 {
                let mut fmt_section = Vec::new();
                for item in section.1.iter() {
                    fmt_section.push(item.to_string());
                }
                to_write.push_str(&format!("\n\n{}\n{}", section.2, fmt_section.join("\n")));
            }
        }

        write!(f, "{}", to_write)
    }
}

impl Message {
    pub fn from_wire(buf: &[u8]) -> Result<Message, &dyn error::Error> {
        let mut m_reply: &[u8] = &buf[..buf.len()];
        let mut message = Message {
            id: m_reply.read_u16::<BigEndian>().unwrap(),
            meta: MessageMeta::new(m_reply.read_u16::<BigEndian>().unwrap()),
            qdcount: m_reply.read_u16::<BigEndian>().unwrap(),
            ancount: m_reply.read_u16::<BigEndian>().unwrap(),
            nscount: m_reply.read_u16::<BigEndian>().unwrap(),
            arcount: m_reply.read_u16::<BigEndian>().unwrap(),
            question: Vec::new(),
            answer: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        };

        let (question, offset) = extract_questions(&buf, 12, message.qdcount)?;
        let (answer, offset) = extract_rrset(&buf, offset, message.ancount)?;
        let (authority, offset) = extract_rrset(&buf, offset, message.nscount)?;
        let (additional, _offset) = extract_rrset(&buf, offset, message.arcount)?;

        message.question = question;
        message.answer = answer;
        message.authority = authority;
        message.additional = additional;
        Ok(message)
    }

    pub fn to_wire(&self) -> Vec<u8> {
        let mut wire = Vec::new();
        wire.push((self.id >> 8) as u8);
        wire.push((self.id & 255) as u8);
        let meta_wire = self.meta.to_wire();
        wire.push(meta_wire[0]);
        wire.push(meta_wire[1]);

        for f in vec![self.qdcount, self.ancount, self.nscount, self.arcount] {
            wire.push((f >> 8) as u8);
            wire.push(f as u8);
        }

        let mut offset_map = HashMap::new();
        for q in self.question.iter() {
            for index in 0..q.qname.len() {
                let name = q.qname[index..].join(".");
                match offset_map.get(&name) {
                    Some(offset) => {
                        let flagged_offset = 0b11000000_00000000 + offset;
                        wire.push((flagged_offset >> 8) as u8);
                        wire.push((flagged_offset & 255) as u8);
                        let qtype_wire: u16 = u16::from(q.qtype);
                        wire.push((qtype_wire >> 8) as u8);
                        wire.push((qtype_wire & 255) as u8);
                    }
                    None => {
                        offset_map.insert(name, wire.len());
                        wire.push(q.qname[index].len() as u8);
                        for byte in q.qname[index].bytes() {
                            wire.push(byte);
                        }
                    }
                }
            }
            let qtype_wire: u16 = u16::from(q.qtype);
            wire.push((qtype_wire >> 8) as u8);
            wire.push((qtype_wire & 255) as u8);
            wire.push((q.qclass >> 8) as u8);
            wire.push((q.qclass & 255) as u8);
        }

        wire
    }

    pub fn new(name: Vec<String>, qtype: String) -> Message {
        Message {
            id: random::<u16>(),
            meta: MessageMeta::new(0x0100), // question with RD flag
            qdcount: 0x0001,
            ancount: 0x0000,
            nscount: 0x0000,
            arcount: 0x0000,
            // basic question type, internet class
            question: vec![Question::new(
                name,
                QType::try_from(qtype).expect("unable to parse qtype"),
                0x0001,
            )],
            answer: Vec::<RR>::new(),
            authority: Vec::<RR>::new(),
            additional: Vec::<RR>::new(),
        }
    }
}

fn extract_rrset(
    buf: &[u8],
    offset: usize,
    rrcount: u16,
) -> Result<(Vec<RR>, usize), &dyn error::Error> {
    let mut idx = offset;
    let mut processed_rrs = 0;
    let mut rrset = Vec::new();

    while processed_rrs < rrcount {
        let (rr, l_idx) = RR::from_wire(buf, idx)?;
        rrset.push(rr);
        idx = l_idx;
        processed_rrs += 1;
    }
    Ok((rrset, idx)) // index of next section
}

fn extract_questions(
    reply: &[u8],
    mut offset: usize,
    qdcount: u16,
) -> Result<(Vec<Question>, usize), &dyn error::Error> {
    let mut questions_processed = 0;
    let mut questions: Vec<Question> = Vec::new();
    while questions_processed < qdcount {
        let (question, l_offset) = Question::from_wire(reply, offset)?;
        offset = l_offset;
        questions.push(question);
        questions_processed += 1;
    }
    Ok((questions, offset)) // offset is index of next section
}

#[cfg(test)]
mod tests {
    use super::*;

    static META_STD_RD_QUERY: u16 = 0b0000_0001_0000_0000;

    #[test]
    fn messagemeta_new() {
        /* a standard query with recursion desired */
        let mm = MessageMeta::new(META_STD_RD_QUERY);
        assert!(mm.rd);
    }

    #[test]
    fn messagemeta_round_trip() {
        let mm = MessageMeta::new(META_STD_RD_QUERY);
        let expected_wire = vec![
            (META_STD_RD_QUERY >> 8) as u8,
            (META_STD_RD_QUERY & 255) as u8,
        ];
        assert_eq!(mm.to_wire(), expected_wire);
    }
}
