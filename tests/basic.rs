use assert_cmd::Command;
use std::time::Duration;

mod common;

#[test]
fn basic() {
    let server = common::TestServer::new(4000, Duration::from_millis(10));

    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd.arg("localhost:4000");
    cmd.assert().success();

    drop(server);
}

#[test]
fn timeout() {
    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd.arg("--timeout").arg("1000").arg("localhost:4001");
    cmd.assert().failure();
}

#[test]
fn multiple() {
    let servers = vec![
        common::TestServer::new(4002, Duration::from_millis(10)),
        common::TestServer::new(4003, Duration::from_millis(15)),
    ];

    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd
        .arg("--timeout")
        .arg("10000")
        .arg("localhost:4002")
        .arg("localhost:4003");
    cmd.assert().success();

    drop(servers);
}

#[test]
fn multiple_fail() {
    let mut cmd = Command::cargo_bin("wait-for-them").unwrap();
    let cmd = cmd
        .arg("--timeout")
        .arg("10000")
        .arg("localhost:4004")
        .arg("localhost:4005");
    let res = cmd.assert();
    res.failure().code(2);
}
