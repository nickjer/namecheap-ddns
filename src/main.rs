#[macro_use]
extern crate clap;
extern crate quick_xml;
extern crate reqwest;
extern crate serde;

use clap::{App, Arg};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ip {
    #[serde(rename = "$value")]
    body: String,
}

#[derive(Debug, Deserialize)]
struct Error {
    #[serde(rename = "$value")]
    body: String,
}

#[derive(Debug, Deserialize)]
struct Errors {
    #[serde(rename = "Err1")]
    error: Error,
}

#[derive(Debug, Deserialize)]
struct InterfaceResponse {
    #[serde(rename = "IP")]
    ip: Option<Ip>,
    errors: Option<Errors>,
}

fn main() {
    let matches = App::new("namecheap-ddns")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Updates the A + Dynamic DNS records for Namecheap")
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .env("NAMECHEAP_DDNS_DOMAIN")
                .value_name("DOMAIN")
                .help("The domain with subdomains")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("subdomain")
                .short("s")
                .long("subdomain")
                .env("NAMECHEAP_DDNS_SUBDOMAIN")
                .value_name("SUBDOMAIN")
                .help("The subdomain to update")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .number_of_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .env("NAMECHEAP_DDNS_IP")
                .value_name("IP")
                .help(
                    "The ip address to set on the subdomains (if blank the ip \
                     used to make this request will be used)",
                )
                .takes_value(true),
        )
        .get_matches();

    let domain = matches.value_of("domain").unwrap();
    let subdomains = matches.values_of("subdomain").unwrap();
    let ip_option = matches.value_of("ip");

    let token_env = "NAMECHEAP_DDNS_TOKEN";
    let token = match std::env::var(token_env) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: {} for {}", e, token_env);
            std::process::exit(1);
        }
    };

    let client = Client::new();
    for subdomain in subdomains {
        let mut query = vec![
            ("domain", domain),
            ("host", subdomain),
            ("password", &token),
        ];
        if let Some(ip) = ip_option {
            query.push(("ip", ip));
        }

        let response = client
            .get("https://dynamicdns.park-your-domain.com/update")
            .query(&query)
            .send();
        let body = response.unwrap().text().unwrap();
        let parsed_body: InterfaceResponse = quick_xml::de::from_str(&body).unwrap();

        if let Some(errors) = parsed_body.errors {
            eprintln!("ERROR: {}", errors.error.body);
            std::process::exit(1);
        }

        println!(
            "{}.{} IP address updated to: {}",
            subdomain,
            domain,
            parsed_body.ip.unwrap().body
        );
    }
}
