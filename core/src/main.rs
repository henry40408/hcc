use structopt::StructOpt;

use potential_giggle::{CheckClient, CheckResultsJSON};

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opts {
    #[structopt(short, long, help = "Print in JSON format")]
    json: bool,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(about = "Check domain name(s) immediately")]
    Check {
        #[structopt(
            short,
            long = "grace",
            help = "Grace period in days",
            default_value = "7"
        )]
        grace_in_days: i64,
        #[structopt(help = "One or many domain names to check")]
        domain_names: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    match opts.command {
        Some(Command::Check {
            domain_names,
            grace_in_days,
        }) => {
            let client = CheckClient::new_with_grace_in_days(grace_in_days);
            let results = client.check_certificates(domain_names.as_slice())?;
            if opts.json {
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
        None => {}
    }

    Ok(())
}
