use crate::server::{show_domain_name, SharedState};
use actix_web::{middleware, HttpServer};
use clap::{crate_authors, crate_description, crate_name, crate_version, Arg};
use log::info;

const BIND: &'static str = "bind";

mod server;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "potential_giggle_server=info,actix_web=info");
    env_logger::init();

    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(","))
        .about(crate_description!())
        .arg(
            Arg::with_name(BIND)
                .short("b")
                .help("host:port to be bound to the server")
                .default_value("127.0.0.1:9292"),
        )
        .get_matches();

    if let Some(bind) = matches.value_of(BIND) {
        info!("Served on {0}", bind);
        HttpServer::new(|| {
            actix_web::App::new()
                .wrap(middleware::Logger::default())
                .data(SharedState::new())
                .service(show_domain_name)
        })
        .bind(bind)?
        .run()
        .await?
    }

    Ok(())
}
