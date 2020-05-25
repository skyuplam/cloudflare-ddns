use std::env;
use std::str::FromStr;
use tokio::net::UdpSocket;
use tokio::task::spawn;

use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientStream;

use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip_addr = dig().await?;
    println!("{:#?}", ip_addr);
    let matches = clap_app!(app =>
        (name: crate_name!())
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg API_KEY: --api_key +takes_value +required env("DDNS_API_KEY") "API key generated on the 'My Account' page")
        (@arg EMAIL: --email +takes_value +required env("DDNS_EMAIL") "Email address associated with your account")
        (@arg USER_SERVICE_KEY:
            --service_key
            +takes_value
            +required
            env("DDNS_SERVICE_KEY")
            "A special Cloudflare API key good for a restricted set of endpoints. Always begins with 'v1.0-', may vary in length."
        )
        (@arg ZONE_ID: --zone +takes_value +required env("DDNS_ZONE") "Zone Identifier")
        (@arg NAME: +required "DNS record name")
    )
    .get_matches();

    let api_key = matches.value_of("API_KEY").unwrap();
    println!("{}", api_key);

    Ok(())
}
