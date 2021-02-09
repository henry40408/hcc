use std::env;

use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand};

use potential_giggle::{CheckClient, CheckResultsJSON};

const CHECK: &'static str = "check";

const JSON: &'static str = "json";

const DOMAIN_NAME: &'static str = "domain_name";

fn main() -> anyhow::Result<()> {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(","))
        .about(crate_description!())
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

    Ok(())
}
