use std::{env, process::exit, time::Instant};

mod command;
mod options;
mod scanner;

fn print_help(error: Option<String>) {
    let error = if let Some(err_msg) = error {
        format!("{}\n", err_msg)
    } else {
        String::new()
    };
    println!(
        "{}Usage:
    wait-for-them [-t timeout] [-s] host:port [host:port [host:port...]] [-- command [arg [arg...]]
    -s | --silent  don't display any output
    -t TIMEOUT | --timeout TIMEOUT  in miliseconds
        Wait till all host:port pairs are opened

    wait-for-them -h | --help
        Display help
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
        silent,
    } = match options::parse(args) {
        Ok(options) => options,
        Err(message) => {
            print_help(message);
            exit(999);
        }
    };

    let instant = Instant::now();

    let res = scanner::perform(&hosts, timeout, instant, silent).await;

    let err_count = res.iter().filter(|e| e.is_none()).count();

    if err_count == 0 {
        if !silent {
            println!(
                "All ports were opened in {:.3} seconds.",
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
            "Failed to open all ports in {:.3} seconds.",
            instant.elapsed().as_secs_f32()
        );
    }

    exit(err_count as i32);
}
