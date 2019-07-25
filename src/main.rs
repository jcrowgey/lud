#[macro_use]
extern crate clap;
use clap::{App, Arg};

use lud::{Config, run};

fn main() {
    let matches = App::new("lud")
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
        .get_matches();

    let config = Config {
        name: matches.value_of("name").unwrap().to_string(),
        server: matches.value_of("server").unwrap_or("9.9.9.9").to_string(),
        qtype: matches.value_of("qtype").unwrap_or("A").to_string(),
    };
    run(config);
}
