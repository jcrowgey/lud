extern crate byteorder;
extern crate num;
extern crate rand;
#[macro_use]
extern crate enum_primitive;

use std::net::UdpSocket;

mod tryfrom;

mod question;
mod rr;
mod utils;
mod message;

use message::Message;

pub fn run(name: String, qtype: String, server: String) {
    let mut name: Vec<String> = name.split(".").map(|s| s.to_string()).collect();
    name.push("".to_string());
    let q_message = Message::new(name, qtype);
    let buf = q_message.to_wire();

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");
    let resolver = server + ":53";
    sock.send_to(&buf, resolver).expect("Failed to send");
    let mut reply = [0u8; message::DNS_MSG_MAX];
    match sock.recv(&mut reply) {
        Ok(received) => {
            let message = Message::from_wire(reply, received);
            println!("{}", message);
        }
        Err(e) => println!("recv function failed: {:?}", e),
    }
}
