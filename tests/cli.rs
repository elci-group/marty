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

#[test]
fn test_scout_json_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("scout").arg("--json").arg(".");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"project_type\""))
        .stdout(predicate::str::contains("\"tree\""))
        .stdout(predicate::str::contains("\"snapshot\""));
}

#[test]
fn test_hotspots_json_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("hotspots").arg("--json");
    cmd.assert().success().stdout(predicate::str::contains("["));
}

#[test]
fn test_version_does_not_start_server() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("marty 0.1"))
        .stdout(predicate::str::contains("HTTP dashboard").not());
}

#[test]
fn test_runs_outside_project_directory() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.current_dir("/tmp").arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("marty 0.1"));
}

#[test]
fn test_beliefs_json_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("beliefs").arg("--json");
    cmd.assert().success().stdout(predicate::str::contains("{"));
}

#[test]
fn test_trace_json_command() {
    let mut cmd = Command::cargo_bin("marty").unwrap();
    cmd.arg("trace").arg("--json");
    cmd.assert().success().stdout(predicate::str::contains("["));
}

#[test]
fn test_state_file_has_version_and_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let state_path = tmp.path().to_string_lossy().to_string();

    let mut visit = Command::cargo_bin("marty").unwrap();
    visit
        .arg("--state")
        .arg(&state_path)
        .arg("visit")
        .arg("/tmp/marty_version_test");
    visit.assert().success();

    let content = std::fs::read_to_string(&state_path).unwrap();
    assert!(content.contains("\"version\""));

    let meta = std::fs::metadata(&state_path).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn test_hotspots_persist_across_invocations() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let state_path = tmp.path().to_string_lossy().to_string();

    let mut visit = Command::cargo_bin("marty").unwrap();
    visit
        .arg("--state")
        .arg(&state_path)
        .arg("visit")
        .arg("/tmp/marty_persist_test");
    visit.assert().success();

    let mut hotspots = Command::cargo_bin("marty").unwrap();
    hotspots
        .arg("--state")
        .arg(&state_path)
        .arg("hotspots")
        .arg("--json");
    hotspots
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp/marty_persist_test"));
}
