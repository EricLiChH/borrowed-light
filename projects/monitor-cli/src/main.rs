use clap::{Parser, Subcommand};
use monitor_core::{CheckPolicy, HealthChecker};
use monitor_domain::{CheckOutcome, MonitorTarget};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "monitor", about = "网站健康监测器课程项目")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a target without making a network request.
    Target { name: String, url: String },
    /// Check one website and print a JSON result.
    Check {
        name: String,
        url: String,
        #[arg(long, default_value_t = 5_000)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 2)]
        attempts: usize,
    },
    /// Check every target in a JSON file with bounded concurrency.
    Batch {
        file: PathBuf,
        #[arg(long, default_value_t = 5_000)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 2)]
        attempts: usize,
        #[arg(long, default_value_t = 8)]
        concurrency: usize,
    },
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

#[derive(Deserialize)]
struct TargetConfig {
    name: String,
    url: String,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Target { name, url } => {
            let target = MonitorTarget::new(name, url)?;
            let output = TargetOutput {
                name: target.name(),
                url: target.url(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Check {
            name,
            url,
            timeout_ms,
            attempts,
        } => {
            let target = MonitorTarget::new(name, url)?;
            let policy = CheckPolicy::new(
                Duration::from_millis(timeout_ms),
                attempts,
                Duration::from_millis(100),
                1,
            )?;
            let checker = HealthChecker::new(build_client()?, policy);
            let result = checker.check(&target).await;
            let output = check_output(&result);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Batch {
            file,
            timeout_ms,
            attempts,
            concurrency,
        } => {
            let contents = std::fs::read_to_string(file)?;
            let configs: Vec<TargetConfig> = serde_json::from_str(&contents)?;
            let targets = configs
                .into_iter()
                .map(|config| MonitorTarget::new(config.name, config.url))
                .collect::<Result<Vec<_>, _>>()?;
            let policy = CheckPolicy::new(
                Duration::from_millis(timeout_ms),
                attempts,
                Duration::from_millis(100),
                concurrency,
            )?;
            let checker = HealthChecker::new(build_client()?, policy);
            let results = checker.check_all(&targets).await;
            let output = results.iter().map(check_output).collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder();
    if std::env::var_os("MONITOR_DISABLE_PROXY").is_some() {
        builder = builder.no_proxy();
    }
    builder.build()
}

fn check_output(result: &monitor_domain::CheckResult) -> CheckOutput<'_> {
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
                monitor_domain::CheckFailureKind::Timeout => "timeout",
                monitor_domain::CheckFailureKind::Connect => "connect",
                monitor_domain::CheckFailureKind::Request => "request",
            }),
            reason: Some(reason),
        },
    }
}
