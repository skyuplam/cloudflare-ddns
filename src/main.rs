extern crate trust_dns_resolver;
#[macro_use] extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use] extern crate serde_json;

use std::io;
use futures::{Future, Stream};
use hyper::{Client, Request, Method};
use hyper::header::{ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use tokio_core::reactor::Core;
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
    // Parse cli arg opts
    let matches = clap_app!(ddns =>
        (version: "1.0")
        (author: "Terrence Lam <skyuplam@gmail>")
        (about: "Dynamically update DNS ip address")
        (@arg EMAIL: -e --email +takes_value +required "Auth Email")
        (@arg KEY: -k --key +takes_value +required "Auth Key")
        (@arg NAME: -n --name +takes_value +required "DNS record name, e.g. example.com")
        (@arg CONTENT: -c --content +takes_value "DNS record content, e.g. 127.0.0.1")
        (@arg PROXIED: -p --proxied "Whether the record is proxied to cloudflare")
        (@arg API_ENDPOINT: +required "API Endpoint for PUT DNS record")
    ).get_matches();

    let email = matches.value_of("EMAIL").unwrap();
    let key = matches.value_of("KEY").unwrap();
    let api_endpoint = matches.value_of("API_ENDPOINT").unwrap();
    let record_name = matches.value_of("NAME").unwrap();
    // let record_content = matches.value_of("CONTENT").unwrap_or("80.213.219.113");
    let record_proxied = matches.is_present("PROXIED");

    // create a tokio event loop
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    // dig IP address
    let address = dig_ip();

    let record = json!({
        "type": "A",
        "name": record_name,
        "content": address,
        "proxied": record_proxied,
    });

    // Make a PUT request
    let mut req = Request::new(Method::Put, api_endpoint.parse().unwrap());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set_raw("X-Auth-Email", email);
    req.headers_mut().set_raw("X-Auth-Key", key);
    req.headers_mut().set(ContentLength(record.to_string().len() as u64));
    req.set_body(record.to_string());

    let put = client.request(req).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;
            println!("Success {}, Message {}, Error {}, Address: {}", v["success"], v["messages"], v["errors"], address);
            Ok(())
        })
    });

    // Execute the request with tokio event loop
    core.run(put).unwrap();
}
