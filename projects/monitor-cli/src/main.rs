use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use monitor_cli::{client, exit, output};
use monitor_core::{CheckPolicy, HealthChecker};
use monitor_domain::{CheckResult, MonitorTarget};
use serde::Deserialize;

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
        #[arg(long, default_value_t = NonZeroUsize::MIN.saturating_add(1))]
        attempts: NonZeroUsize,
    },
    /// Check every target in a JSON file with bounded concurrency.
    Batch {
        file: PathBuf,
        #[arg(long, default_value_t = 5_000)]
        timeout_ms: u64,
        #[arg(long, default_value_t = NonZeroUsize::MIN.saturating_add(1))]
        attempts: NonZeroUsize,
        #[arg(long, default_value_t = NonZeroUsize::MIN.saturating_add(7))]
        concurrency: NonZeroUsize,
    },
}

#[derive(Debug, Deserialize)]
struct TargetConfig {
    name: String,
    url: String,
}

#[tokio::main]
async fn main() {
    // Two different kinds of "no": the command could not run (2), or it ran and
    // at least one site was down (1). A script can tell them apart without
    // parsing the JSON.
    let code = match run(Cli::parse()).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            exit::USAGE
        }
    };
    std::process::exit(code);
}

async fn run(cli: Cli) -> Result<i32, Box<dyn std::error::Error>> {
    match cli.command {
        Command::Target { name, url } => {
            let target = MonitorTarget::new(name, url)?;
            output::print_json(&output::TargetOutput::from(&target))?;
            Ok(exit::SUCCESS)
        }
        Command::Check {
            name,
            url,
            timeout_ms,
            attempts,
        } => {
            let target = MonitorTarget::new(name, url)?;
            let checker = HealthChecker::new(
                client::async_client()?,
                policy(timeout_ms, attempts, NonZeroUsize::MIN),
            );
            let result = checker.check(&target).await;
            output::print_json(&output::CheckOutput::from(&result))?;
            Ok(exit_code_for(std::slice::from_ref(&result)))
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
            let checker = HealthChecker::new(
                client::async_client()?,
                policy(timeout_ms, attempts, concurrency),
            );
            let results = checker.check_all(&targets).await;
            let output = results
                .iter()
                .map(output::CheckOutput::from)
                .collect::<Vec<_>>();
            output::print_json(&output)?;
            Ok(exit_code_for(&results))
        }
    }
}

/// Builds a policy from the command-line flags, keeping the default backoff.
fn policy(timeout_ms: u64, attempts: NonZeroUsize, concurrency: NonZeroUsize) -> CheckPolicy {
    CheckPolicy::builder()
        .total_timeout(Duration::from_millis(timeout_ms))
        .max_attempts(attempts)
        .concurrency(concurrency)
        .build()
}

/// Returns the exit code for a batch: success only if every site answered.
fn exit_code_for(results: &[CheckResult]) -> i32 {
    if results.iter().all(CheckResult::is_reachable) {
        exit::SUCCESS
    } else {
        exit::CHECK_FAILED
    }
}
