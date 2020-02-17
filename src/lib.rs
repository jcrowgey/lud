extern crate byteorder;
extern crate rand;
extern crate resolv_conf;

use std::net::UdpSocket;

mod errors;
pub mod message;
mod question;
mod rdata;
pub mod resconf;
mod rr;
mod utils;

use message::Message;
use std::io;

pub fn send_query(mut recv_buf: &mut [u8], name: String, qtype: String, resolver: String) -> io::Result<usize> {
    let mut name: Vec<String> = name
        .split(".")
        .map(|s| s.to_string())
        .collect();
    name.push("".to_string());

    let q_message = Message::new(name, qtype);
    let buf = q_message.to_wire();

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");

    sock.send_to(&buf, resolver).expect("Failed to send");
    sock.recv(&mut recv_buf)
}
