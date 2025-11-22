use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_visit_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("visit").arg("/tmp");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Visited: /tmp"));
}

#[test]
fn test_hotspots_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("hotspots");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Top Hotspots"));
}

#[test]
fn test_beliefs_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("beliefs");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Beliefs Network"));
}

#[test]
fn test_trace_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("trace");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Recent Activity Trace"));
}
