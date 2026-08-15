use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use monitor_domain::{CheckFailureKind, CheckOutcome, CheckResult, MonitorTarget};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct TargetOutput<'a> {
    name: &'a str,
    url: &'a str,
}

#[derive(Serialize)]
struct CheckOutput<'a> {
    name: &'a str,
    url: &'a str,
    reachable: bool,
    status: Option<u16>,
    failure: Option<&'a str>,
    reason: Option<&'a str>,
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Target { name, url } => print_target(&MonitorTarget::new(name, url)?)?,
        Command::Check { name, url } => {
            let target = MonitorTarget::new(name, url)?;
            let client = build_client()?;
            print_result(&check(&client, &target))?;
        }
        Command::Batch { file } => {
            let configs: Vec<TargetConfig> = serde_json::from_str(&std::fs::read_to_string(file)?)?;
            let targets = configs
                .into_iter()
                .map(|config| MonitorTarget::new(config.name, config.url))
                .collect::<Result<Vec<_>, _>>()?;
            let client = build_client()?;
            let results = targets
                .iter()
                .map(|target| check(&client, target))
                .collect::<Vec<_>>();
            let output = results.iter().map(check_output).collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

fn build_client() -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut builder = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5));
    if std::env::var_os("MONITOR_DISABLE_PROXY").is_some() {
        builder = builder.no_proxy();
    }
    builder.build()
}

fn check(client: &reqwest::blocking::Client, target: &MonitorTarget) -> CheckResult {
    match client.get(target.url()).send() {
        Ok(response) => CheckResult::reachable(target, response.status().as_u16()),
        Err(error) => {
            let kind = if error.is_timeout() {
                CheckFailureKind::Timeout
            } else if error.is_connect() {
                CheckFailureKind::Connect
            } else {
                CheckFailureKind::Request
            };
            CheckResult::unreachable_with(target, kind, error.to_string())
        }
    }
}

fn print_target(target: &MonitorTarget) -> Result<(), serde_json::Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&TargetOutput {
            name: target.name(),
            url: target.url(),
        })?
    );
    Ok(())
}

fn print_result(result: &CheckResult) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(&check_output(result))?);
    Ok(())
}

fn check_output(result: &CheckResult) -> CheckOutput<'_> {
    match result.outcome() {
        CheckOutcome::Reachable { status } => CheckOutput {
            name: result.target_name(),
            url: result.target_url(),
            reachable: true,
            status: Some(*status),
            failure: None,
            reason: None,
        },
        CheckOutcome::Unreachable { kind, reason } => CheckOutput {
            name: result.target_name(),
            url: result.target_url(),
            reachable: false,
            status: None,
            failure: Some(match kind {
                CheckFailureKind::Timeout => "timeout",
                CheckFailureKind::Connect => "connect",
                CheckFailureKind::Request => "request",
            }),
            reason: Some(reason),
        },
    }
}
