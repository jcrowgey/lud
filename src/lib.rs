extern crate byteorder;
extern crate rand;

use std::net::UdpSocket;

mod errors;
mod message;
mod question;
mod rdata;
mod rr;
mod utils;

use message::Message;

pub struct Config {
    pub name: String,
    pub qtype: String,
    pub server: String,
}

pub fn run(config: Config) {
    let mut name: Vec<String> = config.name.split(".").map(|s| s.to_string()).collect();
    name.push("".to_string());
    let q_message = Message::new(name, config.qtype);
    let buf = q_message.to_wire();

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");
    let resolver = config.server + ":53";
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
