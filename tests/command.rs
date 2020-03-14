use assert_cmd::Command;
use std::time::Duration;

mod common;

#[test]
fn command_timeout() {
    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd
        .arg("--timeout")
        .arg("1000")
        .arg("localhost:4100")
        .arg("localhost:4101")
        .arg("localhost:4102")
        .arg("---")
        .arg("true");
    cmd.assert().failure().code(3);
}

#[test]
fn command_error() {
    let server = common::TestServer::new(4103, Duration::from_millis(10));

    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd
        .arg("--timeout")
        .arg("1000")
        .arg("localhost:4103")
        .arg("---")
        .arg("false");
    cmd.assert().failure().code(1);

    drop(server);
}

#[test]
fn command_ok() {
    let server = common::TestServer::new(4104, Duration::from_millis(10));

    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd
        .arg("--timeout")
        .arg("1000")
        .arg("localhost:4104")
        .arg("---")
        .arg("true");
    cmd.assert().success();

    drop(server);
}
