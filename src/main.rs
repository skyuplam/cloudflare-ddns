extern crate trust_dns_resolver;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;


fn main() {
    // Construct a new Resolver with opendns resolvers
    let opendns_ns1 = NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(208,67,222,222)), 53),
        protocol: Protocol::Udp,
    };
    let opendns_ns2 = NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(208,67,220,220)), 53),
        protocol: Protocol::Udp,
    };
    let domain = None;
    let search = vec![];
    let name_servers = vec![opendns_ns1, opendns_ns2];
    let config = ResolverConfig::from_parts(
        domain,
        search,
        name_servers,
    );
    let resolver = Resolver::new(config, ResolverOpts::default()).unwrap();

    // Lookup the IP addresses associated with a name.
    // The final dot forces this to be an FQDN, otherwise the search rules as specified
    //  in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
    let response = resolver.lookup_ip("myip.opendns.com.").unwrap();

    // There can be many addresses associated with the name,
    //  this can return IPv4 and/or IPv6 addresses
    let address = response.iter().next().expect("no addresses returned!");

    // Print the output
    println!("Response: {}", address)
}
