use anyhow::{Context, Result, anyhow};
use clap::Parser;
use quick_xml::de::from_str;
use serde::Deserialize;
use url::Url;

const API_URL: &str = "https://dynamicdns.park-your-domain.com/update";
const REQUEST_TIMEOUT_SECS: u64 = 10;

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
    err_count: u32,

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
    /// The root domain (e.g., example.com)
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

    /// The API URL
    #[clap(long, env = "NAMECHEAP_DDNS_API_URL", default_value = API_URL)]
    api_url: String,
}

fn update(
    api_url: &str,
    domain: &str,
    subdomain: &str,
    token: &str,
    ip: Option<&str>,
) -> Result<()> {
    let mut url = Url::parse(api_url)?;
    url.query_pairs_mut()
        .append_pair("domain", domain)
        .append_pair("host", subdomain)
        .append_pair("password", token);
    if let Some(ip) = ip {
        url.query_pairs_mut().append_pair("ip", ip);
    }

    let response = minreq::get(url.as_str())
        .with_timeout(REQUEST_TIMEOUT_SECS)
        .send()
        .with_context(|| format!("Failed to connect to {api_url}"))?;
    let body: Response = from_str(response.as_str()?).context("Failed to parse API response")?;

    if body.success() {
        let ip = body
            .ip
            .ok_or_else(|| anyhow!("Missing IP address in response"))?;
        println!("{subdomain}.{domain} IP address updated to: {ip}");
        Ok(())
    } else {
        let error = body
            .error()
            .unwrap_or_else(|| "Failed with unknown error".to_string());
        Err(anyhow!("{error}"))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    for subdomain in &cli.subdomain {
        update(
            &cli.api_url,
            &cli.domain,
            subdomain,
            &cli.token,
            cli.ip.as_deref(),
        )
        .with_context(|| format!("Failed to update {subdomain}.{}", cli.domain))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_success_when_no_errors() {
        let response = Response {
            ip: Some("1.2.3.4".to_string()),
            err_count: 0,
            error_list: ErrorList { errors: vec![] },
        };
        assert!(response.success());
    }

    #[test]
    fn response_not_success_when_errors() {
        let response = Response {
            ip: None,
            err_count: 1,
            error_list: ErrorList {
                errors: vec!["Some error".to_string()],
            },
        };
        assert!(!response.success());
    }

    #[test]
    fn response_error_returns_first_error() {
        let response = Response {
            ip: None,
            err_count: 2,
            error_list: ErrorList {
                errors: vec!["First error".to_string(), "Second error".to_string()],
            },
        };
        assert_eq!(response.error(), Some("First error".to_string()));
    }

    #[test]
    fn response_error_returns_none_when_empty() {
        let response = Response {
            ip: None,
            err_count: 0,
            error_list: ErrorList { errors: vec![] },
        };
        assert_eq!(response.error(), None);
    }

    #[test]
    fn parse_success_response() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-16"?>
            <interface-response>
                <Command>SETDNSHOST</Command>
                <Language>eng</Language>
                <IP>1.2.3.4</IP>
                <ErrCount>0</ErrCount>
                <errors />
                <ResponseCount>0</ResponseCount>
                <responses />
                <Done>true</Done>
                <debug><![CDATA[]]></debug>
            </interface-response>
        "#;

        let response: Response = from_str(xml).unwrap();
        assert!(response.success());
        assert_eq!(response.ip, Some("1.2.3.4".to_string()));
        assert_eq!(response.error(), None);
    }

    #[test]
    fn parse_error_response() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-16"?>
            <interface-response>
                <Command>SETDNSHOST</Command>
                <Language>eng</Language>
                <ErrCount>1</ErrCount>
                <errors>
                    <Err1>Passwords do not match</Err1>
                </errors>
                <ResponseCount>1</ResponseCount>
                <responses>
                    <response>
                        <Description>Passwords do not match</Description>
                        <ResponseNumber>304156</ResponseNumber>
                        <ResponseString>Validation error; invalid ; password</ResponseString>
                    </response>
                </responses>
                <Done>true</Done>
                <debug><![CDATA[]]></debug>
            </interface-response>
        "#;

        let response: Response = from_str(xml).unwrap();
        assert!(!response.success());
        assert_eq!(response.ip, None);
        assert_eq!(response.error(), Some("Passwords do not match".to_string()));
    }
}
