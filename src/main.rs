use futures::future::join_all;
use std::{env, process::exit, time::Duration};

mod options;
mod wait;

#[async_std::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let options::Options { hosts, timeout } = if let Ok(options) = options::parse_options(args) {
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
