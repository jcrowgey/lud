use byteorder::{BigEndian, ReadBytesExt};
use num::FromPrimitive;
use std::fmt;

use rand::random;

use crate::rr::RR;
use crate::question::Question;

pub const DNS_MSG_MAX: usize = 512;

// I think the numeric values are superfluous,
// because they're indexed from 0 in sequence, but not sure
enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum RCode {
    NOERROR = 0,
    FORMAT_ERROR = 1,
    SERVFAIL = 2,
    NAME_ERROR = 3,
    NOTIMP = 4,
    REFUSED = 5,
    RESERVED = 6, // reserved is 6 through 15 really
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum QR {
    Q = 0,
    R = 1
}
}

struct MessageMeta {
    qr: QR,
    opcode: u8,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    z: u8,
    rcode: RCode,
}

impl MessageMeta {
    fn new(meta: u16) -> MessageMeta {
        MessageMeta {
            qr: QR::from_u16(meta >> 15).unwrap(),
            opcode: ((meta >> 11) & 0b1111) as u8,
            aa: ((meta >> 10) & 0b1) != 0,
            tc: ((meta >> 9) & 0b1) != 0,
            rd: ((meta >> 8) & 0b1) != 0,
            ra: ((meta >> 7) & 0b1) != 0,
            z: ((meta >> 4) & 0b111) as u8,
            rcode: RCode::from_u16(meta & 0b1111).unwrap(),
        }
    }
}

impl fmt::Display for MessageMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "QR: {:?}; Opcode: {:?}\nFLAGS: AA {:?}; TC {:?} RD: {:?} RA: {:?} Z: {:?}; {:?}",
            self.qr, self.opcode, self.aa, self.tc, self.rd, self.ra, self.z, self.rcode
        )
    }
}

pub struct Message {
    id: u16,
    meta: MessageMeta,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
    question: Vec<Question>,
    answer: Vec<RR>,
    authority: Vec<RR>,
    additional: Vec<RR>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut to_write = String::new();
        to_write = format!(
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
    pub fn from_wire(buf: [u8; DNS_MSG_MAX], len: usize) -> Message {

        let mut m_reply: &[u8] = &buf[..len];
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

        let (question, offset) = extract_questions(&buf, 12, message.qdcount);
        let (answer, offset) = extract_rrset(&buf, offset, message.ancount);
        let (authority, offset) = extract_rrset(&buf, offset, message.nscount);
        let (additional, _offset) = extract_rrset(&buf, offset, message.arcount);

        message.question = question;
        message.answer = answer;
        message.authority = authority;
        message.additional = additional;
        message
    }

    pub fn new(name: Vec<String>) -> Message {
        Message {
            id: random::<u16>(),
            meta: MessageMeta::new(0x0100), // question with RD flag
            qdcount: 0x0001,
            ancount: 0x0000,
            nscount: 0x0000,
            arcount: 0x0000,
            question: vec![Question::new(name, 0x0001, 0x0001)], // basic question type, internet class
            answer: Vec::<RR>::new(),
            authority: Vec::<RR>::new(),
            additional: Vec::<RR>::new(),
        }
    }
}

fn extract_rrset(buf: &[u8], offset: usize, rrcount: u16) -> (Vec<RR>, usize) {
    let mut idx = offset;
    let mut processed_rrs = 0;
    let mut rrset = Vec::new();

    while processed_rrs < rrcount {
        let (rr, l_idx) = RR::from_wire(buf, idx);
        rrset.push(rr);
        idx = l_idx;
        processed_rrs += 1;
    }
    (rrset, idx) // index of next section
}

fn extract_questions(reply: &[u8], mut offset: usize, qdcount: u16) -> (Vec<Question>, usize) {
    let mut questions_processed = 0;
    let mut questions: Vec<Question> = Vec::new();
    while questions_processed < qdcount {
        let (question, l_offset) = Question::from_wire(reply, offset);
        offset = l_offset;
        questions.push(question);
        questions_processed += 1;
    }
    (questions, offset) // offset is index of next section
}
