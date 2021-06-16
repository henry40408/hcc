#[forbid(unsafe_code)]
use std::env;
use std::time::Duration;

use hcc::CheckClient;
use hyper::{Body, Client, Method, Request};
use hyper_rustls::HttpsConnector;
use log::info;
use serde::Serialize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opts {
    /// Domain names to check, separated by comma e.g. sha512.badssl.com,expired.badssl.com
    #[structopt(short, long, env = "DOMAIN_NAMES")]
    domain_names: String,
    /// Interval between each checks in seconds
    #[structopt(short, long, env = "INTERVAL", default_value = "86400")]
    interval: u64,
    /// Pushover API key
    #[structopt(short = "t", long = "token", env = "PUSHOVER_TOKEN")]
    pushover_token: String,
    /// Pushover user key,
    #[structopt(short = "u", long = "user", env = "PUSHOVER_USER")]
    pushover_user: String,
}

/// ref: https://pushover.net/api#messages
#[derive(Serialize)]
struct PushoverBody<'a> {
    token: &'a str,
    user: &'a str,
    message: &'a str,
    title: &'a str,
}

const PUSHOVER_API: &'static str = "https://api.pushover.net/1/messages.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "hcc_pushover=info");
    }

    pretty_env_logger::init();

    let opts: Opts = Opts::from_args();
    let domain_names: Vec<_> = opts.domain_names.split(",").collect();

    info!("check HTTPS certficates every {} seconds", opts.interval);

    let mut timer = tokio::time::interval(Duration::from_secs(opts.interval));
    timer.tick().await; // first tick
    loop {
        info!("check certificate of domain names");
        check_domain_names(&opts, &domain_names).await?;
        info!("done, wait for the next round");
        timer.tick().await; // next tick
    }
}

async fn check_domain_names<S: AsRef<str>>(
    opts: &Opts,
    domain_names: &Vec<S>,
) -> anyhow::Result<()> {
    let check_client = CheckClient::new();
    let results = check_client.check_certificates(domain_names)?;

    let mut futs = vec![];

    let https = HttpsConnector::with_native_roots();
    let pushover_client = Client::builder().build(https);
    for result in results {
        let state_icon = result.state_icon(true);
        let sentence = result.sentence();

        let message = format!("{} {}", state_icon, sentence);
        let body = PushoverBody {
            message: &message,
            user: &opts.pushover_user,
            token: &opts.pushover_token,
            title: &"HTTP Certificate Check",
        };
        let body = serde_urlencoded::to_string(body)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(PUSHOVER_API)
            .body(Body::from(body))?;

        futs.push(pushover_client.request(req));
    }

    futures::future::try_join_all(futs).await?;

    Ok(())
}
