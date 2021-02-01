use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, SubsecRound, TimeZone, Utc};
use clap::{App, Arg, SubCommand};
use crossbeam::thread;
use num_format::{Locale, ToFormattedString};
use rustls::{ClientConfig, Session};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use x509_parser::parse_x509_certificate;

const CHECK: &'static str = "check";

const JSON: &'static str = "json";

const DOMAIN_NAME: &'static str = "domain_name";

fn main() -> anyhow::Result<()> {
    let matches = App::new("Potential-Giggle")
        .version("semantic-release")
        .author("Heng-Yi Wu <2316687+henry40408@users.noreply.github.com>")
        .about("Check expiration date of SSL certificate")
        .arg(
            Arg::with_name(JSON)
                .long("json")
                .takes_value(false)
                .required(false)
                .help("Print in JSON format"),
        )
        .subcommand(
            SubCommand::with_name(CHECK)
                .about("Check domain name(s) immediately")
                .arg(
                    Arg::with_name(DOMAIN_NAME)
                        .min_values(1)
                        .help("One or many domain names to check"),
                ),
        )
        .get_matches();

    if let Some(ref m) = matches.subcommand_matches(CHECK) {
        let domain_names: Vec<String> = m
            .values_of(DOMAIN_NAME)
            .expect("Domain name is not given")
            .map(String::from)
            .collect();
        let results = CheckClient::check_certificates(&domain_names)?;
        if matches.is_present(JSON) {
            if results.len() > 1 {
                let json: Vec<CheckResultJSON> = results.iter().map(|r| r.to_json()).collect();
                let s = serde_json::to_string(&json)?;
                println!("{0}", s);
            } else {
                let json = results.get(0).unwrap().to_json();
                let s = serde_json::to_string(&json)?;
                println!("{0}", s);
            }
        } else {
            for r in results {
                println!("{0}", r);
            }
        }
    }

    Ok(())
}

struct CheckClient {
    config: Arc<ClientConfig>,
}

impl CheckClient {
    fn new() -> Self {
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Self {
            config: Arc::new(config),
        }
    }

    fn check_certificate(&self, domain_name: &str) -> anyhow::Result<CheckResult> {
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

    fn check_certificates<S: AsRef<str>>(domain_names: &[S]) -> anyhow::Result<Vec<CheckResult>> {
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

#[derive(Debug)]
struct CheckResult {
    ok: bool,
    checked_at: DateTime<Utc>,
    days: i64,
    domain_name: String,
    expired_at: String,
    not_after: DateTime<Utc>,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // [v] certificate of sha512.badssl.com expires in 512 days
        // [x] certificate of expired.badssl.com is expired
        let mut s = Vec::<String>::new();

        if self.ok {
            s.push("[v]".into());
        } else {
            s.push("[x]".into());
        }

        s.push(format!("certificate of {0}", self.domain_name));

        if self.ok {
            s.push(format!(
                "expires in {0} days ({1})",
                self.days.to_formatted_string(&Locale::en),
                self.expired_at
            ));
        } else {
            s.push(format!("is expired"));
        }

        write!(f, "{}", s.join(" "))
    }
}

#[derive(Serialize, Deserialize)]
struct CheckResultJSON {
    ok: bool,
    checked_at: String,
    days: i64,
    domain_name: String,
    expired_at: String,
}

impl CheckResult {
    fn new(domain_name: &str, checked_at: DateTime<Utc>) -> CheckResult {
        CheckResult {
            ok: false,
            checked_at,
            domain_name: domain_name.to_string(),
            days: 0,
            expired_at: "".to_string(),
            not_after: Utc.timestamp(0, 0),
        }
    }

    fn to_json(&self) -> CheckResultJSON {
        CheckResultJSON {
            ok: self.ok,
            days: self.days,
            domain_name: self.domain_name.clone(),
            checked_at: self.checked_at.to_rfc3339(),
            expired_at: self.expired_at.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, TimeZone, Utc};

    use crate::CheckClient;

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
        let results = CheckClient::check_certificates(domain_names).unwrap();
        assert_eq!(2, results.len());
        assert!(results.get(0).unwrap().ok);
        assert!(!results.get(1).unwrap().ok);
    }
}
