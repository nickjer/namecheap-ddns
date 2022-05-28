extern crate clap;
extern crate minreq;
extern crate quick_xml;
extern crate url;

use clap::Parser;
use quick_xml::events::Event;
use quick_xml::Reader;
use url::Url;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The domain with subdomains
    #[clap(short, long, env = "NAMECHEAP_DDNS_DOMAIN")]
    domain: String,

    /// The subdomain to update
    #[clap(short, long, env = "NAMECHEAP_DDNS_SUBDOMAIN", required = true)]
    subdomain: Vec<String>,

    /// The ip address to set on the subdomains (if blank the ip used to make
    /// this request will be used)
    #[clap(short, long, env = "NAMECHEAP_DDNS_IP")]
    ip: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let token_env = "NAMECHEAP_DDNS_TOKEN";
    let token = std::env::var(token_env).unwrap_or_else(|error| {
        eprintln!("ERROR: {error} for {token_env}");
        std::process::exit(1);
    });

    let domain = cli.domain.clone();
    for subdomain in cli.subdomain {
        let mut url = Url::parse("https://dynamicdns.park-your-domain.com/update").unwrap();
        url.query_pairs_mut()
            .append_pair("domain", &domain)
            .append_pair("host", &subdomain)
            .append_pair("password", &token);
        if let Some(ref ip) = cli.ip {
            url.query_pairs_mut().append_pair("ip", &ip);
        }

        let response = minreq::get(url.as_str()).with_timeout(10).send().unwrap();
        let body = response.as_str().unwrap();

        let mut reader = Reader::from_str(&body);
        reader.trim_text(true);

        loop {
            match reader.read_event(&mut Vec::new()) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"IP" => {
                        let ip = reader.read_text(e.name(), &mut Vec::new()).unwrap();
                        println!("{subdomain}.{domain} IP address updated to: {ip}");
                    }
                    b"Err1" => {
                        let error = reader.read_text(e.name(), &mut Vec::new()).unwrap();
                        eprintln!("ERROR: {error}");
                        std::process::exit(1);
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
        }
    }
}
