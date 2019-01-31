extern crate byteorder;
extern crate rand;
extern crate num;
#[macro_use]
extern crate enum_primitive;

use rand::random;
use std::net::UdpSocket;

mod rr;
mod utils;
mod question;

mod message;
use message::Message;

// XXX: this should really be based on to_wire methods from the structs
fn build_question_buf(name: String) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.push(random::<u8>());
    buf.push(random::<u8>()); // id
    buf.push(0x01);
    buf.push(0x00); // meta
    buf.push(0x00);
    buf.push(0x01); // qdcount
    buf.push(0x00);
    buf.push(0x00); // ancount
    buf.push(0x00);
    buf.push(0x00); // nscount
    buf.push(0x00);
    buf.push(0x00); // arcount

    let labels = name.split(".");
    for l in labels {
        buf.push(l.len() as u8);
        let l_bytes = l.as_bytes();
        for lb in l_bytes {
            buf.push(*lb);
        }
    }

    buf.push(0x00); // terminate label in q
    buf.push(0x00);
    buf.push(0x01); // qtype
    buf.push(0x00);
    buf.push(0x01); // qclass
    buf
}


pub fn run(server: String, name: String) {
    let buf = build_question_buf(name);

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");
    let resolver = server + ":53";
    sock.send_to(&buf, resolver).expect("Failed to send");
    let mut reply = [0u8; message::DNS_MSG_MAX];
    match sock.recv(&mut reply) {
        Ok(received) => {
            // XXX: this should really in an constructor or parse method for Message
            let message = Message::from_wire(reply, received);
            println!("{}", message);
        }
        Err(e) => println!("recv function failed: {:?}", e),
    }
}
