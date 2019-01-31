extern crate clap;
use clap::{Arg, App};

use lud;

fn main() {
    let matches = App::new("dlu")
                           .version("0.1.0")
                           .about("DNS Lookup Client")
                           .author("Joshua Crowgey")
                           .arg(Arg::with_name("server")
                                .short("s")
                                .long("server")
                                .help("which DNS server to use")
                                .required(false)
                                .takes_value(true))
                           .arg(Arg::with_name("name")
                                .short("n")
                                .long("name")
                                .help("what to look up")
                                .required(true)
                                .takes_value(true))
                           .get_matches();

    let server = matches.value_of("server").unwrap_or("9.9.9.9");
    let name = matches.value_of("name").unwrap();
    lud::run(server.to_string(),
              name.to_string());
}
