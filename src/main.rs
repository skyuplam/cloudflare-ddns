#[macro_use] extern crate log;
extern crate fern;
extern crate syslog;
extern crate trust_dns_resolver;
#[macro_use] extern crate clap;
extern crate futures;
#[macro_use] extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use] extern crate serde_json;

use std::io;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use futures::{Future, Stream};
use hyper::{Client, Request, Method};
use hyper::header::{ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use tokio_core::reactor::Core;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;


header! { (XAuthEmail, "X-Auth-Email") => [String] }
header! { (XAuthKey, "X-Auth-Key") => [String] }

// Setup stdio and syslog logging
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .chain(
            // Console config
            fern::Dispatch::new()
                .level(log::LevelFilter::Debug)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "[{}] {}",
                        record.level(),
                        message,
                    ))
                })
                .chain(std::io::stdout())
        ).chain(
            // Syslog config
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .level_for("syslog", log::LevelFilter::Debug)
                .chain(syslog::unix(syslog::Facility::LOG_USER)?)
        ).apply()?;
    Ok(())
}

// Get IPv4 address by DNS lookup
fn dig_ip() -> IpAddr {
    // Construct a new Resolver with opendns resolvers
    let opendns_ns1 = NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(208,67,222,222)), 53),
        protocol: Protocol::Udp, };
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

#[derive(Deserialize, Debug)]
struct Config {
    email: String,
    key: String,
    zoneid: String,
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
    let file = File::open(path)?;

    let config = serde_json::from_reader(file)?;

    Ok(config)
}

fn main() {
    // Setup Logging
    setup_logger().unwrap();

    // Parse CLI arg opts
    let matches = clap_app!(ddns =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Dynamically update DNS ip address")
        (@arg CONFIG: -c --config +takes_value +required "Config file location in JSON format")
        // (@arg EMAIL: -e --email +takes_value +required "Auth Email")
        // (@arg KEY: -k --key +takes_value +required "Auth Key")
        // (@arg ZONE_ID: -z --zoon_id +takes_value +required "Zone ID")
        (@subcommand list =>
            (about: "List all DNS records in the Zone")
            (@arg NAME: "DNS record name, e.g. example.com")
        )
        (@subcommand get =>
            (about: "Get the DNS record with provided ID")
            (@arg RECORD_ID: +required "DNS record ID")
        )
        (@subcommand update =>
            (about: "Update the DNS record with provided ID")
            (@arg NAME: +required "DNS record name, e.g. example.com")
            (@arg CONTENT: -c --content +takes_value "DNS record content, e.g. 127.0.0.1. Use provided ip address ad the record content instead of looking up the client IP from DNS")
            (@arg PROXIED: -p --proxied "Whether the record is proxied to cloudflare")
        )
    ).get_matches();

    let config_path = matches.value_of("CONFIG").unwrap();
    let config: Config = read_config_from_file(config_path).unwrap();

    let email = config.email;
    let key = config.key;
    let api_endpoint = format!("https://api.cloudflare.com/client/v4/zones/{}", config.zoneid);

    // create a tokio event loop
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    match matches.subcommand() {
        // Handle list subcommand
        ("list", Some(list)) => {
            let name = list.value_of("NAME").unwrap_or("");
            let params = if name.is_empty() {
                format!("")
            } else {
                format!("?name={}", name)
            };
            let uri = format!("{}/dns_records{}", api_endpoint, params);
            // Make a GET request
            let mut get_req = Request::new(Method::Get, uri.parse().unwrap());
            get_req.headers_mut().set(XAuthEmail(email.to_owned()));
            get_req.headers_mut().set(XAuthKey(key.to_owned()));

            let get = client.request(get_req).and_then(|res| {
                info!("List Response: {}", res.status());

                res.body().concat2().and_then(move |body| {
                    let v: Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    if let Some(success) = v["success"].as_bool() {
                        if !success {
                            error!("Errors {}", v["errors"]);
                        }
                        info!("Result {}", v["result"]);
                    }

                    Ok(())
                })
            });
            core.run(get).unwrap();
        },
        // Handle get subcommand
        ("get", Some(get)) => {
            let record_id = get.value_of("RECORD_ID").unwrap();
            let uri = format!("{}/dns_records/{}", api_endpoint, record_id);
            let mut get_req = Request::new(Method::Get, uri.parse().unwrap());
            get_req.headers_mut().set(XAuthEmail(email.to_owned()));
            get_req.headers_mut().set(XAuthKey(key.to_owned()));
            let get = client.request(get_req).and_then(|res| {
                info!("Get Response: {}", res.status());

                res.body().concat2().and_then(move |body| {
                    let v: Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    if let Some(success) = v["success"].as_bool() {
                        if !success {
                            error!("Errors {}", v["errors"]);
                        }
                        info!("Result {}", v["result"]);
                    }

                    Ok(())
                })
            });
            core.run(get).unwrap();
        },
        // Handle update subcommand
        ("update", Some(update)) => {
            let name = update.value_of("NAME").unwrap();
            let content = update.value_of("CONTENT").unwrap_or("");
            let uri = format!("{}/dns_records?name={}", api_endpoint, name);
            let mut get_req = Request::new(Method::Get, uri.parse().unwrap());
            get_req.headers_mut().set(XAuthEmail(email.to_owned()));
            get_req.headers_mut().set(XAuthKey(key.to_owned()));

            let get = client.request(get_req).and_then(|res| {
                res.body().concat2().and_then(move |body| {
                    let v: Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    let success = v["success"].as_bool().unwrap();
                    if !success {
                        panic!("Errors {}", v["errors"]);
                    }
                    let record_id = v["result"][0]["id"].as_str().unwrap();
                    let record_content = v["result"][0]["content"].as_str().unwrap();
                    Ok((record_id.to_string(), record_content.to_string()))
                })
            });
            let (record_id, record_content) = core.run(get).unwrap();
            let old_ip: IpAddr = record_content.parse().unwrap();
            let record_content = if content.is_empty() {
                dig_ip()
            } else {
                content.parse().unwrap()
            };
            if old_ip != record_content {
                let record = json!({
                    "type": "A",
                    "name": name,
                    "content": record_content,
                    "proxied": update.is_present("PROXIED"),
                });

                // Make a PUT request
                let uri = format!("{}/dns_records/{}", api_endpoint, record_id);
                let mut put_req = Request::new(Method::Put, uri.parse().unwrap());
                put_req.headers_mut().set(ContentType::json());
                put_req.headers_mut().set(XAuthEmail(email.to_owned()));
                put_req.headers_mut().set(XAuthKey(key.to_owned()));
                put_req.headers_mut().set(ContentLength(record.to_string().len() as u64));
                put_req.set_body(record.to_string());

                let put = client.request(put_req).and_then(|res| {
                    res.body().concat2().and_then(move |body| {
                        let v: Value = serde_json::from_slice(&body).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
                        })?;

                        if let Some(success) = v["success"].as_bool() {
                            if !success {
                                error!("Errors {}", v["errors"]);
                            } else {
                                info!("Result {}", v["result"]);
                            }
                        }

                        Ok(())
                    })
                });
                core.run(put).unwrap();
            } else {
                info!("Skipped, record is up to date!")
            }
        },
        ("", None) => {
            info!("Please provide at least one of the following subcommands: list, get or update.");
        },
        _ => unreachable!(),
    }
}
