use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use anyhow::Context;
use chrono::{SubsecRound, TimeZone, Utc};
use crossbeam_utils::thread;
use rustls::{ClientConfig, Session};
use x509_parser::parse_x509_certificate;

use crate::check_result::CheckResult;

pub struct CheckClient {
    config: Arc<ClientConfig>,
}

impl CheckClient {
    pub fn new() -> Self {
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Self {
            config: Arc::new(config),
        }
    }

    pub fn check_certificate(&self, domain_name: &str) -> anyhow::Result<CheckResult> {
        let checked_at = Utc::now().round_subsecs(0);

        let dns_name = webpki::DNSNameRef::try_from_ascii_str(domain_name)?;
        let mut sess = rustls::ClientSession::new(&self.config, dns_name);
        let mut sock = TcpStream::connect(format!("{0}:443", domain_name))?;
        let mut tls = rustls::Stream::new(&mut sess, &mut sock);

        match tls.write(Self::build_http_headers(domain_name).as_bytes()) {
            Ok(_) => (),
            Err(_) => return Ok(CheckResult::new(domain_name, checked_at)),
        };

        let certificates = tls
            .sess
            .get_peer_certificates()
            .with_context(|| format!("no peer certificates found for {0}", domain_name))?;

        let certificate = certificates
            .last()
            .with_context(|| format!("no certificate found for {0}", domain_name))?;

        let not_after = match parse_x509_certificate(certificate.as_ref()) {
            Ok((_, cert)) => cert.validity().not_after,
            Err(_) => return Ok(CheckResult::new(domain_name, checked_at)),
        };
        let not_after = Utc.timestamp(not_after.timestamp(), 0);

        let duration = not_after - checked_at;
        Ok(CheckResult {
            ok: true,
            checked_at,
            days: duration.num_days(),
            domain_name: domain_name.to_string(),
            expired_at: not_after.to_rfc3339(),
            not_after,
        })
    }

    pub fn check_certificates<S: AsRef<str>>(
        domain_names: &[S],
    ) -> anyhow::Result<Vec<CheckResult>> {
        let client = Arc::new(CheckClient::new());

        thread::scope(|s| {
            let mut handles = vec![];
            for domain_name in domain_names {
                let domain_name = domain_name.as_ref();
                let client = client.clone();
                handles.push(s.spawn(move |_| client.check_certificate(&domain_name)));
            }

            let mut results = vec![];
            for handle in handles {
                results.push(handle.join().unwrap()?);
            }

            Ok(results)
        })
        .unwrap()
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
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, TimeZone, Utc};

    use crate::check_client::CheckClient;

    fn checked_at_is_positive(checked_at: &DateTime<Utc>) -> bool {
        checked_at.timestamp() > 0
    }

    #[test]
    fn test_good_certificate() {
        let now = Utc.timestamp(0, 0);
        let domain_name = "sha512.badssl.com";

        let client = CheckClient::new();
        let resp = client.check_certificate(domain_name).unwrap();
        assert!(resp.ok);
        assert!(checked_at_is_positive(&resp.checked_at));
        assert!(now < resp.not_after);
    }

    #[test]
    fn test_bad_certificate() {
        let domain_name = "expired.badssl.com";

        let client = CheckClient::new();
        let resp = client.check_certificate(domain_name).unwrap();
        assert!(!resp.ok);
        assert!(checked_at_is_positive(&resp.checked_at));
        assert_eq!(0, resp.not_after.timestamp());
    }

    #[test]
    fn test_check_certificates() {
        let domain_names = vec!["sha512.badssl.com", "expired.badssl.com"];
        let results = CheckClient::check_certificates(domain_names.as_slice()).unwrap();
        assert_eq!(2, results.len());
        assert!(results.get(0).unwrap().ok);
        assert!(!results.get(1).unwrap().ok);
    }
}
