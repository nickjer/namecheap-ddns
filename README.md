# Namecheap DDNS

[![Continuous Integration Status](https://github.com/nickjer/namecheap-ddns/workflows/Continuous%20integration/badge.svg)](https://github.com/nickjer/namecheap-ddns/actions)

A command line interface (CLI) used to update the A + Dynamic DNS records for
Namecheap.

## Installation

Install using [cargo]:

```shell
cargo install namecheap-ddns
```

## Usage

Check the help (`--help`) for details on using this tool:

```shell
namecheap-ddns 0.1.0
Jeremy Nicklas <jeremywnicklas@gmail.com>
Updates the A + Dynamic DNS records for Namecheap

USAGE:
    namecheap-ddns [OPTIONS] --domain <DOMAIN> --subdomain <SUBDOMAIN>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --domain <DOMAIN>             The domain with subdomains [env: NAMECHEAP_DDNS_DOMAIN=]
    -i, --ip <IP>                     The ip address to set on the subdomains (if blank the ip used to make this request
                                      will be used) [env: NAMECHEAP_DDNS_IP=]
    -s, --subdomain <SUBDOMAIN>...    The subdomain to update [env: NAMECHEAP_DDNS_SUBDOMAIN=]
```

You will need to specify Namecheap's Dynamic DNS Password provided to you in
their Advanced DNS control panel as the environment variable
`NAMECHEAP_DDNS_TOKEN`.

> *Tip:* This is not your Namecheap login password.

### Examples

I want to update the host `host1.example.com` with my current public facing ip
address:

```console
$ NAMECHEAP_DDNS_TOKEN=... namecheap-ddns -d example.com -s host1
host1.example.com IP address updated to: 123.123.123.123
```

I want to update multiple subdomains (`host1`, `host2`, and `host3`) with a
given ip address:

```console
$ NAMECHEAP_DDNS_TOKEN=... namecheap-ddns \
>     -d example.com \
>     -s host1 -s host2 -s host3
>     -i 123.123.123.123
host1.example.com IP address updated to: 123.123.123.123
host2.example.com IP address updated to: 123.123.123.123
host3.example.com IP address updated to: 123.123.123.123
```

I want to use an environment variable file:

```console
$ cat .env
export NAMECHEAP_DDNS_TOKEN=...
export NAMECHEAP_DDNS_DOMAIN=example.com
export NAMECHEAP_DDNS_SUBDOMAIN=host1
export NAMECHEAP_DDNS_IP=321.321.321.321
$ source .env
$ namecheap-ddns
host1.example.com IP address updated to: 321.321.321.321
```

[cargo]: https://doc.rust-lang.org/cargo/
