#[macro_use]
extern crate clap;
extern crate trust_dns_resolver;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;


fn dig_ip() -> IpAddr {
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
    response.iter().next().expect("no addresses returned!")
}

fn main() {
    let matches = clap_app!(ddns =>
        (version: "1.0")
        (author: "Terrence Lam <skyuplam@gmail>")
        (about: "Dynamically update DNS ip address")
        (@arg EMAIL: -e --email +takes_value +required "Auth Email")
        (@arg KEY: -k --key +takes_value +required "Auth Key")
        (@arg TYPE: -t --type +takes_value +required "DNS record type, e.g. A")
        (@arg NAME: -n --name +takes_value +required "DNS record name, e.g. example.com")
        (@arg CONTENT: -c --content +takes_value +required "DNS record content, e.g. 127.0.0.1")
        (@arg API_ENDPOINT: +required "API Endpoint for PUT DNS record")
    ).get_matches();

    let email = matches.value_of("EMAIL").unwrap();
    let key = matches.value_of("KEY").unwrap();

    println!("{}, {}", email, key);

    let address = dig_ip();
    // Print the output
    println!("IP Address: {}", address)
}
