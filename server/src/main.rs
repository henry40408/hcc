#[forbid(unsafe_code)]
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use log::info;
use serde::Serialize;
use structopt::StructOpt;
use warp::Filter;

use potential_giggle::{CheckClient, CheckResultJSON};

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opts {
    #[structopt(
        short,
        long,
        help = "host:port to be bound to the server",
        default_value = "127.0.0.1:9292"
    )]
    bind: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

async fn show_domain_name(
    domain_name: String,
    client: Arc<CheckClient>,
) -> Result<impl warp::Reply, Infallible> {
    let result = match client.check_certificate(&domain_name).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(warp::reply::json(&ErrorMessage {
                message: format!("{:?}", e),
            }))
        }
    };
    let json = CheckResultJSON::new(&result);
    Ok(warp::reply::json(&json))
}

fn with_client(
    client: Arc<CheckClient>,
) -> impl Filter<Extract = (Arc<CheckClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "potential_giggle_server=info");
    }
    pretty_env_logger::init();

    let opts: Opts = Opts::from_args();
    let client = Arc::new(CheckClient::new());

    let show_domain_name = warp::path!(String)
        .and(with_client(client))
        .and_then(show_domain_name);

    let routes = warp::any()
        .and(show_domain_name)
        .with(warp::log("potential_giggle_server"));

    let addr: SocketAddr = opts.bind.parse()?;
    info!("Served on {0}", opts.bind);
    warp::serve(routes).run(addr).await;

    Ok(())
}
