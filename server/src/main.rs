#[forbid(unsafe_code)]
use crate::server::{show_domain_name, SharedState};
use actix_web::{middleware, App, HttpServer};
use log::info;
use structopt::StructOpt;

mod server;

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

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "potential_giggle_server=info,actix_web=info");
    env_logger::init();

    let opts: Opts = Opts::from_args();

    info!("Served on {0}", opts.bind);
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .data(SharedState::new())
            .service(show_domain_name)
    })
    .bind(opts.bind)?
    .run()
    .await?;

    Ok(())
}
