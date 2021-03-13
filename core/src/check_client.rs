use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use anyhow::Context;
use chrono::{SubsecRound, TimeZone, Utc};
use futures::{executor, future};
use rustls::{ClientConfig, Session};
use x509_parser::parse_x509_certificate;

use crate::check_result::CheckResult;

/// Client to check SSL certificate
pub struct CheckClient {
    config: Arc<ClientConfig>,
    grace_in_days: i64,
}

impl CheckClient {
    /// Create an instance of client
    ///
    /// ```
    /// # use hcc::CheckClient;
    /// let client = CheckClient::new();
    /// ```
    pub fn new() -> Self {
        Self::new_with_grace_in_days(7)
    }

    /// Create an instance of client with grace period in days
    ///
    /// ```
    /// # use hcc::CheckClient;
    /// let client = CheckClient::new_with_grace_in_days(100);
    /// ```
    pub fn new_with_grace_in_days(grace_in_days: i64) -> Self {
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Self {
            config: Arc::new(config),
            grace_in_days,
        }
    }

    /// Check SSL certificate of one domain name
    ///
    /// ```
    /// # use hcc::CheckClient;
    /// let client = CheckClient::new();
    /// client.check_certificate("sha512.badssl.com");
    /// ```
    pub async fn check_certificate(&self, domain_name: &str) -> anyhow::Result<CheckResult> {
        let checked_at = Utc::now().round_subsecs(0);

        let dns_name = webpki::DNSNameRef::try_from_ascii_str(domain_name)?;
        let mut sess = rustls::ClientSession::new(&self.config, dns_name);
        let mut sock = TcpStream::connect(format!("{0}:443", domain_name))?;
        let mut tls = rustls::Stream::new(&mut sess, &mut sock);

        match tls.write(Self::build_http_headers(domain_name).as_bytes()) {
            Ok(_) => (),
            Err(_) => return Ok(CheckResult::expired(domain_name, &checked_at)),
        };

        let certificates = tls
            .sess
            .get_peer_certificates()
            .with_context(|| format!("no peer certificates found for {0}", domain_name))?;

        let certificate = certificates
            .first()
            .with_context(|| format!("no certificate found for {0}", domain_name))?;

        let not_after = match parse_x509_certificate(certificate.as_ref()) {
            Ok((_, cert)) => cert.validity().not_after,
            Err(_) => return Ok(CheckResult::default()),
        };
        let not_after = Utc.timestamp(not_after.timestamp(), 0);

        let duration = not_after - checked_at;
        let days = duration.num_days();
        Ok(CheckResult {
            ok: days > self.grace_in_days,
            checked_at: checked_at.timestamp(),
            days: duration.num_days(),
            domain_name: domain_name.to_string(),
            not_after: not_after.timestamp(),
            ..Default::default()
        })
    }

    /// Check SSL certificates of multiple domain names
    ///
    /// ```
    /// # use hcc::CheckClient;
    /// let client = CheckClient::new();
    /// client.check_certificates(&["sha256.badssl.com", "sha256.badssl.com"]);
    /// ```
    pub fn check_certificates<S: AsRef<str>>(
        &self,
        domain_names: &[S],
    ) -> anyhow::Result<Vec<CheckResult>> {
        let client = Arc::new(self);

        let mut futs = vec![];
        for domain_name in domain_names {
            let domain_name = domain_name.as_ref();
            let client = client.clone();
            futs.push(client.check_certificate(domain_name));
        }

        let resolved = executor::block_on(future::join_all(futs));
        let mut results = vec![];
        for result in resolved {
            let result = result?;
            results.push(result);
        }
        Ok(results)
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
    use chrono::{TimeZone, Utc};

    use crate::check_client::CheckClient;

    #[tokio::test]
    async fn test_good_certificate() {
        let now = Utc.timestamp(0, 0);
        let domain_name = "sha512.badssl.com";
        let client = CheckClient::new();
        let result = client.check_certificate(domain_name).await.unwrap();
        assert!(result.ok);
        assert!(!result.expired);
        assert!(result.checked_at > 0);
        assert!(now < Utc.timestamp(result.not_after, 0));
    }

    #[tokio::test]
    async fn test_bad_certificate() {
        let domain_name = "expired.badssl.com";
        let client = CheckClient::new();
        let result = client.check_certificate(domain_name).await.unwrap();
        assert!(!result.ok);
        assert!(result.expired);
        assert!(result.checked_at > 0);
        assert_eq!(0, result.not_after);
    }

    #[tokio::test]
    async fn test_check_certificates() {
        let domain_names = vec!["sha512.badssl.com", "expired.badssl.com"];
        let client = CheckClient::new();
        let results = client.check_certificates(domain_names.as_slice()).unwrap();
        assert_eq!(2, results.len());

        let result = results.get(0).unwrap();
        assert!(result.ok);
        assert!(!result.expired);

        let result = results.get(1).unwrap();
        assert!(!result.ok);
        assert!(result.expired);
    }

    #[tokio::test]
    async fn test_check_certificate_with_grace_in_days() {
        let domain_name = "sha512.badssl.com";

        let client = CheckClient::new();
        let result = client.check_certificate(domain_name).await.unwrap();
        assert!(result.ok);
        assert!(!result.expired);

        let client = CheckClient::new_with_grace_in_days(result.days + 1);
        let result = client.check_certificate(domain_name).await.unwrap();
        assert!(!result.ok);
        assert!(!result.expired);
    }
}
