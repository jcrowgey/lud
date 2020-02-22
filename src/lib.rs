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
    labels
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_domain() {
        let ascii_domain = "google.com".to_string();
        let expected_labels = vec!["google".to_string(), "com".to_string(), "".to_string()];

        let parsed_labels = parse_name(ascii_domain);
        assert_eq!(expected_labels, parsed_labels);
    }

    #[test]
    fn test_valid_idn() {
        let unicode_domain = "са.com".to_string();
        let expected_labels = vec!["xn--80a7a".to_string(), "com".to_string(), "".to_string()];

        let parsed_labels = parse_name(unicode_domain);
        assert_eq!(expected_labels, parsed_labels);
    }

    #[test]
    fn test_valid_idn_tld() {
        let unicode_domain = "са.中國".to_string();
        let expected_labels = vec![
            "xn--80a7a".to_string(),
            "xn--fiqz9s".to_string(),
            "".to_string(),
        ];

        let parsed_labels = parse_name(unicode_domain);
        assert_eq!(expected_labels, parsed_labels);
    }

    #[test]
    fn test_punycode_domain() {
        let punycode_domain = "xn--80a7a.com".to_string();
        let expected_labels = vec!["xn--80a7a".to_string(), "com".to_string(), "".to_string()];

        let parsed_labels = parse_name(punycode_domain);
        assert_eq!(expected_labels, parsed_labels);
    }

    #[test]
    fn test_invalid_idn() {
        let invalid_unicode_domain = "xn--са.com".to_string();
        let expected_labels = vec!["xn--са".to_string(), "com".to_string(), "".to_string()];

        let parsed_labels = parse_name(invalid_unicode_domain);
        assert_eq!(expected_labels, parsed_labels);
    }
}
