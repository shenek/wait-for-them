use futures::future::join_all;
use std::{env, process::exit, time::Duration};

mod wait;

struct Options {
    hosts: Vec<String>,
    timeout: Option<u64>,
}

fn parse_options(args: Vec<String>) -> Result<Options, ()> {
    let mut options = Options {
        hosts: vec![],
        timeout: None,
    };
    let mut read_timeout: bool = false;
    for arg in args {
        if read_timeout {
            options.timeout = Some(arg.parse().map_err(|_| ())?);
            read_timeout = false;
            continue;
        }
        if arg == "-t" || arg == "--timeout" {
            read_timeout = true;
            continue;
        }
        options.hosts.push(arg);
    }
    Ok(options)
}

#[async_std::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let Options { hosts, timeout } = if let Ok(options) = parse_options(args) {
        options
    } else {
        exit(999)
    };

    let futures = hosts
        .iter()
        .map(|addr| wait::Wait::new(addr.clone(), timeout.map(Duration::from_millis)).wait())
        .collect::<Vec<_>>();
    let res = join_all(futures).await;
    let err_count = res.iter().filter(|&e| !e).count();
    exit(err_count as i32);
}
