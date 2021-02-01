use actix_web::middleware::Logger;
use actix_web::HttpServer;
use clap::{Arg, SubCommand};
use log::info;

use potential_giggle::{CheckClient, CheckResultsJSON};

use crate::server::{show_domain_name, SharedState};

mod server;

const CHECK: &'static str = "check";

const JSON: &'static str = "json";

const DOMAIN_NAME: &'static str = "domain_name";

const SERVE: &'static str = "serve";

const BIND: &'static str = "bind";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "potential_giggle=info,actix_web=info");
    env_logger::init();

    let matches = clap::App::new("Potential-Giggle")
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
        .subcommand(
            SubCommand::with_name(SERVE).about("Run a server").arg(
                Arg::with_name(BIND)
                    .short("b")
                    .required(false)
                    .help("host:port to listen to")
                    .default_value("0.0.0.0:9292"),
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
                let json: CheckResultsJSON = results.iter().map(|r| r.to_json()).collect();
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

    if let Some(ref m) = matches.subcommand_matches(SERVE) {
        let bind: &str = m
            .value_of(BIND)
            .expect("host and port are required to bind to");

        info!("listening on {0}...", bind);

        HttpServer::new(|| {
            actix_web::App::new()
                .data(SharedState::new())
                .wrap(Logger::default())
                .service(show_domain_name)
        })
        .bind(bind)?
        .run()
        .await?
    }

    Ok(())
}
