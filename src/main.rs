extern crate clap;
extern crate minreq;
extern crate quick_xml;
extern crate url;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use quick_xml::de::from_str;
use serde::Deserialize;
use url::Url;

const API_URL: &str = "https://dynamicdns.park-your-domain.com/update";

#[derive(Debug, Deserialize)]
struct ErrorList {
    #[serde(rename = "$value", default)]
    errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "IP")]
    ip: Option<String>,

    #[serde(rename = "ErrCount")]
    err_count: u8,

    #[serde(rename = "errors")]
    error_list: ErrorList,
}

impl Response {
    fn success(&self) -> bool {
        self.err_count == 0
    }

    fn error(&self) -> Option<String> {
        self.error_list.errors.first().cloned()
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The domain with subdomains
    #[clap(short, long, env = "NAMECHEAP_DDNS_DOMAIN")]
    domain: String,

    /// The subdomain to update
    #[clap(
        short,
        long,
        env = "NAMECHEAP_DDNS_SUBDOMAIN",
        required = true,
        use_value_delimiter = true
    )]
    subdomain: Vec<String>,

    /// The ip address to set on the subdomains (if blank the ip used to make
    /// this request will be used)
    #[clap(short, long, env = "NAMECHEAP_DDNS_IP")]
    ip: Option<String>,

    /// The secret token
    #[clap(short, long, env = "NAMECHEAP_DDNS_TOKEN")]
    token: String,
}

fn update(domain: &str, subdomain: &str, token: &str, ip: Option<&str>) -> Result<()> {
    let mut url = Url::parse(API_URL)?;
    url.query_pairs_mut()
        .append_pair("domain", domain)
        .append_pair("host", subdomain)
        .append_pair("password", token);
    if let Some(ip) = ip {
        url.query_pairs_mut().append_pair("ip", ip);
    }

    let response = minreq::get(url.as_str())
        .with_timeout(10)
        .send()
        .with_context(|| format!("Failed to connect to {API_URL}"))?;
    let body: Response = from_str(response.as_str()?)?;

    if body.success() {
        match body.ip {
            Some(ip) => {
                println!("{subdomain}.{domain} IP address updated to: {ip}");
                Ok(())
            }
            None => Err(anyhow!("Missing IP address in response")),
        }
    } else {
        match body.error() {
            Some(error) => Err(anyhow!("{error}")),
            None => Err(anyhow!("Failed with unknown error")),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let domain = cli.domain.clone();
    for subdomain in cli.subdomain {
        update(&domain, &subdomain, &cli.token, cli.ip.as_deref())
            .with_context(|| format!("Failed to update {subdomain}.{domain}"))?;
    }

    Ok(())
}
