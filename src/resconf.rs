use std::fs::File;
use std::io::Read;

use std::net::Ipv4Addr;

const RESOLVCONF_PATH: &str = "/etc/resolv.conf";

pub fn get_resolv_conf() -> resolv_conf::Config {
    // Read the file
    let mut buf = Vec::with_capacity(4096);
    let mut f = File::open(RESOLVCONF_PATH).unwrap();
    f.read_to_end(&mut buf).unwrap();

    // Parse the buffer
    resolv_conf::Config::parse(&buf).unwrap()
}

pub fn get_resolver() -> Ipv4Addr {
    let config = get_resolv_conf();
    match config.nameservers[0] {
        resolv_conf::ScopedIp::V4(ipv4) => ipv4,
        _ => panic!("oh no"),
    }
}
