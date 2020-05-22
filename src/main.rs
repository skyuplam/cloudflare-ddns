use std::str::FromStr;
use tokio::net::UdpSocket;
use tokio::task::spawn;

use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientStream;

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
    Ok(())
}
