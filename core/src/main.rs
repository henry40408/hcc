#[forbid(unsafe_code)]

use structopt::StructOpt;

use potential_giggle::{CheckClient, CheckResultJSON};

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
            ref domain_names,
            grace_in_days,
        }) => check_command(opts.json, domain_names, grace_in_days),
        None => Ok(()),
    }
}

fn check_command<S: AsRef<str>>(
    json: bool,
    domain_names: &Vec<S>,
    grace_in_days: i64,
) -> anyhow::Result<()> {
    let client = CheckClient::new_with_grace_in_days(grace_in_days);

    let results = client.check_certificates(domain_names.as_slice())?;

    if json {
        let s = if results.len() > 1 {
            let json: Vec<CheckResultJSON> = results.iter().map(CheckResultJSON::new).collect();
            serde_json::to_string(&json)?
        } else {
            let result = results.get(0).unwrap();
            let json = CheckResultJSON::new(result);
            serde_json::to_string(&json)?
        };
        println!("{0}", s);
    } else {
        for r in results {
            println!("{0}", r);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::check_command;

    #[test]
    fn test_check_command() {
        check_command(false, &vec!["sha512.badssl.com"], 7).unwrap();
    }

    #[test]
    fn test_check_command_json() {
        check_command(true, &vec!["sha512.badssl.com"], 7).unwrap();
    }

    #[test]
    fn test_check_command_expired() {
        check_command(false, &vec!["expired.badssl.com"], 7).unwrap();
    }

    #[test]
    fn test_check_command_expired_json() {
        check_command(true, &vec!["expired.badssl.com"], 7).unwrap();
    }
}
