use std::net::Ipv4Addr;

extern crate ipconfig;

pub fn get_resolver() -> Ipv4Addr {
    let adapters = ipconfig::get_adapters().expect("Can't find adapters");
    let v4dns = adapters
        .iter()
        .flat_map(|adapter| adapter.dns_servers())
        .filter_map(|address| {
            match *address {
                std::net::IpAddr::V4(address) => Some(address),
                _ => None
            }
        })
        .next().expect("Can't find V4 dns address");
    
    v4dns
}
