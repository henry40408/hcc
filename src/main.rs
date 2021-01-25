use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use anyhow::anyhow;
use clap::{App, Arg};
use rustls::{ClientConfig, Session};
use x509_parser::parse_x509_certificate;
use x509_parser::time::ASN1Time;

const DOMAIN_NAME: &'static str = "domain_name";

fn main() -> anyhow::Result<()> {
    let matches = App::new("Potential-Giggle")
        .version("semantic-release")
        .author("Heng-Yi Wu <2316687+henry40408@users.noreply.github.com>")
        .about("Check expiration date of SSL certificate")
        .arg(Arg::with_name(DOMAIN_NAME).min_values(1))
        .get_matches();

    let domain_name = match matches.value_of(DOMAIN_NAME) {
        None => return Err(anyhow!("no domain name specific")),
        Some(d) => d,
    };
    let not_after = get_certificate(domain_name);
    println!("{:?}", not_after);
    Ok(())
}

fn build_config() -> Arc<ClientConfig> {
    let mut config = rustls::ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    Arc::new(config)
}

fn build_http_headers(domain_name: &str) -> String {
    format!(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: {0}\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        ),
        domain_name
    )
}

fn get_certificate(domain_name: &str) -> anyhow::Result<ASN1Time> {
    let config = build_config();

    let dns_name = webpki::DNSNameRef::try_from_ascii_str(domain_name)?;
    let mut sess = rustls::ClientSession::new(&config, dns_name);
    let mut sock = TcpStream::connect(format!("{0}:443", domain_name))?;
    let mut tls = rustls::Stream::new(&mut sess, &mut sock);
    tls.write(build_http_headers(domain_name).as_bytes())?;

    let certificates = match tls.sess.get_peer_certificates() {
        Some(certs) => certs,
        None => return Err(anyhow!("no certificates found for {0}", domain_name)),
    };
    let certificate = match certificates.last() {
        Some(cert) => cert,
        None => return Err(anyhow!("no certificate found for {0}", domain_name)),
    };
    let (_, parsed) = parse_x509_certificate(certificate.as_ref())?;
    Ok(parsed.validity().not_after)
}

#[cfg(test)]
mod test {
    use x509_parser::prelude::*;

    use crate::get_certificate;

    #[test]
    fn test_good_certificate() {
        let domain_name = "sha512.badssl.com";
        let not_after = get_certificate(domain_name).unwrap();
        assert!(not_after > ASN1Time::now());
    }

    #[test]
    fn test_bad_certificate() {
        let domain_name = "expired.badssl.com";
        let not_after = get_certificate(domain_name);
        assert!(not_after.is_err());
    }
}
