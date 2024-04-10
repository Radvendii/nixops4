mod eval_client;

use anyhow::Result;
use clap::Command;
use eval_client::EvalClient;
use nixops4_core::eval_api::{AssignRequest, EvalRequest, FlakeRequest};
use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn root_command() -> Command {
    Command::new("nixops4")
        .version(VERSION)
        .about("Deploy with Nix and manage resources declaratively")
        .subcommand(
            Command::new("deployments").subcommand(Command::new("list").about("List deployments")),
        )
}

fn main() {
    let matches = root_command().get_matches();
    let r: Result<()> = match matches.subcommand() {
        Some(("deployments", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("list", _)) => {
                    EvalClient::with(|mut c| {
                        let flake_id = c.next_id();
                        // TODO: use better file path string type more
                        let cwd = std::env::current_dir()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        c.send(&EvalRequest::LoadFlake(AssignRequest {
                            assign_to: flake_id.clone(),
                            payload: FlakeRequest { abspath: cwd },
                        }))?;
                        c.send(&EvalRequest::ListDeployments(flake_id.clone()))?;
                        let deployments = c.receive_until(|client| {
                            client.check_error(flake_id.clone())?;
                            let x = client.get_deployments(flake_id.clone());
                            Ok(x.map(|x| x.clone()))
                        })?;
                        for d in deployments {
                            println!("{}", d);
                        }
                        Ok(())
                    })
                }
                Some((name, _)) => {
                    eprintln!("nixops4 internal error: unknown subcommand: {}", name);
                    exit(1);
                }
                None => {
                    // TODO: list instead?
                    eprintln!("nixops4 internal error: no subcommand given");
                    exit(1);
                }
            }
        }
        Some((name, _)) => {
            eprintln!("nixops4 internal error: unknown subcommand: {}", name);
            exit(1);
        }
        None => {
            root_command().print_help().unwrap();
            eprintln!("\nNo subcommand given");
            exit(1);
        }
    };
    handle_result(r);
}

fn handle_result(r: Result<()>) {
    match r {
        Ok(()) => {}
        Err(e) => {
            eprintln!("nixops4 error: {}, {}", e.root_cause(), e.to_string());
            exit(1);
        }
    }
}
