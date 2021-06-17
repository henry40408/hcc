#[forbid(unsafe_code)]
use std::env;
use std::time::Duration;

use hcc::CheckClient;
use log::info;
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

    let pushover_client = reqwest::Client::new();
    for result in results {
        let state_icon = result.state_icon(true);
        let sentence = result.sentence();

        let message = format!("{} {}", state_icon, sentence);
        let form = [
            ("message", &message),
            ("user", &opts.pushover_user),
            ("token", &opts.pushover_token),
            (
                "title",
                &format!("HTTP Certificate Check - {}", result.domain_name),
            ),
        ];
        futs.push(pushover_client.post(PUSHOVER_API).form(&form).send());
    }

    futures::future::try_join_all(futs).await?;

    Ok(())
}
