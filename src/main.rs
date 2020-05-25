use std::env;
use std::str::FromStr;
use tokio::net::UdpSocket;
use tokio::task::spawn;

use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientStream;

use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
use serde::Deserialize;

// https://api.cloudflare.com/
const API_ENDPOINT: &str = "https://api.cloudflare.com/client/v4";

/// DNS lookup with CH Class and TXT RecordType from Cloudflare's DNS 1.1.1.1
/// and Name server `whoami.cloudflare`
///
/// Reference dig command:
/// ```
/// dig @1.1.1.1 -c CH -t TXT whoami.cloudflare +short
/// ```
async fn dig() -> Result<String, Box<dyn std::error::Error>> {
    // We need a connection, TCP and UDP are supported by DNS servers
    //   (tcp construction is slightly different as it needs a multiplexer)
    let stream = UdpClientStream::<UdpSocket>::new(([1, 1, 1, 1], 53).into());

    let (mut client, bg) = AsyncClient::connect(stream).await?;

    spawn(bg);

    let resp = client
        .query(
            Name::from_str("whoami.cloudflare.")?,
            DNSClass::CH,
            RecordType::TXT,
        )
        .await?;

    let record = match resp.answers().iter().next() {
        Some(record) => {
            if let RData::TXT(ref data) = *record.rdata() {
                let txt_data: Vec<_> = data
                    .txt_data()
                    .iter()
                    .map(|bytes| String::from_utf8_lossy(bytes.as_ref()).into_owned())
                    .collect();

                return Ok(txt_data.join(""));
            }
            "".to_string()
        }
        None => "".to_string(),
    };
    Ok(record)
}


#[derive(Deserialize, Debug)]
struct DNSRecord {
    id: String,
    #[serde(rename="type")]
    record_type: String,
    name: String,
    content: String,
    proxiable: bool,
    proxied: bool,
    ttl: u8,
    locked: bool,
    zone_id: String,
    zone_name: String,
    created_on: String,
    modified_on: String,
}

#[derive(Deserialize, Debug)]
struct RequestError {
    code: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct ListResponse {
    success: bool,
    result: Option<Vec<DNSRecord>>,
    errors: Vec<RequestError>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip_addr = dig().await?;
    println!("{:#?}", ip_addr);
    let matches = clap_app!(app =>
        (name: crate_name!())
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg API_TOKEN:
            --api_token +takes_value
            +required
            env("DDNS_API_TOKEN")
            "API Token generated from the User Profile 'API Tokens' page"
        )
        (@arg ZONE: --zone +takes_value +required env("DDNS_ZONE") "Zone Identifier")
        (@arg NAME: +required "DNS record name")
    )
    .get_matches();

    // Get the required parameters from cli
    let api_token = matches.value_of("API_TOKEN").unwrap();
    let zone = matches.value_of("ZONE").unwrap();
    let name = matches.value_of("NAME").unwrap();

    let dns_records_endpoint = format!("{}/zones/{}/dns_records", API_ENDPOINT, zone);

    let res = reqwest::Client::new()
        .get(dns_records_endpoint.as_str())
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .query(&[("name", name)])
        .send()
        .await?
        .json::<ListResponse>()
        .await?;

    println!("{:#?}", res);

    Ok(())
}
