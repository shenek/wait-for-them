use futures::future::join_all;
use std::{env, process::exit, time::Duration};

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
    wait-for-them [-t timeout] host:port [host:port [host:port...]]
    -t TIMEOUT | --timeout TIMEOUT  in miliseconds
",
        error
    );
}

#[async_std::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let options::Options { hosts, timeout } = match options::parse_options(args) {
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
    exit(err_count as i32);
}
