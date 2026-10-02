use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use monitor_cli::{client, exit, output};
use monitor_core::classify_transport_error;
use monitor_domain::{CheckResult, MonitorTarget};
use serde::Deserialize;

/// How long one blocking request may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Parser)]
#[command(name = "monitor-sync", about = "网站健康监测器的顺序 CLI 检查点")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a target without making a network request.
    Target { name: String, url: String },
    /// Check one website with a blocking client.
    Check { name: String, url: String },
    /// Check a JSON target list one item at a time.
    Batch { file: PathBuf },
}

#[derive(Debug, Deserialize)]
struct TargetConfig {
    name: String,
    url: String,
}

fn main() {
    let code = match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            exit::USAGE
        }
    };
    std::process::exit(code);
}

fn run(cli: Cli) -> Result<i32, Box<dyn std::error::Error>> {
    match cli.command {
        Command::Target { name, url } => {
            let target = MonitorTarget::new(name, url)?;
            output::print_json(&output::TargetOutput::from(&target))?;
            Ok(exit::SUCCESS)
        }
        Command::Check { name, url } => {
            let target = MonitorTarget::new(name, url)?;
            let client = client::blocking_client(REQUEST_TIMEOUT)?;
            let result = check(&client, &target);
            output::print_json(&output::CheckOutput::from(&result))?;
            Ok(exit_code_for(std::slice::from_ref(&result)))
        }
        Command::Batch { file } => {
            let configs: Vec<TargetConfig> = serde_json::from_str(&std::fs::read_to_string(file)?)?;
            let targets = configs
                .into_iter()
                .map(|config| MonitorTarget::new(config.name, config.url))
                .collect::<Result<Vec<_>, _>>()?;

            let client = client::blocking_client(REQUEST_TIMEOUT)?;
            let results = targets
                .iter()
                .map(|target| check(&client, target))
                .collect::<Vec<_>>();
            let output = results
                .iter()
                .map(output::CheckOutput::from)
                .collect::<Vec<_>>();
            output::print_json(&output)?;
            Ok(exit_code_for(&results))
        }
    }
}

/// Checks one target and classifies a failure exactly like the async checker.
fn check(client: &reqwest::blocking::Client, target: &MonitorTarget) -> CheckResult {
    match client.get(target.url().clone()).send() {
        Ok(response) => CheckResult::reachable(target, response.status().as_u16()),
        Err(error) => CheckResult::unreachable_with(
            target,
            classify_transport_error(&error),
            error.to_string(),
        ),
    }
}

/// Returns the exit code for a batch: success only if every site answered.
fn exit_code_for(results: &[CheckResult]) -> i32 {
    if results.iter().all(CheckResult::is_reachable) {
        exit::SUCCESS
    } else {
        exit::CHECK_FAILED
    }
}
