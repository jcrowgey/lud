#[macro_use]
extern crate clap;
use clap::{App, Arg, ArgMatches};

use lud::{message, resconf, send_query};

use std::process;

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("lud")
        .version(crate_version!())
        .about("DNS Lookup Client")
        .author("Joshua Crowgey")
        .arg(
            Arg::with_name("name")
                .help("what to look up")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("qtype")
                .short("q")
                .long("qtype")
                .help("what are you asking")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("which DNS server to use")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("raw")
                .short("r")
                .long("raw")
                .help("Print the raw reply, no parsing")
                .required(false)
                .takes_value(false),
        ).get_matches()
}

fn main() {
    let cli = parse_cli();

    let name = cli
        .value_of("name")
        .map(String::from)
        .expect("A name to lookup is required");
    let qtype = cli
        .value_of("qtype")
        .map(String::from)
        .unwrap_or("A".to_string());
    let resolver;
    match cli.value_of("server").map(String::from) {
        Some(server) => {
            resolver = server + ":53";
        }
        _ => {
            resolver = resconf::get_resolver().to_string() + ":53";
        }
    }
    let raw = cli.is_present("raw");
    let mut recv_buf = [0u8; message::DNS_MSG_MAX];
    let send_res = send_query(&mut recv_buf, name, qtype, resolver);
    let received = send_res.unwrap();
    if raw {
        let mut sep = "";
        for (i, b) in recv_buf[..received].iter().enumerate() {
            if i % 2 == 0 {
                print!("{}", sep);
                sep = " "
            }
            print!("{:02x}", b);
        }
        println!();
        process::exit(0);
    }

    let message = message::Message::from_wire(&recv_buf[..received]);
    println!("{}", message);
}
