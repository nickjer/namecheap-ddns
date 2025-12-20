use assert_cmd::cargo;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn success_response(ip: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-16"?>
        <interface-response>
            <Command>SETDNSHOST</Command>
            <Language>eng</Language>
            <IP>{ip}</IP>
            <ErrCount>0</ErrCount>
            <errors />
            <ResponseCount>0</ResponseCount>
            <responses />
            <Done>true</Done>
            <debug><![CDATA[]]></debug>
        </interface-response>"#
    )
}

fn error_response(error_message: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-16"?>
        <interface-response>
            <Command>SETDNSHOST</Command>
            <Language>eng</Language>
            <ErrCount>1</ErrCount>
            <errors>
                <Err1>{error_message}</Err1>
            </errors>
            <ResponseCount>1</ResponseCount>
            <responses>
                <response>
                    <Description>{error_message}</Description>
                    <ResponseNumber>304156</ResponseNumber>
                    <ResponseString>Validation error</ResponseString>
                </response>
            </responses>
            <Done>true</Done>
            <debug><![CDATA[]]></debug>
        </interface-response>"#
    )
}

fn multiple_errors_response() -> String {
    r#"<?xml version="1.0" encoding="utf-16"?>
        <interface-response>
            <Command>SETDNSHOST</Command>
            <Language>eng</Language>
            <ErrCount>2</ErrCount>
            <errors>
                <Err1>First error message</Err1>
                <Err2>Second error message</Err2>
            </errors>
            <ResponseCount>2</ResponseCount>
            <responses />
            <Done>true</Done>
            <debug><![CDATA[]]></debug>
        </interface-response>"#
        .to_string()
}

#[test]
fn missing_required_domain() {
    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args(["--subdomain", "test", "--token", "secret"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--domain"));
}

#[test]
fn missing_required_subdomain() {
    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args(["--domain", "example.com", "--token", "secret"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--subdomain"));
}

#[test]
fn missing_required_token() {
    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args(["--domain", "example.com", "--subdomain", "test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--token"));
}

#[test]
fn help_output() {
    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Updates the A + Dynamic DNS records",
        ));
}

#[test]
fn version_output() {
    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("namecheap-ddns"));
}

#[async_std::test]
async fn successful_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .and(query_param("domain", "example.com"))
        .and(query_param("host", "test"))
        .and(query_param("password", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_response("1.2.3.4")))
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args([
            "--api-url",
            &format!("{}/update", mock_server.uri()),
            "--domain",
            "example.com",
            "--subdomain",
            "test",
            "--token",
            "secret",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "test.example.com IP address updated to: 1.2.3.4",
        ));
}

#[async_std::test]
async fn successful_update_with_custom_ip() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .and(query_param("domain", "example.com"))
        .and(query_param("host", "test"))
        .and(query_param("password", "secret"))
        .and(query_param("ip", "5.6.7.8"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_response("5.6.7.8")))
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args([
            "--api-url",
            &format!("{}/update", mock_server.uri()),
            "--domain",
            "example.com",
            "--subdomain",
            "test",
            "--token",
            "secret",
            "--ip",
            "5.6.7.8",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "test.example.com IP address updated to: 5.6.7.8",
        ));
}

#[async_std::test]
async fn api_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(error_response("Passwords do not match")),
        )
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args([
            "--api-url",
            &format!("{}/update", mock_server.uri()),
            "--domain",
            "example.com",
            "--subdomain",
            "test",
            "--token",
            "wrong",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Passwords do not match"));
}

#[async_std::test]
async fn api_multiple_errors_returns_first() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .respond_with(ResponseTemplate::new(200).set_body_string(multiple_errors_response()))
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args([
            "--api-url",
            &format!("{}/update", mock_server.uri()),
            "--domain",
            "example.com",
            "--subdomain",
            "test",
            "--token",
            "token",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("First error message"));
}

#[async_std::test]
async fn multiple_subdomains() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .and(query_param("domain", "example.com"))
        .and(query_param("host", "www"))
        .and(query_param("password", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_response("1.2.3.4")))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .and(query_param("domain", "example.com"))
        .and(query_param("host", "api"))
        .and(query_param("password", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_response("1.2.3.4")))
        .expect(1)
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .args([
            "--api-url",
            &format!("{}/update", mock_server.uri()),
            "--domain",
            "example.com",
            "--subdomain",
            "www,api",
            "--token",
            "secret",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "www.example.com IP address updated to: 1.2.3.4",
        ))
        .stdout(predicate::str::contains(
            "api.example.com IP address updated to: 1.2.3.4",
        ));
}

#[async_std::test]
async fn env_var_configuration() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/update"))
        .and(query_param("domain", "env.example.com"))
        .and(query_param("host", "test"))
        .and(query_param("password", "env_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_response("1.2.3.4")))
        .mount(&mock_server)
        .await;

    Command::new(cargo::cargo_bin!("namecheap-ddns"))
        .env(
            "NAMECHEAP_DDNS_API_URL",
            format!("{}/update", mock_server.uri()),
        )
        .env("NAMECHEAP_DDNS_DOMAIN", "env.example.com")
        .env("NAMECHEAP_DDNS_SUBDOMAIN", "test")
        .env("NAMECHEAP_DDNS_TOKEN", "env_token")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "test.env.example.com IP address updated to: 1.2.3.4",
        ));
}
