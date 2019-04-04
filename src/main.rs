use chrono::offset::Local;
use colored::*;
use quicli::prelude::*;
use std::time::{Duration, Instant};
use std::{thread, time};
use structopt::StructOpt;
use url::Url;

/// Emulate kubernetes health check for testing purposes
#[derive(Debug, StructOpt)]
struct Cli {
    /// Delay between health check calls
    #[structopt(long = "delay", short = "d", default_value = "15")]
    delay: u64,
    /// Failures before marking service as unhealthy
    #[structopt(long = "failure-threshold", short = "f", default_value = "3")]
    failures: u64,
    /// Request timeout for health check
    #[structopt(long = "timeout", short = "t", default_value = "10")]
    timeout: u64,
    /// The URL to check
    url: Url,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    let delay = time::Duration::from_secs(args.delay);
    run_health_check_loop(args.url.clone(), delay, args.failures, args.timeout);

    Ok(())
}

fn run_health_check_loop(url: Url, delay: Duration, failure_threshold: u64, timeout: u64) {
    let mut fail_count = 0;

    loop {
        let timestamp = Local::now().to_rfc3339();
        let start_time = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(time::Duration::from_secs(timeout))
            .build()
            .unwrap();

        let res = client.get(url.clone()).send();

        let mut failed = false;

        let check_result = match res {
            Ok(response) => {
                let checked_response = &response.error_for_status_ref();
                if let Err(_status_err) = checked_response {
                    failed = true;
                }
                response.status().to_string()
            }
            Err(connect_err) => {
                failed = true;
                connect_err.to_string()
            }
        };

        if failed {
            fail_count = fail_count + 1;
        } else {
            fail_count = 0;
        }

        println!(
            "{} {} {}: {} ({:#?})",
            if fail_count >= failure_threshold {
                "DOWN     ".red()
            } else if fail_count > 0 {
                "UNHEALTHY".yellow()
            } else {
                "UP       ".green()
            },
            timestamp,
            url.path(),
            if failed {
                check_result.red()
            } else {
                check_result.green()
            },
            start_time.elapsed()
        );

        thread::sleep(delay);
    }
}
