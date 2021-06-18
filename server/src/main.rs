#![forbid(unsafe_code)]
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};

use log::info;
use serde::Serialize;
use structopt::StructOpt;
use warp::Filter;

use hcc::{CheckClient, CheckResultJSON};

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opts {
    /// host:port to be bound to the server
    #[structopt(short, long, default_value = "127.0.0.1:9292")]
    bind: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

async fn show_domain_name(
    domain_names: String,
    client: Arc<CheckClient>,
) -> Result<impl warp::Reply, Infallible> {
    let domain_names: Vec<&str> = domain_names.split(',').map(|s| s.trim()).collect();
    let results = match client.check_certificates(domain_names.as_slice()) {
        Ok(r) => r,
        Err(e) => {
            return Ok(warp::reply::json(&ErrorMessage {
                message: format!("{:?}", e),
            }));
        }
    };
    if results.len() == 1 {
        let json = CheckResultJSON::new(results.first().unwrap());
        Ok(warp::reply::json(&json))
    } else {
        let json: Vec<CheckResultJSON> = results.iter().map(|r| CheckResultJSON::new(&r)).collect();
        Ok(warp::reply::json(&json))
    }
}

fn with_client(
    client: Arc<CheckClient>,
) -> impl Filter<Extract = (Arc<CheckClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "hcc_server=info");
    }
    pretty_env_logger::init();

    let opts: Opts = Opts::from_args();
    let client = Arc::new(CheckClient::builder().elapsed(true).build());

    let show_domain_name = warp::path!(String)
        .and(with_client(client))
        .and_then(show_domain_name);

    let routes = warp::any()
        .and(show_domain_name)
        .with(warp::log("hcc_server"));

    let (tx, rx) = mpsc::channel();

    let addr: SocketAddr = opts.bind.parse()?;
    info!("Served on {0}", opts.bind);
    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
        let _ = rx.recv();
        info!("shutdown gracefully");
    });

    ctrlc::set_handler(move || {
        info!("SIGINT received");
        let _ = tx.send(true);
    })
    .expect("unable to bind signal handler");

    tokio::task::spawn(server).await?;

    Ok(())
}
