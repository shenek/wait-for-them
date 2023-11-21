mod command;
mod options;

use std::{env, process::exit, time::Instant};
use wait_for_them::wait_for_them;

// For some reason these "deps" are required with `-D unused-crate-dependencies`
use futures as _;
#[cfg(feature = "http")]
use hyper as _;
#[cfg(feature = "http")]
use hyper_rustls as _;
#[cfg(feature = "ui")]
use indicatif as _;
use regex as _;

fn print_help(error: Option<String>) {
    let error = if let Some(err_msg) = error {
        format!("{}\n", err_msg)
    } else {
        String::new()
    };
    let first_line = if cfg!(feature = "http") {
        "wait-for-them [-t timeout] [-s] host:port|url [host:port|url [host:port|url...]] [-- command [arg [arg...]]"
    } else {
        "wait-for-them [-t timeout] [-s] host:port [host:port [host:port...]] [-- command [arg [arg...]]"
    };
    println!(
        "{}Usage:
    {}
    -s | --silent  don't display any output
    -t TIMEOUT | --timeout TIMEOUT  in milliseconds
        Wait till all links are verified

    wait-for-them -h | --help
        Display help
",
        error, first_line,
    );
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let options::Options {
        to_check,
        timeout,
        command,
        silent,
    } = match options::parse(args) {
        Ok(options) => options,
        Err(message) => {
            print_help(message);
            exit(999);
        }
    };

    let instant = Instant::now();

    let res = wait_for_them(&to_check, timeout, Some(instant), silent).await;

    let err_count = res.iter().filter(|e| e.is_none()).count();

    if err_count == 0 {
        if !silent {
            println!(
                "All items verified in {:.3} seconds.",
                instant.elapsed().as_secs_f32()
            )
        }

        if let Some(mut cmd) = command {
            if !silent {
                println!("Starting '{}'", cmd.join(" "));
            }
            let executable = cmd.remove(0);
            match command::exec(&executable, cmd).await {
                // if no status code is present the command is terminated
                // via signal
                Ok(status_code) => exit(status_code.code().unwrap_or(0)),
                Err(_) => exit(999),
            }
        }
    } else if !silent {
        println!(
            "Failed verify all items in {:.3} seconds.",
            instant.elapsed().as_secs_f32()
        );
    }

    exit(err_count as i32);
}
