extern crate byteorder;
extern crate rand;
extern crate resolv_conf;

use std::net::{UdpSocket};

mod errors;
mod message;
mod question;
mod rdata;
mod rr;
mod utils;
mod resconf;

use message::Message;

pub struct Config {
    pub name: Option<String>,
    pub qtype: Option<String>,
    pub server: Option<String>,
}

pub fn run(config: Config) {
    let mut name: Vec<String> = config.name.unwrap().split(".").map(|s| s.to_string()).collect();
    name.push("".to_string());

    let q_message = Message::new(name, config.qtype.unwrap_or("A".to_string()));
    let buf = q_message.to_wire();


    let resolver;
    match config.server {
        Some(server) => {
            resolver = server + ":53";
        },
        _ => {
            resolver = resconf::get_resolver().to_string() + ":53";
        },
    }

    let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to this address");

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
