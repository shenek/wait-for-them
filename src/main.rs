use futures::future::join_all;
use std::{env, process::exit, time::Duration};

mod command;
mod options;
mod wait;

fn print_help(error: Option<String>) {
    let error = if let Some(err_msg) = error {
        format!("{}\n", err_msg)
    } else {
        String::new()
    };
    println!(
        "{}Usage:
    wait-for-them [-t timeout] host:port [host:port [host:port...]] [--- command [arg [arg...]]
    -t TIMEOUT | --timeout TIMEOUT  in miliseconds
",
        error
    );
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let options::Options {
        hosts,
        timeout,
        command,
    } = match options::parse(args) {
        Ok(options) => options,
        Err(message) => {
            print_help(message);
            exit(999);
        }
    };

    let futures = hosts
        .iter()
        .map(|addr| wait::Wait::new(addr.clone(), timeout.map(Duration::from_millis)).wait())
        .collect::<Vec<_>>();
    let res = join_all(futures).await;
    let err_count = res.iter().filter(|&e| !e).count();

    if err_count == 0 {
        if let Some(mut cmd) = command {
            let executable = cmd.remove(0);
            match command::exec(&executable, cmd).await {
                // if no status code is present the command is terminated
                // via signal
                Ok(status_code) => exit(status_code.code().unwrap_or(0)),
                Err(_) => exit(999),
            }
        }
    }
    exit(err_count as i32);
}
