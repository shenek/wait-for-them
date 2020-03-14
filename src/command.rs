use std::io;
use tokio::process::Command;

pub async fn exec(program: &str, args: Vec<String>) -> Result<std::process::ExitStatus, io::Error> {
    Command::new(program).args(args).status().await
}
