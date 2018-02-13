extern crate trust_dns_resolver;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;


fn main() {
    // Construct a new Resolver with default configuration options
    let domain = None;
    let opendns_ns1 = NameServerConfig {
        socker_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(208,67,222,222)), 53),
        protocol: Protocol::Udp,
    };
    let opendns_ns2 = NameServerConfig {
        socker_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(208,67,220,220)), 53),
        protocol: Protocol::Udp,
    };
    let config = ResolverConfig::from_parts(
        domain,
    );
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    // Lookup the IP addresses associated with a name.
    // The final dot forces this to be an FQDN, otherwise the search rules as specified
    //  in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
    let response = resolver.lookup_ip("www.example.com.").unwrap();

    // There can be many addresses associated with the name,
    //  this can return IPv4 and/or IPv6 addresses
    let address = response.iter().next().expect("no addresses returned!");
}
