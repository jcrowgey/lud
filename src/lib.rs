extern crate byteorder;
extern crate idna;
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

pub fn parse_name(mut name: String) -> Vec<String> {
    // XXX: name really needs to be bytes
    if !name.ends_with(".") {
        name.push('.');
    }

    let mut labels: Vec<String> = name
        .split(".")
        .map(|s| {
            match idna::domain_to_ascii(&s.to_string()) {
                Ok(l) => l,
                Err(e) => {
                    println!("error parsing label {}: {:?}", s, e);
                    // default to input label if it can't be parsed
                    s.to_string()
                }
            }
        })
        .collect();

    if labels[0] == "" {
        // this is query against the root
        labels.pop();
    }

    return labels;
}

pub fn send_query(
    mut recv_buf: &mut [u8],
    name: String,
    qtype: String,
    resolver: String,
) -> io::Result<usize> {
    let labels = parse_name(name);
    let q_message = Message::new(labels, qtype);
    let buf = q_message.to_wire();

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");

    sock.send_to(&buf, resolver).expect("Failed to send");
    sock.recv(&mut recv_buf)
}
